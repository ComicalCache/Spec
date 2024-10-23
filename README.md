## spec

First function in `spec!{}` must be annotated with `#[spec]`. Other 
annotations are allowed.

Following functions are annotated with `#[case(...)]`, the pattern follows 
normal match statement syntax. Functions are evaluated in definition order.
Functions must use the destructured variables as parameters (those must be 
the same!), the original parameters of the `#[spec]` function are only to 
be used if applicable.
