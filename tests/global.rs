use std::ops::Not;

use specialize_fn::{case, spec};

#[spec]
fn bool_f(x: bool) -> bool {}

#[case(false)]
fn bool_f(_: bool) -> bool {
    true
}

#[case(true)]
fn bool_f(_: bool) -> bool {
    false
}

#[cfg(test)]
mod bool {
    use super::*;

    #[test]
    fn r#true() {
        assert!(bool_f(true).not());
    }

    #[test]
    fn r#false() {
        assert!(bool_f(false));
    }
}
