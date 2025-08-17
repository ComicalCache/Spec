## spec

A proof-of-concept crate of adding "function specialization" to rust. It currently supports regular parameters, wild card (`_`) parameters and tuple destructuring. Struct destructuring is not yet supported.

The crate allows defining a function and its specialized cases, using rusts match statements to create a "specialized function". This has the neat side effect of getting exhaustive pattern enforcement by rust itself while staying light weight.

```rust
// This definition:
spec! {
    fn f(c: char) -> &'static str {}

    #[case('@')] {
        "[at]"
    }

    #[case(_)] #[when(c < 'a' || c > 'z')] {
        "not a lowercase letter"
    }

    #[case(_)] {
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
