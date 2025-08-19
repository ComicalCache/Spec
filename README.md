## spec

A proof-of-concept crate of adding "function specialization" to rust. It supports regular parameters, wild card (`_`) parameters, and tuple, tuple struct and struct destructuring.

The crate allows defining a function and its specialized cases, using rusts match statements to create a "specialized function". This has the neat side effect of getting exhaustive pattern enforcement by rust itself while staying light weight.

```rust
// This definition:
spec! {
    fn f(c: char) -> &'static str;

    '@' => {
        "[at]"
    }

    _ if c < 'a' || c > 'z' => {
        "not a lowercase letter"
    }

    _ => {
        "a lowercase letter"
    }
}

// Turns into this:
fn f(c: char) -> &'static str {
    #[allow(unused_parens)]
    match (c) {
        ('@') => "[at]",
        (_) if c < 'a' || c > 'z' => "not a lowercase letter",
        (_) => "a lowercase letter",
    }
}
```

Destructured tuples and structs are put into a tuple in the match case!

```rust
spec! {
    fn f((x, y): (i32, i32)) -> i32;

    (_ , 0) => { 0 }

    (_, _) => { x / y }
}
```

```rust
struct T {
    x: i32,
    y: i32,
}

spec! {
    fn f(T { x, y }: T) -> i32;

    (_ , 0) => { 0 }

    (_, _) => { x / y }
}
```
