# 3. Variables

A variable holds a value under a name. Declare it with `def var` (changeable) or `def const` (never changes). A type in parentheses is optional when Luca can tell the type from the value you assign:

```luca
def const name (str) = "Luca"
def var visits (int) = 1
def var mood = "calm"          -- inferred: str
print("{name} visited {visits} times, mood is {mood}")
```

```text
Luca visited 1 times, mood is calm
```

Reassign an existing variable by naming it with its type again — same name, same type:

```luca
def var visits (int) = 1
visits (int) = visits + 1
print(visits)
```

```text
2
```

## `change` — switch a variable's type

Normally a variable keeps its declared type forever. `change` is the one (explicit) way to give a variable a new type and value:

```luca
def var badge = "ready"
change badge (bool) = false
print(badge)
```

```text
false
```

The badge variable no longer holds text — it now holds a boolean. `change` is always written out in full, and it is the only way to switch types.

## `clear()` — set a variable to `none`

Calling `.clear()` on a mutable variable sets it to `none` while keeping its name and type; you can assign a new value of that type afterwards. Clearing a `const` is an error.

```luca
def var score (int) = 50
score.clear()
print(score)
score (int) = 12
print(score)
```

```text
none
12
```

## `forget` — remove a variable, then optionally re-declare

`forget` removes a binding entirely. The name can then be declared again — even with a different type.

```luca
def var mode = "fast"
forget var mode (str)
def var mode (bool) = true
print(mode)
```

```text
true
```

## The rules in one place

- One binding per name per scope; re-declaring an existing name is an error (`Error at 3:21: 'visits' is already declared`).
- `const` cannot be reassigned, cleared, or mutated (`Cannot change constant`).
- Assigning a value that does not match the declared type is an error: `Error at 1:1: Expected int, found str` for `def var age (int) = "hi"`.
- A variable of any declared type may hold `none`. Working with that `none` in arithmetic or logic is an error; print it or compare it instead.
- `uni` accepts any value: `def var anything (uni) = 1`, then later `anything (uni) = "text"` is fine.

Next: [strings](04-strings.md).
