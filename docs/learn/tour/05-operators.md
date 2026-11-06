# 5. Operators and comparisons

## Arithmetic

| Operator | Meaning | Example | Result |
|---|---|---|---|
| `+` | add (two numbers) or join (two strings) | `4 + 3` | `7` |
| `-` | subtract | `10 - 4` | `6` |
| `*` | multiply | `2 * 3` | `6` |
| `/` | numeric division | `10 / 4` | `2.5` |
| `%` | remainder | `10 % 3` | `1` |
| `-` (unary) | negate | `3 - -2` — note the spaces | `5` |

Division is always numeric (see chapter 2). Division or remainder by zero is an error, as is integer overflow.

## Precedence

Highest first: `not`, then `* / %`, then `+ -`, then comparisons, then `and`, then `or`. Parentheses override everything:

```luca
print(2 + 3 * 4)
print((2 + 3) * 4)
print(10 / 2 - 1)
```

```text
14
20
4.0
```

Unary minus needs a space on its left: `3 - -2` is `5`. Written without one, `3--2`, the `--` starts a comment and swallows the rest of the line (`Error at 1:12: print accepts exactly one argument`).

## Worded comparisons

Comparisons read as words: `is == to`, `is != to`, `is > to`, `is >= to`, `is < to`, `is <= to`.

```luca
print(5 is == to 5)
print(3 is < to 9)
print(3.0 is >= to 3)
print("abc" is < to "abd")
print(16 is != to 20)
```

```text
true
true
true
true
true
```

Equality (`is == to`, `is != to`) also works for booleans and, later, collections — and with `none`: `x is == to none`. Ordering (`is > to` and friends) works for numbers and for strings (lexicographic, case-sensitive); ordering anything else is an error. Comparing values of incompatible types is an error rather than `false`.

## Logic

`and`, `or`, and `not` work on booleans only, and `and`/`or` short-circuit (the right side is skipped when the left decides the outcome):

```luca
print(true and true)
print(false or true)
print(not false)
```

```text
true
true
true
```

Next: [conditionals](06-conditionals.md).
