use rust_template_core::{Error, greet};

#[test]
fn greets_trimmed_name() {
    assert_eq!(greet("  Ferris ").unwrap(), "Hello, Ferris!");
}

#[test]
fn rejects_blank_name() {
    assert!(matches!(greet("   "), Err(Error::EmptyName)));
}
