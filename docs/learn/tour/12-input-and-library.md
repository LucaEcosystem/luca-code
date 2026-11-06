# 12. Input and the library

## `ask` — read one line from the user

`ask "question"` is an expression that produces the user's input, converted to the type of whatever will hold it:

```luca
def const name (str) = ask "What is your name?"
def var age (int) = ask "How old are you?"
print("Hello, {name}. You are {age} years old.")
```

In a terminal this shows the question, then reads the next line you type:

```text
What is your name?
Luca
How old are you?
16
Hello, Luca. You are 16 years old.
```

(The console above includes your typed answers; piping input in — `printf 'Luca\n16\n' | luca ask.lucc` — feeds the same two answers without echoing them.)

Conversion rules, straight from the receiving variable:

- `str` keeps the line exactly as typed.
- `int` parses whole numbers (signed, trimmed). `dec` accepts decimals too, including integer-looking input like `16`.
- `bool` accepts exactly `true` or `false` — case-sensitive.

Anything unparseable for the declared type is an error — `Error at 2:21: Invalid input 'yesterday': expected int` (the `ask` that reads it) — or if the input stream has ended, `No more input available`. Both are ordinary runtime errors: they stop the program unless a `try to` catches them, and no retry is attempted.

`ask` works anywhere an expression works, which also lets you compose it — `print(ask "Who?")` reads a line and passes it straight to `print`.

## The standard library at a glance

Four built-in modules cover the everyday terrain; all of them are imported like any module and enforce strict types on their arguments.

| Module | Covers | Examples |
|---|---|---|
| `Math` | exact math helpers | `Math.sqrt(25)` → `5.0`, `Math.max(1, 2.5)` → `2.5` |
| `Text` | string surgery | `Text.upper("shout")` → `"SHOUT"`, `Text.split("a-b-c", "-")` |
| `File` | files on disk | `File.read("data.txt")`, `File.exists("notes.md")` |
| `System` | the host world | `System.os`, `System.args()`, `System.exit(3)` |

```luca
import Text

def var raw (str) = ask "tag: "
def var tags (str) = Text.replace(raw, " ", "-")
print(Text.upper(tags))
```

```text
tag: live demo
LIVE-DEMO
```

(The `tag:` prompt prints first, then your typed line follows it in the console.)

Notes worth internalizing now:

- Wrong argument types or counts are errors — e.g. `Math.sqrt(-1)` refuses negative numbers, `Math.pow` needs an integer exponent, and every module function rejects `none` arguments.
- `File` functions that cannot succeed — reading a missing file, writing into a nonexistent directory — are errors and pair well with `try to`. Deleting an *already absent* file is the exception: a warning, no change.
- `System.args()` returns the full command line as `list(str)`: the `luca` program itself, your script's path, then any extra words after it. `System.env()` returns the process environment as a dictionary. `System.exit(code)` ends the run immediately — uncapturable (chapter 11), with `finally` still running; the OS sees your `code`.
- `System.os` and `System.arch` are *properties*, read without parentheses: `System.os()` with parens is an error.

Where next? The complete function tables live in the [stdlib reference](../../reference/stdlib.md), exact wordings of every message in the [language reference](../../reference/language-reference.md), and the runnable showcase programs in the [examples gallery](../examples-gallery.md).
