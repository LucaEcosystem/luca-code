# 7. Loops

Luca has three loop shapes. All of them open an indented block with `repeat :`.

## A fixed number of times

```luca
for 3 times repeat :
    print("hi")
```

```text
hi
hi
hi
```

The count is any `int` expression, and it must not be negative. It is evaluated once, before the loop starts — the loop above prints one `x`, because `3 - n` is `1` where `n` is `2`.

## While a condition holds

```luca
def var count (int) = 3
while count is > to 0 repeat :
    print(count)
    count (int) = count - 1
```

```text
3
2
1
```

Like with `if`, the condition must be a real boolean. An endless `while` runs forever — Luca does not add a hidden iteration limit, so loop logic is yours to get right.

## Over a collection

```luca
def list names (str) = ["Luca", "Alex", "Sam"]
for name from names repeat :
    print("Hi {name}")
```

```text
Hi Luca
Hi Alex
Hi Sam
```

This reads each item in turn: list, tuple, and set items come out as stored, and dictionary keys iterate (a `dict` is a set of entries, so the chapter-9 example prints its keys). You do not declare `name` — the loop binding exists inside the block.

Mutating a collection *while iterating it* is an error; build a fresh list instead if you need changes.

## Leaving early: `stop` and `skip`

`stop` exits the innermost loop; `skip` jumps straight to the next iteration. Both are exact words, not functions, and using either outside a loop is an error.

```luca
for 5 times repeat :
    if true then :
        skip
    print("never printed")
print("done")

for n from [1, 2, 3] repeat :
    if n is == to 2 then :
        stop
    print(n)
print("stopped at two")
```

```text
done
1
stopped at two
```

Note the second loop's `[1, 2, 3]`: collections can be written inline wherever a value is expected, not only in declarations.

Next: [functions](08-functions.md).
