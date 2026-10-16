// Node integration tests for the Luca WASM host (M3 B3).
//
// Exercises the actual compiled artifact
// (target/wasm32-unknown-unknown/debug/luca_wasm_host.wasm) through its
// exported C ABI. Run from this directory after building:
//   cargo build --target wasm32-unknown-unknown
//   node --test smoke.mjs
import { describe, it, before } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = dirname(fileURLToPath(import.meta.url));
const wasmPath = join(root, 'target', 'wasm32-unknown-unknown', 'debug', 'luca_wasm_host.wasm');

let api;
let memory;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function bytes() {
  return new Uint8Array(memory.buffer);
}
function view() {
  return new DataView(memory.buffer);
}

// Write a JS string into guest memory; caller frees with free(ptr, len).
function writeStr(text) {
  const data = encoder.encode(text);
  const ptr = api.luca_alloc(data.length);
  assert.notEqual(ptr, 0);
  bytes().set(data, ptr);
  return [ptr, data.length];
}
function free(ptr, len) {
  api.luca_free(ptr, len);
}
// Read a guest response (ptr + out-param length), decode, and free it.
function readResponse(ptr, lenPtr) {
  const len = view().getUint32(lenPtr, true);
  const text = decoder.decode(bytes().subarray(ptr, ptr + len));
  free(ptr, len);
  return text;
}
function allocLen() {
  const ptr = api.luca_alloc(4);
  assert.notEqual(ptr, 0);
  return ptr;
}
// Call luca_run and return the parsed response document.
function run(handle, source) {
  const [ptr, len] = writeStr(source);
  const lenPtr = allocLen();
  const out = api.luca_run(handle, ptr, len, lenPtr);
  free(ptr, len);
  assert.notEqual(out, 0, 'luca_run must not return null for valid args');
  const doc = JSON.parse(readResponse(out, lenPtr));
  free(lenPtr, 4);
  return doc;
}
function check(handle, entry) {
  const [ptr, len] = writeStr(entry);
  const lenPtr = allocLen();
  const out = api.luca_check(handle, ptr, len, lenPtr);
  free(ptr, len);
  assert.notEqual(out, 0);
  const doc = JSON.parse(readResponse(out, lenPtr));
  free(lenPtr, 4);
  return doc;
}
function addFile(handle, path, content) {
  const [pp, pl] = writeStr(path);
  const [cp, cl] = writeStr(content);
  try {
    assert.equal(api.luca_add_file(handle, pp, pl, cp, cl), 0);
  } finally {
    free(pp, pl);
    free(cp, cl);
  }
}
function pushInput(handle, line) {
  const [ptr, len] = writeStr(line);
  try {
    assert.equal(api.luca_push_input(handle, ptr, len), 0);
  } finally {
    free(ptr, len);
  }
}
function makeHost() {
  const handle = api.luca_host_new();
  assert.notEqual(handle, 0);
  return handle;
}
function dropHost(handle) {
  assert.equal(api.luca_host_free(handle), 0);
}

before(async () => {
  const binary = readFileSync(wasmPath);
  const module = await WebAssembly.compile(binary);
  const instance = await WebAssembly.instantiate(module, {});
  api = instance.exports;
  memory = api.memory;
  assert.ok(memory instanceof WebAssembly.Memory);
});

describe('execution', () => {
  it('runs successfully with exact output ordering', () => {
    const h = makeHost();
    try {
      assert.deepEqual(run(h, 'print("one")\nprint("two")\n'), {
        status: 'ok',
        output: ['one', 'two'],
        warnings: [],
      });
    } finally {
      dropHost(h);
    }
  });

  it('returns exit outcomes without terminating node', () => {
    const h = makeHost();
    try {
      assert.deepEqual(
        run(h, 'import System\nprint("bye")\nSystem.exit(7)\n'),
        { status: 'exit', code: 7, output: ['bye'], warnings: [] }
      );
      // Node is alive and the handle is reusable.
      assert.deepEqual(run(h, 'print("again")\n'), {
        status: 'ok',
        output: ['again'],
        warnings: [],
      });
    } finally {
      dropHost(h);
    }
  });

  it('reports runtime errors with locations', () => {
    const h = makeHost();
    try {
      const doc = run(h, 'print(10 / 0)\n');
      assert.equal(doc.status, 'error');
      assert.match(doc.message, /Division by zero/);
      assert.equal(typeof doc.line, 'number');
      assert.equal(typeof doc.column, 'number');
    } finally {
      dropHost(h);
    }
  });
});

describe('modules', () => {
  it('resolves virtual imports and reports missing modules', () => {
    const h = makeHost();
    try {
      addFile(h, '/Utils.lucc', 'def var version = "1.0"\n');
      assert.deepEqual(run(h, 'import Utils\nprint(Utils.version)\n'), {
        status: 'ok',
        output: ['1.0'],
        warnings: [],
      });
      const missing = run(h, 'import Nope\n');
      assert.equal(missing.status, 'error');
      assert.match(missing.message, /missing module 'Nope'/);
    } finally {
      dropHost(h);
    }
  });

  it('rejects directories named like modules', () => {
    const h = makeHost();
    try {
      const [pp, pl] = writeStr('/lib/DirMod.lucc');
      try {
        assert.equal(api.luca_add_dir(h, pp, pl), 0);
      } finally {
        free(pp, pl);
      }
      const doc = run(h, 'import DirMod\n');
      assert.equal(doc.status, 'error');
      assert.match(doc.message, /missing module 'DirMod'/);
    } finally {
      dropHost(h);
    }
  });
});

describe('input', () => {
  it('serves the scripted queue in order', () => {
    const h = makeHost();
    try {
      pushInput(h, 'Luca');
      assert.deepEqual(run(h, 'def const n (str) = ask "Name?"\nprint("hi {n}")\n'), {
        status: 'ok',
        output: ['Name?', 'hi Luca'],
        warnings: [],
      });
    } finally {
      dropHost(h);
    }
  });

  it('denies input with the frozen diagnostic', () => {
    const h = makeHost();
    try {
      assert.equal(api.luca_set_deny_input(h, 1), 0);
      const doc = run(h, 'def const n (str) = ask "Name?"\n');
      assert.equal(doc.status, 'error');
      assert.match(doc.message, /Failed to read input: input is not available in this host/);
      assert.equal(api.luca_set_deny_input(h, 0), 0);
      pushInput(h, 'back');
      assert.deepEqual(run(h, 'def const n (str) = ask "Name?"\nprint(n)\n'), {
        status: 'ok',
        output: ['Name?', 'back'],
        warnings: [],
      });
    } finally {
      dropHost(h);
    }
  });
});

describe('repeated calls and warnings', () => {
  it('isolates per-call output while files persist', () => {
    const h = makeHost();
    try {
      addFile(h, '/K.lucc', 'def var k = 1\n');
      assert.deepEqual(run(h, 'import K\nprint("first")\n'), {
        status: 'ok',
        output: ['first'],
        warnings: [],
      });
      // Second call sees only its own output; the scripted file persists.
      assert.deepEqual(run(h, 'import K\nprint("second")\n'), {
        status: 'ok',
        output: ['second'],
        warnings: [],
      });
    } finally {
      dropHost(h);
    }
  });

  it('captures warnings alongside output', () => {
    const h = makeHost();
    try {
      const doc = run(h, 'def set s = {1, 1}\nprint("ok")\n');
      assert.equal(doc.status, 'ok');
      assert.deepEqual(doc.output, ['ok']);
      assert.equal(doc.warnings.length, 1);
      assert.equal(doc.warnings[0][2], 'Duplicate set value');
    } finally {
      dropHost(h);
    }
  });

  it('round-trips quotes, backslashes, newlines, and non-ASCII exactly', () => {
    const h = makeHost();
    try {
      const doc = run(
        h,
        'print("say \\"hi\\"")\nprint("a\\\\b")\nprint("x\\ny")\nprint("héllo wörld ✓")\n'
      );
      assert.deepEqual(doc, {
        status: 'ok',
        output: ['say "hi"', 'a\\b', 'x\ny', 'héllo wörld ✓'],
        warnings: [],
      });
    } finally {
      dropHost(h);
    }
  });
});

describe('robustness and memory', () => {
  it('rejects invalid handles without trapping', () => {
    const lenPtr = allocLen();
    try {
      assert.equal(api.luca_run(0, 0, 0, lenPtr), 0);
      assert.equal(api.luca_host_free(0), -1);
      assert.equal(api.luca_set_deny_input(0, 1), -1);
    } finally {
      free(lenPtr, 4);
    }
  });

  it('reports malformed UTF-8 as a document', () => {
    const h = makeHost();
    try {
      const ptr = api.luca_alloc(2);
      bytes().set([0xff, 0xfe], ptr);
      const lenPtr = allocLen();
      const out = api.luca_run(h, ptr, 2, lenPtr);
      free(ptr, 2);
      assert.notEqual(out, 0);
      const doc = JSON.parse(readResponse(out, lenPtr));
      free(lenPtr, 4);
      assert.equal(doc.status, 'host-error');
    } finally {
      dropHost(h);
    }
  });

  it('survives repeated alloc/run/free cycles', () => {
    const h = makeHost();
    try {
      for (let i = 0; i < 50; i++) {
        const doc = run(h, `print(${i})\n`);
        assert.deepEqual(doc.output, [String(i)]);
      }
    } finally {
      dropHost(h);
    }
  });
});

describe('check', () => {
  it('validates scripted graphs and reports diagnostics', () => {
    const h = makeHost();
    try {
      addFile(h, '/main.lucc', 'import Utils\nprint(Utils.version)\n');
      addFile(h, '/Utils.lucc', 'def var version = "1.0"\n');
      assert.deepEqual(check(h, '/main.lucc'), { status: 'ok', diagnostics: [] });
      addFile(h, '/bad.lucc', 'def var = 1\n');
      const doc = check(h, '/bad.lucc');
      assert.equal(doc.status, 'ok');
      assert.equal(doc.diagnostics.length, 1);
    } finally {
      dropHost(h);
    }
  });
});
