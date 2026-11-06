# 4. Strings

Text is written between double quotes:

```luca
print("Luca Code is a programming language.")
```

```text
Luca Code is a programming language.
```

## Interpolation — put values inside text

Inside a string, `{name}` inserts the *declared variable* `name`. Interpolation renders the value for printing, so you never convert manually:

```luca
def const name (str) = "Luca"
def const age (int) = 16
print("Hello, {name}. You are {age} years old.")
```

```text
Hello, Luca. You are 16 years old.
```

Every `{...}` must name a variable and only a variable — plain braces (`{}`), math inside braces, or an unknown name are all errors:

- `print("brace {} here")` → `Error at 1:7: Interpolation supports variable names only in this milestone`
- `print("Hi {ghost}")` (never declared) → `Error at 1:7: 'ghost' is not declared`

## Joining strings

`+` accepts exactly two strings — no mixing text with numbers:

```luca
print("fresh " + "water")
print("version " + "1")
```

```text
fresh water
version 1
```

`print("a" + 1)` is an error: `Error at 1:11: Arithmetic requires matching numeric types; '+' also accepts two str values`. Use interpolation instead.

## Escapes

Escapes start with a backslash: `\n` newline, `\r` carriage return, `\t` tab, `\"` double quote, `\\` a literal backslash. Any other escaped character passes through as itself.

```luca
print("first\nsecond")
print("quote:\" inner")
```

```text
first
second
quote:" inner
```

A string literal cannot span lines — `Error at 1:19: String literal cannot contain a raw newline`; write `\n` instead.

## Comments

```luca
-- This whole line is a comment.
print("seen")       -- This end-of-line part is too.

[[
   Everything here is a multiline comment.
   print("not executed")
]]
print("also seen")
```

```text
seen
also seen
```

Beware: `[[` always starts a multiline comment, which matters in step 9 when writing lists of lists (a leading space `[ [ …` is the habit to keep).

Next: [operators](05-operators.md).
