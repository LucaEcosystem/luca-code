# Examples gallery

The `examples/` folder in the repository holds one runnable program per language area. Each subsection below shows what the program demonstrates and what it prints, verified against `luca` 1.0.3 Beta. Run any of them after installing, from the repository root:

```sh
luca examples/hello.lucc
```

| File | Teaches |
|---|---|
| `examples/hello.lucc` | First program: `print`, interpolation, reassignment |
| `examples/arithmetic.lucc` | Numeric arithmetic, exact decimals, precedence |
| `examples/inference.lucc` | Type inference, `none`, `clear()` |
| `examples/comparison.lucc` | Worded comparisons across types |
| `examples/conditional.lucc` | `if` / `else if` / `else`, nesting, logic |
| `examples/functions.lucc` | Parameters, defaults, return values, recursion |
| `examples/loops.lucc` | All three loop forms, `stop`/`skip` |
| `examples/collections.lucc` | Lists, tuples, sets, dictionaries |
| `examples/modules/main.lucc` | Importing a second file |
| `examples/errors.lucc` | `try`/`capture`/`finally`, custom errors, `raise` |
| `examples/ask.lucc` | Typed input with `ask` |
| `examples/stdlib.lucc` | `Math`, `Text`, `File`, `System` |
| `examples/text.lucc` | Every `Text` helper, including empty-separator edge cases |
| `examples/math.lucc` | Every `Math` helper and exact division |
| `examples/files.lucc` | Writing, appending, reading, checking, and deleting a file |
| `examples/utility.lucc` | Reusable helpers plus a `uni` variable holding a collection |
| `examples/integration.lucc` | Everything combined |

## `hello.lucc`

```sh
luca examples/hello.lucc
```

```text
Hello, Luca!
Visit 2
Six divided by two is:
3.0
```

## `arithmetic.lucc`

```text
2.5
14
40
6
2
21.50
5.00
0.3
```

## `inference.lucc`

```text
Hello, Luca! Visit 2
3.14
none
5
```

## `comparison.lucc` — nine comparison checks

```text
true
true
true
true
true
false
true
true
true
```

## `conditional.lucc`

```text
Hello, Luca!
B
nested
after inner
and true
not false
inner
outer
```

## `functions.lucc`

```text
Hello, Luca!
Hello, Alex!
5
30
120
2
15
5
hi
```

## `loops.lucc`

```text
5
10
hi
hi
hi
3
13
nested
nested
nested
nested
5
4
3
done
3
2
1
```

## `collections.lucc` — the four kinds in action

```text
["Luca", "Alex"]
Luca
2
true
["Luca", "Alex", "Sam"]
["Alex", "Sam"]
(123, true, "Luca")
Luca
(456, true, "Luca")
{"alpha", "stable"}
true
{"alpha", "stable", "beta"}
{"Name" = "Luca", "Age" = 16}
Luca
true
true
{"Name" = "Luca", "Age" = 16, "City" = "Berlin"}
{"Name" = "Luca", "Age" = 17, "City" = "Berlin"}
Alex
Sam
true
2
2
```

## `modules/main.lucc` — run the file, not the folder

```sh
luca examples/modules/main.lucc
```

```text
hi World
hi Luca
1.0
demo
true
1
2
demo
modules
```

## `errors.lucc`

```text
caught division
cleanup
no trouble
always
inner finally
outer caught
5.0
0
503
unavailable
41
500
boom
```

## `ask.lucc` — feed it lines

`ask` needs input; pipe it in when not typing interactively:

```sh
printf 'Luca\n16\n' | luca examples/ask.lucc
```

```text
What is your name?
How old are you?
Hello, Luca. You are 16 years old.
```

## `stdlib.lucc`

The program deliberately probes a missing file first — its warning is part of the demo, printed with the location of the offending statement:

```text
Warning at 14:1: File 'examples/.no-such-demo-file' does not exist
5.0
1024.0
3
1
HELLO
a-b-c
true
true
true
true
```

## `text.lucc`

```text
Luca Code
LUCA CODE
luca code
Luca-Code
["list", "tuple", "set", "dict"]
list / tuple / set
["", "a", "b", "c", ""]
.a.b.c.
```

## `math.lucc`

```text
5.0
1024.0
7
2.50
3
-3
-3
-2
1.50
4
2.5
0.3333333333333333333333333333
0.25
```

## `files.lucc`

Creates `luca-example.tmp`, reads it, then deletes it — running it leaves no file behind:

```text
true
first and second
false
```

## `utility.lucc`

```text
32.0
212.0
0
42
100
3
["alpha", "beta", true, 3.50]
now a scalar
```

## `integration.lucc`

```text
skipped oops
35.50
skipped oops
1 over 10.00
{"retries" = 5}
true
no crash
done
```

Continue with the [guided tour](tour/index.md), the [language reference](../reference/language-reference.md), or the [stdlib reference](../reference/stdlib.md) for function-level details.
