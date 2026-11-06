# 2. Values and types

Luca has five everyday types plus one special value:

| Type | Holds | Examples |
|---|---|---|
| `int` | whole numbers | `7`, `-2`, `0` |
| `dec` | decimal numbers, kept **exact** | `2.5`, `0.10`, `-1.75` |
| `str` | text in double quotes | `"Luca"`, `"Age 16"` |
| `bool` | `true` or `false` | `true`, `false` |
| `uni` | any of the above | `1`, `"any"`, `true` |
| `none` | "no value" — a special value, not a type | `none` |

You can print any value directly:

```luca
print(10 / 4)
print(20.00 + 1.50)
print(true)
print(none)
```

```text
2.5
21.50
true
none
```

## Decimals are exact

This is Luca's headline numeric feature: `10 / 4` is `2.5`, not the truncated `2` that many languages produce for whole-number types. Decimal arithmetic keeps the digits you wrote — `20.00 + 1.50` prints `21.50`, keeping two decimal places — and `0.1 + 0.2` really is `0.3`, with no hidden floating-point fuzz.

```luca
print(10 / 4)
print(0.1 + 0.2)
print(2 - 0.5)
```

```text
2.5
0.3
1.5
```

When `int` and `dec` values are mixed by arithmetic, the result is `dec`. Division is numeric division, so even whole results come back as `dec`: `6 / 2` prints `3.0`, while `6 % 2` (remainder) is `0`.

Two boundaries to remember:

- Division or modulo **by zero** is an error: `print(1 / 0)` fails with `Error at 1:9: Division by zero`.
- Integer results that exceed the 64-bit limit are an error too: `Error at 1:27: Integer overflow`.

## `none` means "nothing here"

`none` is a value that means no value. Printing it prints `none`; comparing it is fine (`value is == to none`); but doing arithmetic or logic with it is an error. You will meet it again in variables (a cleared variable becomes `none`) and collections (a missing entry reads as `none`).

Text (`str`) and booleans get their own chapters next, briefly in [strings](04-strings.md) and [operators](05-operators.md).

Next: [variables](03-variables.md).
