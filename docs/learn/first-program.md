# Your first Luca program

You need the `luca` command first: [install.md](install.md). This takes about two minutes.

## 1. Create the file

Luca files end in `.lucc` — the CLI refuses anything else. Create `hello.lucc`:

```luca
print("Hello, Luca!")
```

## 2. Run it

```sh
luca hello.lucc
```

You should see exactly:

```
Hello, Luca!
```

## 3. Make it yours

Try variables and interpolation (`{name}` inside a string inserts the value):

```luca
def const name (str) = "Luca"
def var visits (int) = 1
visits (int) = visits + 1
print("Hello, {name}! Visit {visits}")
```

```
Hello, Luca! Visit 2
```

Notes for your first hour:

- `print(...)` takes exactly **one** argument. Combine values with interpolation: `print("a and {b}")`.
- Luca programs run top to bottom; there is no `main()` to declare.
- Lines starting with `--` are comments.
- Comparisons are worded: `if visits is > to 1 then :` (the `:` starts an indented block).
- Every `{name}` inside a string must be a declared variable — literal braces are not allowed.

## 4. Explore

- Continue with the [guided tour](tour/index.md) — short chapters, each with runnable code.
- `examples/` has a runnable program per feature: `luca examples/errors.lucc`. See the [examples gallery](examples-gallery.md).
- The language rules: [the language reference](../reference/language-reference.md).
- The VS Code extension: [vscode.md](vscode.md).

## When something goes wrong

Luca errors look like this:

```
Error at 2:5: 'visits' is not declared
```

Read it as: **what** (`'visits' is not declared`), **where** (line 2, column 5). Fix the spot, re-run. More help: [troubleshooting.md](troubleshooting.md).
