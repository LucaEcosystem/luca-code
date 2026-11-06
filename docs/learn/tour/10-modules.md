# 10. Modules

Code lives in files; modules are how files share what they declare. `import Name` reads the file `Name.lucc` — same folder first, then the folder you run from — and gives you everything it declared at top level.

Make two files in the same folder.

`Helpers.lucc`:

```luca
def const company (str) = "Luca"
def func hello(name (str)) (str):
    return "hi {name}"
```

`main.lucc`:

```luca
import Helpers
print(Helpers.company + " says it.")
print(Helpers.hello("you"))
```

```text
Luca says it.
hi you
```

What the example shows: the file `Helpers.lucc` becomes the module `Helpers` — the import name and the file name match exactly — and every top-level `def` inside it is automatically visible through the module's name, no export keyword needed. The module is also all you can see from outside: `main.lucc`'s own variables stay home.

Module behavior, briefly:

- **Own scope.** A module cannot read your variables; you cannot reach its locals if they are not top-level definitions.
- **Runs once.** The first import executes the module's top-level statements; each later import of the same name (even `import Helpers` inside another imported file) reuses that result. If the top level fails, the error surfaces at the importer.
- **`Module.member` reads values; `Module.func(args)` calls functions.** Prints of the module itself are not supported.
- **Circular imports are errors** (`Circular import 'C'`), as are missing files (`missing module 'Nope'`) and unknown members (`Module 'X' has no member 'y'`).

## The standard library arrives as modules

`Math`, `Text`, `File`, and `System` are built-in modules — import them exactly like your own, and they win if you ever have a file with the same name:

```luca
import Math
print(Math.sqrt(25))
```

```text
5.0
```

Forgetting the import is an error: `Error at 1:11: Module 'Text' is not imported`. Chapter 12 tells the whole story of `ask` and the library; the [stdlib reference](../../reference/stdlib.md) lists every function.

Next: [errors](11-errors.md).
