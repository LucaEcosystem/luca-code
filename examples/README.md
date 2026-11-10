# Luca Code examples

One runnable program per language area. From the repository root, after
installing `luca` ([install guide](../docs/learn/install.md)):

```sh
luca examples/hello.lucc
```

| File | Demonstrates |
|---|---|
| `hello.lucc` | `print`, interpolation, reassignment |
| `arithmetic.lucc` | Arithmetic, exact decimals, precedence |
| `inference.lucc` | Type inference, `none`, `clear()` |
| `comparison.lucc` | Worded comparisons across types |
| `conditional.lucc` | `if` / `else if` / `else`, nesting, logic |
| `functions.lucc` | Parameters, defaults, returns, recursion |
| `loops.lucc` | All three loop forms, `stop` / `skip` |
| `collections.lucc` | Lists, tuples, sets, dictionaries |
| `modules/main.lucc` | Importing a sibling module |
| `errors.lucc` | `try` / `capture` / `finally`, custom errors, `raise` |
| `ask.lucc` | Typed console input (needs stdin) |
| `stdlib.lucc` | `Math`, `Text`, `File`, `System` |
| `text.lucc` | Every `Text` helper and empty-separator edges |
| `math.lucc` | Every `Math` helper and exact division |
| `files.lucc` | Write, append, read, check, delete (cleans up) |
| `utility.lucc` | Reusable helpers and a `uni` value holding a list |
| `integration.lucc` | Several features together |

## Programs that need a little setup

- `ask.lucc` reads two lines:

  ```sh
  printf 'Luca\n16\n' | luca examples/ask.lucc
  ```

- `stdlib.lucc` expects to run from the repository root, because its `File`
  demo path is relative. It intentionally probes a missing file first, so its
  warning is part of the expected output.

- `modules/main.lucc` imports `examples/modules/Helpers.lucc`; run the file
  itself, not the folder:

  ```sh
  luca examples/modules/main.lucc
  ```

- `files.lucc` creates `luca-example.tmp` in the current directory and deletes
  it before exiting.

Expected output for every example is in the
[examples gallery](../docs/learn/examples-gallery.md). The test suite runs the
deterministic examples on every CI build, so a changed output is a change in
behavior.
