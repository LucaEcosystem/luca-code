# The `luca` command-line reference

One binary, four behaviors: run a program, validate a program, show usage, show the version. `luca --help` (exactly what it prints):

```text
Usage: luca <program.lucc> [args...]
       luca check <program.lucc> [--format human|json]

Run a Luca Code program. Extra arguments are visible to the program via System.args().

Options:
  --help     Show this help and exit
  --version  Show the Luca Code version and exit

Check options:
  luca check <program.lucc> [--format human|json]
    Validate a program without executing it. Prints diagnostics and exits
    0 when valid, 1 when validation errors are found, 2 for usage or
    internal validator failures.

Examples:
  luca hello.lucc
  luca hello.lucc extra args here
  luca check hello.lucc
  luca check hello.lucc --format json
```

## `luca --version`

```text
luca 1.0.3 Beta
```

Exit code `0`.

## Running a program

```sh
luca hello.lucc [extra words...]
```

- The file must end in `.lucc` — anything else prints `Luca Code source files must use the .lucc extension` and exits `2`.
- Extra words after the program path are passed through to the program: `System.args()` returns the whole command line (`luca`, the program path, then those extras).
- Exit code `0` when the program finished (or when the program called `System.exit(code)` — the OS then sees your `code`).
- A runtime error prints one `Error at line:column: message` diagnosis and exits `1`. Output printed *before* the error is discarded, so the terminal shows only the diagnosis (see the [language reference](language-reference.md#10-errors-and-warnings)).

## `luca check` — validate without executing

```sh
luca check program.lucc [--format human|json]
```

`check` parses the program **and its whole import graph** (following every `import Name` transitively) and reports diagnostics. It never executes your code:

- `print` produces no output and `ask` never prompts;
- `File.write`/`append`/`delete` never touch the filesystem;
- `System.exit` never terminates, and module top-level code never runs.

That restraint has a flip side: dynamic errors — bad argument types on runtime values, division by zero on computed values, invalid `ask` conversions of real input, declared types that only match at runtime — can only appear when you run the program with `luca <file.lucc>`. `check` covers syntax and the import graph only.

The import walk is bounded so a pathological graph cannot stall validation: at most **100 modules** (`Too many modules`) and **32 import levels** (`Import depth exceeded`). Source nesting is likewise bounded at 32 levels by the parser (`Block/Expression nesting is too deep`), and a clean file is silent.

Formats:

- **human** (default): one diagnostic per line, `Error at line:column: message`; blank output and exit `0` when valid.
- **json**: a stable array `[{"message":"...","line":N,"column":M}]`; `[]` when valid.

Exit codes:

| Code | Meaning |
|---|---|
| `0` | Valid (JSON prints `[]`) |
| `1` | Validation errors found |
| `2` | Usage or internal validator failure — printed to stderr with a `luca check:` prefix and never as JSON diagnostics, e.g. `luca check: Could not read program.lucc: No such file or directory (os error 2)` |

A worked pair, since this distinction matters when trying to catch typos with `check`:

```text
$ luca check syntax-bad.lucc          -- a `then` with no colon
Error at 1:29: Expected ':' after 'then'
$ echo $?
1                                     ← syntax: caught by check, exit 1

$ luca check type-bad.lucc            -- def var age (int) = "hi"
[]
$ echo $?
0                                     ← dynamic: check sees nothing wrong,
$ luca type-bad.lucc                     because declared types are checked
Error at 1:1: Expected int, found str    when the program runs
$ echo $?
1
```

## Diagnostic locations

Every line/column pair refers to `program.lucc` itself and, for imported files, to the imported source with an `In module 'Name':` prefix on propagating errors. See [cli integration in the language reference](language-reference.md#14-command-line-behavior).

## Having trouble?

[troubleshooting](../learn/troubleshooting.md) collects the common install and first-run problems.
