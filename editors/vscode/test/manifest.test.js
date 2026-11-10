// Manifest/grammar checks: every contributed file must exist and parse,
// and the wiring (language id, commands, config) must stay consistent.
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const root = path.join(__dirname, "..");
const repoRoot = path.join(root, "..", "..");

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8"));
}

test("package.json is a valid extension manifest", () => {
  const manifest = readJson("package.json");
  assert.equal(manifest.name, "luca-code");
  assert.match(manifest.version, /^\d+\.\d+\.\d+$/);
  assert.equal(manifest.main, "./extension.js");
  assert.ok(fs.existsSync(path.join(root, "extension.js")));
  assert.deepEqual(manifest.activationEvents, ["onLanguage:luca"]);
});

test("language, grammar, and snippets wiring is consistent", () => {
  const manifest = readJson("package.json");
  const [language] = manifest.contributes.languages;
  assert.equal(language.id, "luca");
  assert.ok(language.extensions.includes(".lucc"));
  for (const file of [
    language.configuration,
    manifest.contributes.grammars[0].path,
    manifest.contributes.snippets[0].path,
  ]) {
    assert.ok(file, "contributed path is set");
    assert.ok(fs.existsSync(path.join(root, file)), `${file} exists`);
  }
  assert.equal(manifest.contributes.grammars[0].scopeName, "source.luca");
  assert.equal(manifest.contributes.grammars[0].language, "luca");
  assert.equal(manifest.contributes.snippets[0].language, "luca");
  const commands = manifest.contributes.commands.map((command) => command.command);
  assert.ok(commands.includes("luca.runFile"));
  assert.ok(commands.includes("luca.checkFile"));
  assert.ok(commands.includes("luca.createFile"));
  assert.ok(manifest.contributes.configuration.properties["luca.cliPath"]);
  assert.equal(manifest.contributes.configuration.properties["luca.runInTerminal"].default, true);
});

test("check and run are offered in the editor title bar", () => {
  const manifest = readJson("package.json");
  for (const command of ["luca.runFile", "luca.checkFile"]) {
    const entry = manifest.contributes.menus["editor/title"].find((item) => item.command === command);
    assert.ok(entry, `${command} menu entry`);
    assert.equal(entry.when, "editorLangId == luca");
  }
});

test("packaging metadata and license are present", () => {
  const manifest = readJson("package.json");
  assert.match(manifest.bugs.url, /\/issues$/);
  assert.match(manifest.homepage, /editors\/vscode\/README\.md$/);
  for (const file of ["LICENSE", "CHANGELOG.md", "README.md"]) {
    assert.ok(fs.existsSync(path.join(root, file)), `${file} exists`);
  }
  assert.equal(
    fs.readFileSync(path.join(root, "LICENSE"), "utf8"),
    fs.readFileSync(path.join(repoRoot, "LICENSE"), "utf8"),
    "extension LICENSE matches the repository LICENSE"
  );
  const readme = fs.readFileSync(path.join(root, "README.md"), "utf8");
  assert.ok(readme.includes("Luca: Check Luca File"));
  assert.ok(readme.includes("## Limitations"));
});

test("extension version matches the CLI package and product version", () => {
  const manifest = readJson("package.json");
  const cargo = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
  const cargoVersion = /^version = "([^"]+)"/m.exec(cargo)[1];
  const lib = fs.readFileSync(path.join(repoRoot, "src", "lib.rs"), "utf8");
  const productVersion = /PRODUCT_VERSION: &str = "([^"]+)"/.exec(lib)[1];
  assert.equal(manifest.version, cargoVersion);
  assert.equal(manifest.version, productVersion.split(" ")[0]);
});

test("grammar covers Luca keywords and interpolation", () => {
  const grammar = readJson("syntaxes/luca.tmLanguage.json");
  assert.equal(grammar.scopeName, "source.luca");
  const text = JSON.stringify(grammar);
  for (const word of ["raise", "capture", "finally", "func", "ask", "forget", "change"]) {
    assert.ok(text.includes(word), `grammar mentions ${word}`);
  }
  assert.ok(text.includes("comment.line.double-dash"));
  assert.ok(text.includes("variable.interpolation"));
});

test("grammar regexes all compile", () => {
  const grammar = readJson("syntaxes/luca.tmLanguage.json");
  const patterns = [];
  (function collect(node) {
    if (!node || typeof node !== "object") {
      return;
    }
    for (const key of ["match", "begin", "end"]) {
      if (typeof node[key] === "string") {
        patterns.push(node[key]);
      }
    }
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) {
        value.forEach(collect);
      } else if (value && typeof value === "object") {
        collect(value);
      }
    }
  })(grammar.repository);
  assert.ok(patterns.length > 0);
  for (const pattern of patterns) {
    assert.doesNotThrow(() => new RegExp(pattern), `compiles: ${pattern}`);
  }
});

test("snippets parse and cover the core forms", () => {
  const snippets = readJson("snippets/luca.json");
  for (const key of ["Try / capture / finally", "Raise a custom error", "Function definition"]) {
    assert.ok(snippets[key], `snippet present: ${key}`);
    assert.ok(snippets[key].prefix);
    assert.ok(snippets[key].body);
  }
});
