# 1. Hello, Luca

Luca Code programs are plain text files ending in `.lucc`. Create one:

```luca
print("Hello, Luca!")
```

Say the file is named `hello.lucc` and you are in its folder:

```sh
luca hello.lucc
```

You see exactly:

```text
Hello, Luca!
```

That is a whole program. Two things worth noticing right away:

- There is no `main()` and no startup ceremony. Luca runs your file top to bottom, statement by statement, and prints as it goes.
- `print(...)` takes exactly **one** argument. To include several values, put them in one string with `{name}` placeholders — that is the next chapters' subject.

To try more than one statement, make a slightly bigger file. Whitespace between statements does not matter, and lines starting with `--` are comments:

```luca
-- A three-line program.
print("First")
print("Second")

print("Third")
```

```text
First
Second
Third
```

Next: [values and types](02-values-and-types.md).
