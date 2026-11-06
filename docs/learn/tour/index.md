# The Luca tour

A guided tour of Luca Code, one small chapter at a time. Every snippet is a complete, runnable program — type it into a `.lucc` file, run `luca that-file.lucc`, and compare what you see with the shown output. Nothing here assumes prior experience with any particular language; if you have programmed before, skim the early chapters.

Before starting, install the `luca` command: [install.md](../install.md). Verify with `luca --version` (prints `luca 1.0.3 Beta`).

## How to follow along

1. Create a folder for practice and open a terminal there.
2. Create a file ending in `.lucc` (for example `tour.lucc`) and paste in a snippet.
3. Run it:

```sh
luca tour.lucc
```

Luca runs programs top to bottom, prints output as it goes, and stops with a plain-language message if something is wrong — nothing to configure.

## Chapters

| # | Chapter | You learn |
|---|---|---|
| 1 | [Hello, Luca](01-hello.md) | Running your first program |
| 2 | [Values and types](02-values-and-types.md) | Numbers (`int`, exact `dec`), text, booleans, `none` |
| 3 | [Variables](03-variables.md) | `def var`/`def const`, reassigning, `change`, `forget`, `clear()` |
| 4 | [Strings](04-strings.md) | Interpolation, joining, comments, escapes |
| 5 | [Operators](05-operators.md) | Arithmetic precedence, worded comparisons, logic |
| 6 | [Conditionals](06-conditionals.md) | `if` / `else if` / `else` with indented blocks |
| 7 | [Loops](07-loops.md) | `for … times`, `while`, `for … from`, `stop`/`skip` |
| 8 | [Functions](08-functions.md) | `def func`, parameters, defaults, `return`, closures |
| 9 | [Collections](09-collections.md) | Lists, tuples, sets, dictionaries, 1-based indexing |
| 10 | [Modules](10-modules.md) | Sharing code between files, the standard library |
| 11 | [Errors](11-errors.md) | `try`/`capture`/`finally`, custom errors with `raise` |
| 12 | [Input and the library](12-input-and-library.md) | `ask`, command-line arguments, `Math`/`Text`/`File`/`System` |

After the tour, keep [language-reference](../../reference/language-reference.md), [stdlib](../../reference/stdlib.md), and [CLI](../../reference/cli.md) within reach, and browse the runnable [examples](../examples-gallery.md).

Conventions used in this tour: console blocks show what the terminal prints; inline code like `def` marks Luca keywords; little asides about what an error looks like quote the real message you would see.
