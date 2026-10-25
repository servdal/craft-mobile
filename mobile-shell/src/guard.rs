/// Last-resort host boundary. Engines still validate input and return normal errors.
/// This does not roll back an operation that mutated state before a panic.
pub fn guarded<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|_| {
        Err("The operation failed unexpectedly. Review the open document before continuing.".into())
    })
}
#[cfg(test)]
mod tests {
    use super::guarded;
    #[test]
    fn escaped_callback_panic_becomes_an_error() {
        let document = vec![1, 2, 3];
        let result: Result<(), String> = guarded(|| panic!("synthetic failed callback"));
        assert!(result.is_err());
        assert_eq!(document, vec![1, 2, 3]);
    }
    #[test]
    fn ordinary_errors_and_results_are_preserved() {
        assert_eq!(guarded(|| Ok(7)), Ok(7));
        assert_eq!(
            guarded::<()>(|| Err("format error".into())),
            Err("format error".into())
        );
    }
}
