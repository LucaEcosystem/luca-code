# Standard library reference (1.0 Beta, package 1.0.3 Beta)

Four built-in modules — `Math`, `Text`, `File`, and `System` — ship with the interpreter. Import them like any module (`import Math`); a same-named `.lucc` file you happen to have is ignored in favor of the built-in.

Shared rules for all functions listed here:

- **Strict types.** Arguments are checked before the call; an `int`-expecting function rejects `7.0` and `"7"`.
- **No implicit conversion** and **no `none`** — passing `none` or a wrong type is an error (`Cannot use 'none' as an argument to 'M.f'`).
- **Arity is exact.** Calling with the wrong count is an error (`Expected 2 arguments, found 3` style for fixed-arity functions; `Expected at least 2 arguments` for `min`/`max`).
- **Errors are ordinary runtime errors** — a failed documented condition stops the run unless caught with `try to` ([errors chapter](../learn/tour/11-errors.md)).

## Math — exact numeric helpers

| Function | Signature | Returns | Behavior |
|---|---|---|---|
| `Math.sqrt(x)` | `x` int/dec ≥ 0 | `dec` | Exact truncated square root, up to 10 decimal places, trailing zeros trimmed. Negative input → error (`Math.sqrt of negative number`). |
| `Math.pow(base, exp)` | `base` int/dec, `exp` int | `dec` | Exact exponentiation by squaring. `exp` must be `int` (`Math.pow exponent must be int`). `pow(x, 0)` is `1`. Negative exponents divide, so `pow(0, -1)` → error (`Division by zero`). |
| `Math.abs(x)` | `x` int/dec | same type as input | Absolute value, preserving `int`/`dec`. |
| `Math.round(x)` | `x` int/dec | `int` | Half away from zero. |
| `Math.floor(x)` | `x` int/dec | `int` | Round toward negative infinity (mathematical). |
| `Math.ceil(x)` | `x` int/dec | `int` | Round toward positive infinity (mathematical). |
| `Math.min(a, b, …)` | 2+ int/dec | winner's type | Smallest; all-`int` → `int`, any `dec` → `dec`. |
| `Math.max(a, b, …)` | 2+ int/dec | winner's type | Largest; same mixing rule. |

Verified console session:

```luca
import Math
print(Math.sqrt(25))
print(Math.pow(2, 10))
print(Math.round(2.5))
print(Math.abs(-3))
print(Math.max(1, 2.5))
```

```text
5.0
1024.0
3
3
2.5
```

## Text — string helpers

| Function | Signature | Returns | Behavior |
|---|---|---|---|
| `Text.upper(s)` | `s` str | `str` | Uppercased copy. |
| `Text.lower(s)` | `s` str | `str` | Lowercased copy. |
| `Text.trim(s)` | `s` str | `str` | Copy with edge whitespace removed. |
| `Text.replace(s, old, new)` | all str | `str` | All occurrences of `old` replaced with `new`. An empty `old` follows the host platform's string semantics. |
| `Text.split(s, sep)` | both str | `list(str)` | Split on every occurrence of `sep`. An empty `sep` follows the host platform's string semantics. |
| `Text.join(list, sep)` | `list` list(str), `sep` str | `str` | Elements joined with `sep` between them. |

Verified console session (`Text.split` output shows collection quoting):

```luca
import Text
print(Text.upper("shout"))
print(Text.lower("CALM"))
print(Text.trim("  hi  ") + "|")
print(Text.replace("old new old", "old", "x"))
print(Text.split("a-b-c", "-"))
print(Text.join(["x", "y"], "+"))
```

```text
SHOUT
calm
hi|
x new x
["a", "b", "c"]
x+y
```

## File — files on disk

| Function | Signature | Returns | Behavior |
|---|---|---|---|
| `File.read(path)` | `path` str | `str` | Whole file contents; missing or unreadable → error (`Could not read file '…'`). |
| `File.write(path, content)` | both str | `none` | Creates or **overwrites**. Failure → error (`Could not write file '…'`). |
| `File.append(path, content)` | both str | `none` | Creates or appends. Failure → error. |
| `File.exists(path)` | `path` str | `bool` | True for existing files **and** directories. |
| `File.delete(path)` | `path` str | `none` | Deletes an existing file; deleting a **missing** path warns (`File '…' does not exist`) with no change; deleting a directory → error. |

Paths are strings; relative paths resolve against the process working directory, and absolute paths are allowed. All failures except missing-path deletion are errors, so they pair naturally with `try to` in the [tour chapter](../learn/tour/11-errors.md).

## System — the host world

| Member | Kind | Returns | Behavior |
|---|---|---|---|
| `System.os` | property | `str` | Platform label (e.g. `macos`, `linux`, `windows`). Property form only — `System.os()` is an error. |
| `System.arch` | property | `str` | Architecture label (e.g. `aarch64`, `x86_64`). |
| `System.args()` | function | `list(str)` | The **full** command line: the `luca` program, your script's path, then every extra word (`luca script.lucc one two` → `[…, script.lucc, one, two]`). |
| `System.env()` | function | `dict` | Environment variables, dictionary keyed by name, sorted by name at call time. |
| `System.exit(code)` | function | — | Terminate the run with `code` (32-bit range; outside → `System.exit code out of range`). Pending output flushes; `finally` blocks run; `capture error` never intercepts it (see the [language reference](language-reference.md)). |

Values for `os`/`arch`/environment depend on your machine; sample on one host: `os=macos arch=aarch64`.
