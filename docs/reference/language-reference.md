# Luca Code language reference

This is the complete, user-facing description of the language as implemented for **Luca Code 1.0 Beta** (package version **1.0.3 Beta**: `luca --version` prints `luca 1.0.3 Beta`). It describes exactly what runs; for step-by-step learning, start with the [guided tour](../learn/tour/index.md).

Everything here is verified behavior. When this reference and any doc disagree, this page is the correction target — and if a discrepancy turns out to be a defect in the implementation, report it in the repository.

---

## 1. Source files and execution model

- Programs are plain-text files with the `.lucc` extension; every other extension is refused (`Luca Code source files must use the .lucc extension`).
- Execution starts at the first statement and runs top to bottom. There is no `main()` and no entry-point ceremony.
- Statements are line-oriented. No separator/semicolon exists.
- Importable files use the same `.lucc` extension and syntax, with no export declarations (see §9).

## 2. Lexical rules

| Element | Rule |
|---|---|
| Identifiers | Letters, digits, `_`; must not start with a digit; case-sensitive; no spaces; cannot be keywords |
| Keywords | e.g. `def`, `var`, `const`, `func`, `return`, `if`, `then`, `else`, `for`, `while`, `times`, `repeat`, `from`, `stop`, `skip`, `change`, `forget`, `import`, `try`, `to`, `capture`, `error`, `finally`, `error`, `raise`, `ask`, `is`, `and`, `or`, `not`, `none`, `true`, `false`, collection words (`list`, `tuple`, `set`, `dict`), `print` |
| Comments | `--` to end of line; `[[ ... ]]` for a multiline block (`[[` always opens a block, anywhere it appears) |
| Strings | Double quotes, single line only (`String literal cannot contain a raw newline`). Escapes: `\n` `\r` `\t` `\"` `\\`; any other escaped char is taken literally |
| Interpolation | `{name}` inside any string value substitutes the *declared variable* `name`, rendering it for display. Braces must wrap exactly one variable name: `{}`, expressions, or unknown names are errors (`Interpolation supports variable names only in this milestone`, `'ghost' is not declared`) |

## 3. Types and values

| Type | Domain | Notes |
|---|---|---|
| `int` | 64-bit signed integers | Overflow is an error, never a wrap |
| `dec` | Exact base-10 decimals | Arbitrary meaningful scale with retained display precision; no binary floating-point fuzz |
| `str` | Unicode text | Double-quoted; compare by exact bytes |
| `bool` | `true` / `false` | |
| `uni` | Any real value | Universal type for heterogeneous slots |
| `none` | The absence value | Occupies any declared type; prints as `none` |

`none` is usable for print and comparisons (`x is == to none`) but any operation that needs a real value (arithmetic, logic, conditions, strict library arguments) rejects it.

**Numeric model.** `10 / 4` is `2.5`, not truncated integer division — division never truncates; whole-number division yields `dec` (`6 / 2` prints `3.0`). `int`-`dec` mixes produce `dec`; otherwise integer arithmetic stays `int`. Decimal scale is preserved exactly (`20.00 + 1.50` → `21.50`; `0.1 + 0.2` → `0.3`). Division/modulo by zero and integer overflow are errors. There is no exponentiation operator in the language — see `Math.pow`.

## 4. Bindings: `def var`, `def const`

```luca
def var age (int) = 16          -- mutable, typed
def const name (str) = "Luca"   -- immutable, typed
def var mood = "calm"           -- type inferred (str)
def var anything (uni) = 1      -- uni accepts any value
```

- Reassign an existing variable by repeating its name with its annotation: `age (int) = age + 1`. The type annotation must match the declared type.
- `def const` may not be reassigned, cleared, or have its contents mutated (`Cannot change constant`). Collections through a `const` binding are deep-copied on read, so constant contents can never change via any alias.
- Exactly one binding per name per scope; duplicates are errors (`'x' is already declared`).
- `change var name (type) = value` changes a mutable variable's stored type and value in one step; `change` is the only conversion mechanism — there are no `int()`/`str()`/`dec()` casts or implicit coercions.
- `forget var name (type)` (or `forget const name (type)`) removes the binding, after which the name can be declared again.
- `name.clear()` on a mutable variable (scalar) sets it to `none` while keeping the binding and type; it errors on a `const`. On collections it empties the collection (§8).
- A declared type accepts `none` as a value. Collections declared without an element type keep per-entry flexibility; `uni` is the permissive element type.

## 5. Expressions and operators

Precedence, highest first; parentheses group freely:

| Level | Operators |
|---|---|
| 1 | `not` |
| 2 | `*` `/` `%` |
| 3 | `+` `-` (unary `-` on numbers likewise) |
| 4 | worded comparisons |
| 5 | `and` |
| 6 | `or` |

Multiple collection indexers apply left-to-right (`matrix[1][2]`), and postfix member/method forms chain the same way.

- Arithmetic `+ - * / %` on numbers; `+` on `str + str` only (mixed → `Arithmetic requires matching numeric types; '+' also accepts two str values`).
- Unary `-` needs a space on its left (`3 - -2`); without one, `--` reads as a comment.
- Comparisons, worded exactly: `is == to`, `is != to`, `is > to`, `is >= to`, `is < to`, `is <= to`.
  - Equality is content-based and recursive for collections; type-mismatched equality operands are errors.
  - Ordering is defined for `int`/`int`, `dec`/`dec`, mixed `int`/`dec`, and `str`/`str` (lexicographic, case-sensitive). Everything else is an order error.
- `and`/`or` require `bool` operands and short-circuit; `not` negates a `bool`.
- Conditions (in `if`/`while`) must be plain booleans — no truthiness (`Condition must be bool; found 2`).

## 6. Control flow

Blocks open with `:` and indent one level (spaces or tabs, used consistently per block; empty blocks are errors).

```luca
if condition then :
    ...
else if condition then :
    ...
else :
    ...
```

Loops — three forms, all with `repeat :` bodies:

- `for N times repeat :` — `N` an `int` expression, evaluated **once** before the loop; non-negative.
- `while condition repeat :` — boolean condition, no hidden iteration cap.
- `for item from collection repeat :` — iterates list/tuple/set elements and dictionary keys; the loop item is a fresh binding available inside the block.

`stop` exits the innermost loop; `skip` continues with the next iteration. Either outside a loop is an error (`stop/skip outside loop`).

Loops are statements (they produce no value); so are `if` blocks, `try` blocks, and all declarations.

Conditionals' bodies accept any statements, including `def func`, `import`, `try`, and loops.

## 7. Functions

```luca
def func describe(name (str) = "you") (str):
    return "Hi {name}"
```

- Signature: `def func name(parameters) (return_type):` — parameters and return type are each optional; either write an explicit return type or omit it entirely, with no inference.
- Parameters: `pname (type) = default`. Calls are positional only; keyword-style `f(a = x)` is a syntax error (that `field = value` shape belongs to `raise`).
- Untyped parameters accept any value; typed parameters check on call. Argument count mismatches are errors (`Expected 2 arguments, found 3`; missing one → `Missing argument for 'b'`).
- Untyped return: functions may vary their return types; bare `return` and reaching the end both yield `none`.
- Typed return: every returned value must match the annotation; a valueless `return` under a non-`none`-able annotation is an error.
- `return` outside a function is an error.
- Recursion is supported; nesting limit is 64 (`Recursion limit exceeded` beyond that). Recursion depth errors are capturable by `try to` like other runtime errors.
- Closures may read and mutate captured `var` bindings; top-level bindings act as globals. Default values evaluate at call time, in the caller's scope.
- Primitive arguments pass by value — reassigning a parameter inside does not affect the caller's variable. Collections are shared references, so mutations by the callee are visible to the caller.
- No overloading: a fresh `def func` with an already-taken name is an error.

## 8. Collections

| Kind | Notation | Mutability | Element rules |
|---|---|---|---|
| `list` | `[1, 2, 3]` | Yes | One element type annotation; 1-based indexing |
| `tuple` | `(123, true, "Luca")` | Yes (a single-element tuple is `(x,)`) | Same as lists |
| `set` | `{"alpha", "stable"}` | Yes | Unique members; adding an existing member warns and no-ops |
| `dict` | `{"Name" = "Luca", "Age" = 16}` | Yes | String keys; each entry carries its own value type |

An element annotation is optional; `uni` is the element type to use when entries differ.

### Indexing

- Indexing is 1-based and is an expression: `names[1]`, discovering nested values via chained indexing.
- Missing index/key reads **warn** and yield `none`: `Warning at 2:8: Index out of range`, `Warning at 2:13: Key 'Phone' not found`.
- Writing: `change names[1] = "Jordan"` (element type must match), `change person["Age"] = 17` (existing entries only). Out-of-range writes are errors; `change` of a missing key is an error.
- Dictionaries: keys are plain double-quoted strings (no interpolation in keys). Tuples index exactly like lists.

### Method statements (not expressions)

`.add`, `.remove`, `.clear()` appear only as statements; `.contains`, `.length`, and `[i]` are expressions.

| Method | Lists/tuples/sets | Dictionaries |
|---|---|---|
| `.add(v)` | list: appends; set: add-or-warn (`Set value already exists`) | `.add(key, value)`: add-or-warn (`Duplicate dictionary key` — first value kept) |
| `.remove(v)` | removes (absent → warn + no change) | `.remove(key)` removes entry (absent → warn + no change) |
| `.contains(...)` | `.contains(value)` → bool | `.contains(key = k)` or `.contains(value = v)` |
| `.clear()` | empties — list becomes `[]`, tuple `()`, set `{}` | empties — dict becomes `{}` |

Ranges and other index arithmetic do not exist; `.length` covers sizes:

- `.length` on a collection → item/entry count; on a string → character count; on `int`/`dec` → digit count excluding sign and decimal point (invalid for `bool`).
- `.length(decimals)` on a `dec` → its decimal-place count; `.length(items)` on a collection → item count (a spelled-out synonym).

### Display and equality

- Reading displays values as `["Luca", "Alex"]`, `(123,)` for single tuples, `{"alpha"}`, `{"Name" = "Luca", "Age" = 16}` — note strings stay quoted inside collections.
- Equality is content-based and recursive; sets/dicts compare as unordered mappings/sets. Copy/sharing semantics match functions (§7): assignments and parameters alias storage; `const` bindings deep-copy on read.
- Iteration: `for item from x repeat :`; mutating the *iterated* collection while iterating is an error.

**Lexical note:** `[[` always begins a multiline comment. To write a list whose first element is a list, keep a space: `[ [1, 2], [3] ]`.

## 9. Modules

```luca
import Helpers
print(Helpers.company)
print(Helpers.hello("you"))
```

- `import Name` (single identifier, no path or alias) resolves `Name.lucc` from the program's own directory first, then the working directory. Import name and file name must match exactly, including capitalization.
- Every top-level declaration (vars, consts, collections, functions, errors) is automatically exported through the module's real namespace: `Module.member` reads, `Module.func(args)` calls. Mutable module state persists between calls; modules cannot import the importer's bindings.
- Each module's top level runs exactly once, lazily, on first import; later imports reuse the prepared module. Initialization errors propagate to the importer (capturable).
- Circular imports are errors (`Circular import 'X'`); so are missing files (`missing module 'X'`), unknown members (`Module 'X' has no member 'y'`), calling a non-function member, and printing a module.
- Errors inside a module report with an `In module 'X': …` prefix.
- Built-in modules (`Math`, `Text`, `File`, `System`) exist natively; importing them always binds the built-in even if a same-named `.lucc` file exists. Module-defined custom errors (§10) are definitions, not values — `ErrorName.field` access or `Module.Error` reads error out; only functions from within the module can raise them into your capture.

## 10. Errors and warnings

**Errors stop execution** (the interpreter discards buffered `print` output on an uncaught one, so only the `Error at line:column: message` line and any warnings precede the diagnosis you see). **Warnings are nonfatal** — they report a line+column and execution continues.

### Structural forms

```luca
try to :
    ...
capture error :
    ...
finally :
    ...
```

- `capture error :` and `finally :` are optional; `try to :` is not.
- The capture block runs only on runtime failure of the try block (syntax/parse errors happen before any execution). The finally block always runs — after success, after capture, and once before an error keeps propagating outward — even for `return`/`stop`/`skip` flowing out of these blocks.
- Signals behave uniformly: `return`/`stop`/`skip` inside the try body still run `finally` first.

### `def error` and `raise`

```luca
def error Validation :
    code = "VALIDATION"
    message = "no good input"

raise Validation(code = "EMPTY_NAME")
```

- The body establishes at least `code` and `message` via bare assignments (`code = 404`); the initial values of both are `none` until assigned. Declaring `def var code` inside the body collides (`'code' is already declared`). The body runs once, at declaration time, in its own fresh scope.
- `raise Name(field = value, ...)` is a **statement**: parentheses mandatory; zero or more `field = value` overrides (comma-separated, a trailing comma is fine); each field must already exist on the error and accept that value's type (`none` accepted); duplicate overrides warn and keep the first; overriding with a different type is an error (`Cannot change type of field 'f'`).
- Raising unknown/non-error names, unknown fields are errors. `raise` positions carry the source `line`/`column` of the raise.
- `raise Module.Error(...)` via qualified paths is not supported — only errors declared or imported as bare names.
- Module members named after errors are definitions; only module functions may raise them and the raise enters the importer's capture.

### The caught dictionary

Within `capture error :`, the name `error` binds a dictionary (read via `error["code"]` etc.):

- Custom errors: the declared field values, plus `line` and `column` of the raise.
- Runtime errors: the same `line`/`column`, with **`code = none`**; most also carry a `message` string (`error["message"]`).

`error` is an ordinary expression identifier inside the capture block only.

### Message catalog

Runtime errors (from the lexer/parser/interpreter/stdlib), each `Error at line:column:`), commonly include: division/modulo by zero (`Division by zero`), integer overflow (`Integer overflow`), type mismatches (`Expected int, found str`, `Cannot change constant`, `Cannot order given types`), identifier resolution (`'x' is not declared`), call/param errors (arity, `Missing argument for`), recursion (`Recursion limit exceeded`), indentation and block errors (`Expected indented block`), module failures (`missing module`, `Circular import`, `In module` context), `ask` failures (`Invalid input '…': expected …`, `No more input available`), `File.*` failures (`Could not read/write/append/delete file '…'`), `System.exit` range check (`System.exit code out of range`), and colons/wording errors from parsing (`Expected ':' after 'then'`).

### System.exit

`System.exit(int)` is an *uncapturable* termination: `capture error` never sees it, but `finally` blocks on the running path do run first, and pending output flushes before the process ends with that code. Unconditional `System.exit(...)` statements make following code unreachable (`Warning at line:column: Unreachable code`, like `return`/`stop`).

### The warning catalog

Warnings trace confirmed nonfatal situations: missing/out-of-range collection reads (then `none`), removing an absent member/key, adding a duplicate set member, adding a duplicate dictionary key (first value kept), deleting a missing file (no change), unreachable code after unconditional `return`/`stop`/`raise`/`System.exit` in the same block, and duplicate raise-override fields. Warning locations point at the statement/fragment responsible, and warnings never enter captures.

## 11. Input and output

- `print(...)` takes exactly one argument (any type; `print(none)` prints `none`) and appends a newline. Output leaves the program in program order; uncaught errors discard pending output, `System.exit` flushes first (§10).
- `ask "question"` produces one line of console input for the consuming binding (§12), with the prompt printed before reading. Prompt must be a string.

## 12. Types and conversion by `ask`

The declared type of the receiving binding drives conversion:

| Receives as | Valid input lines | Otherwise |
|---|---|---|
| `str` (or untyped) | exact line as typed, trimmed of trailing newline only | everything works |
| `int` | optional sign + digits (surrounding spaces tolerated) | `Invalid input '…': expected int` |
| `dec` | decimals or integers, signed | `Invalid input '…': expected dec` |
| `bool` | exactly `true` / `false` (case-sensitive, no spaces) | `Invalid input '…': expected bool` |

No retries, no silent `none`: failure is an ordinary (capturable) runtime error, and exhausted input is `No more input available`. Collection-typed bindings don't convert — the asking happens and the mismatch surfaces as a normal type error. `ask` composes as a primary expression (`print(ask "Who?")`).

## 13. Standard library built-ins

Module functions use strict argument types and reject `none` where real values are needed. Full per-function documentation: [stdlib.md](stdlib.md). Summary:

- **Math** — `sqrt(x)` dec (non-negative; negative is an error), `pow(base, exp)` dec (integer exponent; negative exponent divides, so `pow(0, -1)` is a division-by-zero error; `0^0` = `1`), `abs(x)` (type preserved), `round(x)` int (half away from zero), `floor(x)`/`ceil(x)` int (mathematical), `min(...)`/`max(...)` two or more numbers (all-`int` stays `int`, mixtures go `dec`).
- **Text** — `upper(s)`, `lower(s)`, `trim(s)` (edge whitespace), `replace(s, old, new)` (all occurrences; empty `old` follows platform string semantics), `split(s, sep)` → `list(str)` (empty separator follows platform string semantics), `join(list(str), sep)` → `str`.
- **File** — `read(path)` → str content (missing/unreadable → error), `write(path, content)` (overwrites), `append(path, content)` (create-or-append), `exists(path)` → bool (files *or* directories), `delete(path)` (existing file removed; missing → warning, no change; directories → error). Relative paths resolve against the process working directory. Write/append/delete return `none`.
- **System** — `os` and `arch` are property reads (strings; calling them as `System.os()` is an error), `args()` → `list(str)` full argv (`luca`, program, then extra words), `env()` → dict of environment variables sorted by name, `exit(int)` → terminates (i32 range check) as described in §10.

## 14. Command-line behavior

`luca` runs programs; `luca check` validates without executing; `--help`/`--version` print usage and the package version. Exit codes: 0 success, 1 program/validation errors, 2 usage/internal failures. Complete details: [cli.md](cli.md).
