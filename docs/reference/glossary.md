# Glossary

Plain-language definitions of the terms used across the Luca Code documentation. Cross-references point into the [language reference](language-reference.md).

| Term | Meaning |
|---|---|
| **Luca Code** | The general-purpose programming language documented here. Source files end in `.lucc`. (A sibling language, Luca UI, uses `.lucu` and is planned separately.) |
| **Beta / 1.0 Beta** | The language's *product identity* for this generation: the complete feature set, undergoing validation and stabilization. See [CHANGELOG](../../CHANGELOG.md). |
| **Package version** | The machine-facing version (currently **1.0.3**): `luca --version` prints `luca 1.0.3 Beta`. Marketing text says "1.0 Beta". |
| **Interpreter** | The reference implementation: reads your program, checks it, and executes it directly. Written in Rust. The 1.0 line is interpreter-only; no native compiler is planned for it. |
| **`.lucc`** | A Luca Code source file. The CLI refuses any other extension. |
| **`int` / `dec`** | Whole numbers / exact base-10 decimals. `dec` keeps the digits you wrote — `0.1 + 0.2` is exactly `0.3`, and `10 / 4` is `2.5`. |
| **`str`** | Text in double quotes, single line, with `\n`-style escapes. |
| **`bool`** | `true` or `false` — the only valid types for conditions; there is no truthiness. |
| **`uni`** | The universal type: accepts any value (`int`, `str`, bool, or an entire collection). Useful for heterogeneous slots and for variables that switch between scalar and collection values. |
| **`none`** | The absence value. Prints as `none`, occupies any declared type, and rejects value-requiring operations. |
| **Binding** | A name attached to a value in a scope: `def var` (changeable) or `def const` (frozen). |
| **Inference** | Omitting the `(type)` when the assigned value makes it obvious: `def var mood = "calm"` is a `str`. |
| **`change`** | The one keyword that switches a variable's stored type: `change badge (bool) = false`. There are no casting functions. |
| **`forget`** | Removes a binding so the name can be re-declared. |
| **`clear()`** | Sets a mutable variable to `none`, keeping its name and type; empties a collection to its zero form (`[]`, `()`, `{}`). |
| **Interpolation** | `{name}` inside a string substitutes a declared variable's rendered value, wherever strings are built. |
| **Escape** | A backslash sequence in a string: `\n` newline, `\r`, `\t`, `\"`, `\\`. |
| **Comment** | `--` to end of line; `[[ … ]]` for multiline. Comment-only lines do not affect indentation, so they may sit at any depth. `[[` always opens a block, so `[ [1]]` needs the space for nested lists. |
| **Block** | The statements indented under a `:` — used by `if`/`else`, loops, `def func`, `try`/`capture`/`finally`, and `def error`. |
| **Worded comparisons** | Luca's comparison forms: `is == to`, `is != to`, `is > to`, `is >= to`, `is < to`, `is <= to`. |
| **Truthiness** | Auto-converting conditions to booleans. Luca has none: conditions must be real `bool`s. |
| **Short-circuit** | `and`/`or` skip the right side when the left side already decides the outcome. |
| **Loop item** | The fresh binding created by `for item from collection repeat :`, available inside the block. |
| **Closure** | A function that reads (and mutates) variables from the scope where it was declared. |
| **Recursion** | A function calling itself. Nesting is capped at 64; deeper calls error (`Recursion limit exceeded`). |
| **1-based indexing** | Collections start at index `1`: `names[1]` is the first element. |
| **Collection kinds** | `list` `[a, b]`, `tuple` `(a, b)`, `set` `{a, b}`, `dict` `{"k" = v}` — the last keyed by strings. |
| **Sharing** | Assignments, parameters, and closures alias the *same* collection storage; mutating through one alias changes what all aliases see. `const` bindings deep-copy on read, so their contents stay frozen. |
| **Module** | Another `.lucc` file brought in with `import Name`; its file name matches the module name, and every top-level `def` inside it is automatically exported through `Name.member` access. |
| **Built-in modules** | `Math`, `Text`, `File`, `System` — provided by the interpreter itself; an import of these names always binds them (see [stdlib](stdlib.md)). |
| **Error** | A fatal condition: prints `Error at line:column: message` and stops the run (discarding buffered print output). Capturable only by `try to`'s `capture error`. |
| **Warning** | A nonfatal notice: prints `Warning at line:column: message` and execution continues. Never captured; never fatal. |
| **`try to` / `capture` / `finally`** | Run the risky block; on failure of runtime errors only, run the capture block with the caught dictionary bound to `error`; `finally` runs last in every case — success, capture, and propagation. |
| **Caught dictionary** | What `error` holds inside `capture error :`: the custom error's declared fields plus `line`/`column`, or `code = none` (+ usually `message`) for runtime errors. |
| **`def error` / `raise`** | Declaring a custom error (fields include at least `code` and `message`) and actively raising it with field overrides: `raise Validation(code = "EMPTY_NAME")`. |
| **`ask`** | Reads one input line for the receiving binding, converting per that binding's declared type; invalid typed input is an error (no retry). |
| **Diagnostics** | The formatted error/warning lines: `Error at line:column: message` / `Warning at line:column: message`. |
| **`luca check`** | CLI subcommand validating a program and its whole import graph without executing it; reports the same kinds of diagnostics ([details](cli.md)). |
| **Exit code** | What the process reports to the OS: `0` success; `1` when a run fails or `luca check` finds validation errors; `2` usage/internal CLI failure — or the code your program passed to `System.exit(code)`. |
| **`System.exit`** | The standard way to terminate with a code. Uncapturable by `capture error`, though `finally` still runs. |

Missing a term you expected? The [language reference](language-reference.md) has the complete rule set, and [contacting the project](https://github.com/LucaEcosystem/luca-code) is welcome for what remains unclear.
