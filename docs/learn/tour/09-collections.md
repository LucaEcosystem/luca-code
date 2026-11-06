# 9. Collections

Luca has four collection kinds. declaring one reads almost like prose — the kind, then the element type:

```luca
def list names (str) = ["Luca", "Alex"]
def tuple data (uni) = (123, true, "Luca")
def set labels (str) = {"alpha", "stable"}
def dict person = {"Name" = "Luca", "Age" = 16}
```

Lists, tuples, and sets hold one element type (`str` above; `uni` when values differ). Dictionaries pair `string` keys with values, and each value carries its own type.

## Reading and indexing

Indices are **1-based** — the first item is `[1]`:

```luca
def list names (str) = ["Luca", "Alex"]
print(names[1])
print(names.length)
```

```text
Luca
2
```

`.length` counts items (also string characters and `int`/`dec` digit counts); `.contains(value)` answers membership. A missing read produces a **warning** and yields `none` — the program continues:

```luca
def list names (str) = ["Luca", "Alex"]
print(names[5])
print("still running")
```

```text
Warning at 2:8: Index out of range
none
still running
```

For a dictionary, index with the key: `person["Name"]`. Missing keys warn with `Key 'Phone' not found` and similarly yield `none`.

## Changing contents

`.add` and `.remove` are statements; `change name[index] = value` rewrites one slot, and the replacement must match the element type:

```luca
def list names (str) = ["Luca", "Alex"]
names.add("Sam")
print(names)
names.remove("Luca")
print(names)
change names[1] = "Jordan"
print(names)
```

```text
["Luca", "Alex", "Sam"]
["Alex", "Sam"]
["Jordan", "Sam"]
```

Dictionaries mirror this: `person.add("City", "Berlin")`, `change person["Age"] = 17`, `person.remove("Name")`, and `person.clear()` empties it. Membership has two forms: `person.contains(key = "Age")` and `person.contains(value = 16)`.

Notes that save surprises later:

- A set ignores duplicates: adding `"alpha"` again warns (`Set value already exists`) and changes nothing. Sets display in insertion order, though membership logic treats them as unordered.
- Duplicate dictionary keys at creation warn (`Duplicate dictionary key`) and keep the first value.
- Mixing a declaration like `def dict person = ...` without a value type means values may differ — look again at the `Age = 16` line: no `(str)`-style forcing is needed.
- Collections read like values but share storage: passing one to a function gives the function the same collection, so its `.add` calls are visible to you afterwards.
- `[[` starts a multiline comment, so a list whose first item is a list needs a space: write `[ [1, 2], [3] ]`, never `[[1, 2], [3] ]`.

## Looping

`for item from collection repeat :` visits each element; for dictionaries it visits keys:

```luca
def dict person = {"Name" = "Luca", "Age" = 16}
for key from person repeat :
    print(key)
```

```text
Name
Age
```

Next: [modules](10-modules.md).
