# 11. Errors

Some mistakes stop the program — that is a plain and useful behavior. This chapter is about the parts of Luca that let you *respond* to a failing operation instead of crashing.

## The shape of trouble

When something cannot proceed, Luca prints one message giving what and where, then stops:

```text
Error at 1:9: Division by zero
```

Line 1, column 9 — the failing expression. If the program had printed earlier lines, they are not shown: an uncaught error discards pending output (warnings/errors still arrive), which keeps the terminal showing only the diagnosis.

## `try to` — run something, catch the error

```luca
import File

def var contents (str) = none
try to :
    contents (str) = File.read("missing.txt")
    print("Read worked")
capture error :
    print("Could not read the file.")
finally :
    print("Cleanup finished")
```

```text
Could not read the file.
Cleanup finished
```

Three blocks, each optional except `try to :` itself:

- `try to :` — the risky part. A statement in it fails → the rest of the block is skipped.
- `capture error :` — runs only when the try block failed. Only runtime errors land here, not syntax mistakes.
- `finally :` — always runs last: after success, after capture, and once before an error keeps propagating outward.

Errors thrown inside functions and modules propagate upward too: `capture error` in the caller catches a failure from the callee. Warnings, by contrast, are never errors: out-of-range reads, removing absent entries, duplicate keys, missing-deletes — they print a `Warning at line:column` and continue.

## The caught error is a dictionary

Inside `capture error :`, the name `error` is bound to a dictionary with the details. Built-in failures carry the location (`line`, `column`) and mark `code` as `none`:

```luca
import Math

try to :
    print(Math.sqrt(-1))
capture error :
    print(error["code"])
    print(error["column"])
```

```text
none
15
```

Column `15` points into the failing call on line 2 — the location of the operation that went wrong, not of the capture block.

## Custom errors: `def error` and `raise`

A custom error declares at least `code` and `message`:

```luca
def error Validation :
    code = "VALIDATION"
    message = "no good input"

try to :
    raise Validation(code = "EMPTY_NAME", message = "The name was empty")
capture error :
    print(error["code"])
    print(error["message"])
    print(error["line"])
```

```text
EMPTY_NAME
The name was empty
6
```

Reading the rules plainly: each `raise Name(field = value, ...)` **replaces** the declared field value for this one raise; parentheses after the name are required (`raise Validation` alone is an error: `Expected '(' after error name`); `line`/`column` tell the raise's spot. Unknown field names or mismatched types are themselves errors — an override must name a field the error actually declares, with a value of that field's type. `raise` is a statement, never a value-producing expression.

Then the big easy claims: recursion is capped, warnings are nonfatal, and `System.exit(n)` — the standard library's way to terminate — cannot be caught, though a `finally` block still runs before the process ends.

Next: [input and the library](12-input-and-library.md).
