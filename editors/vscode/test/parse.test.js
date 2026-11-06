// Unit tests for the pure helpers in extension.js (no vscode API needed).
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { parseDiagnostics, findOnPath, resolveCli, buildRunCommand } = require("../extension.js");

test("parses error and warning locations", () => {
  const output = [
    "Hello, Luca!",
    "Error at 8:11: 'code' does not exist",
    "Warning at 2:5: Duplicate field 'code'",
  ].join("\n");
  assert.deepEqual(parseDiagnostics(output), [
    { kind: "error", line: 8, column: 11, message: "'code' does not exist" },
    { kind: "warning", line: 2, column: 5, message: "Duplicate field 'code'" },
  ]);
});

test("ignores ordinary program output", () => {
  assert.deepEqual(parseDiagnostics('print("Error at sea")\n42\n'), []);
});

test("findOnPath finds fixtures and misses absent names", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "luca-ext-"));
  const exeName = process.platform === "win32" ? "luca.cmd" : "luca";
  const exePath = path.join(dir, exeName);
  fs.writeFileSync(exePath, process.platform === "win32" ? "@echo off\r\n" : "#!/bin/sh\n");
  if (process.platform !== "win32") {
    fs.chmodSync(exePath, 0o755);
  }
  assert.equal(findOnPath("luca", dir), exePath);
  assert.equal(findOnPath("definitely-not-luca", dir), null);
});

test("resolveCli honors absolute paths and PATH lookup", () => {
  assert.equal(resolveCli("/no/such/luca-binary"), null);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "luca-ext-cli-"));
  const exeName = process.platform === "win32" ? "luca.cmd" : "luca";
  const exePath = path.join(dir, exeName);
  fs.writeFileSync(exePath, process.platform === "win32" ? "@echo off\r\n" : "#!/bin/sh\n");
  if (process.platform !== "win32") {
    fs.chmodSync(exePath, 0o755);
  }
  const savedPath = process.env.PATH;
  process.env.PATH = dir;
  try {
    assert.equal(resolveCli("luca"), exePath);
    assert.equal(resolveCli("  "), exePath); // blank falls back to "luca"
  } finally {
    process.env.PATH = savedPath;
  }
});

test("buildRunCommand quotes paths with spaces", () => {
  assert.equal(
    buildRunCommand("/Users/luca/.cargo/bin/luca", "/tmp/my dir/hello.lucc"),
    '"/Users/luca/.cargo/bin/luca" "/tmp/my dir/hello.lucc"'
  );
});
