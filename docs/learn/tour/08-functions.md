# 8. Functions

A function bundles steps behind a name and can hand a value back. The block opens with `:` like the other chapters' blocks:

```luca
def func greet(name (str)) (str):
    return "Hello, {name}!"

print(greet("Luca"))
print(greet("Alex"))
```

```text
Hello, Luca!
Hello, Alex!
```

Read the signature as: *define function* `greet` taking a parameter `name` of type `str`, returning `str`. Types are optional — an untyped parameter accepts any value, and a function without a return type may return different kinds of results.

## Defaults and multiple parameters

Parameters are separated by commas; any may carry a default with `= value`, which makes it optional at the call site. Calls are positional — `name(argument)`, nothing more:

```luca
def func describe(name (str) = "you") (str):
    return "Hi {name}"

print(describe())
print(describe("Sam"))
```

```text
Hi you
Hi Sam
```

Too many arguments is `Error at 3:7: Expected 2 arguments, found 3`; too few is `Error at 3:7: Missing argument for 'b'` (named for the first parameter you failed to supply). Keyword-style calls (`describe(a = "x")`) do not exist — `Error at 5:18: Expected ')' after arguments` — that `field = value` shape belongs to `raise` in chapter 11.

## Returning none

No return type means no invented value at the end: a function that reaches its end — or says plain `return` — returns `none`. Calling it in a `print` shows exactly that:

```luca
def func shout(text (str)) :
    print("{text}!")
    -- no return: yields none

print(shout("Here"))
```

```text
Here!
none
```

An explicit return type, `(str)` in the first example, checks every `return x` against it, and the function *must* end with a `return` (reaching the end would otherwise be a mismatch). A bare `return` without a value is an error in a typed function.

## Recursion

A function may call itself; nesting is capped at 64 deep — going past it errors with `Recursion limit exceeded` rather than hanging. Counting down is the clearest demo:

```luca
def func countdown(n (int)) :
    if n is == to 0 then :
        return
    print(n)
    countdown(n - 1)

countdown(3)
```

```text
3
2
1
```

## Closures

A function may access variables declared outside it — read them, and modify mutable ones. Each call sees the same outer binding:

```luca
def var total (int) = 0
def func add(n (int)) :
    total (int) = total + n

add(2)
add(3)
print(total)
```

```text
5
```

Rules worth knowing: `return` outside a function is an error; there is no overloading (two functions with one name is an error); parameters are independent values, so reassigning one inside does not change the caller's variable; recursion and local functions may shadow names freely.

Next: [collections](09-collections.md).
