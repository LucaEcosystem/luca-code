# 6. Conditionals

A conditional runs a block only when a condition is `true`. The colon `:` opens an indented block; `then` ends the condition:

```luca
def const name (str) = "Luca"
if name is == to "Luca" then :
    print("Hello, {name}!")
else :
    print("Hello.")
```

```text
Hello, Luca!
```

Chain alternatives with `else if`:

```luca
def var hour (int) = 15
if hour is < to 12 then :
    print("Morning")
else if hour is < to 18 then :
    print("Afternoon")
else :
    print("Evening")
```

```text
Afternoon
```

Blocks indent by four spaces (or tabs) — one level per block, used consistently. In the example above, the two `print` calls belong to their respective branches; the second `print` after the block below would run unconditionally.

```luca
def var age (int) = 16
if age is >= to 16 then :
    print("Old enough")
print("Checked")     -- outside the block: always runs
```

```text
Old enough
Checked
```

## Conditions must be booleans

There is no truthiness: numbers, strings, collections, and `none` are not conditions. `if count then :` when `count` is a number fails with `Error at 2:1: Condition must be bool; found 2`. Compare or combine explicitly:

```luca
def var count (int) = 2
if count is > to 0 and count is < to 10 then :
    print("in range")
if not (count is == to 3) then :
    print("not three")
```

```text
in range
not three
```

Anything except an empty block may live inside the branches — declarations, loops, more conditionals (nesting indents one more level), even `def func`.

Next: [loops](07-loops.md).
