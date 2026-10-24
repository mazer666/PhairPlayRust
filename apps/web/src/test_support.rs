//! The helpers that the tests of the dashboard share.
//!
//! WHY: The workspace denies `unwrap()` and `expect()`. A test that fails
//! has to say *why* it failed, and a panic has to be a deliberate decision
//! instead of a habit.

/// Unwraps the result of a call in a test.
pub(crate) trait Must<T> {
    /// Unwraps the value or fails with the given context.
    fn must(self, context: &str) -> T;
}

impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, context: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }
}

impl<T> Must<T> for Option<T> {
    fn must(self, context: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{context}: the value was missing"),
        }
    }
}

/// Unwraps the error of a call that had to fail in a test.
pub(crate) trait MustErr<T, E> {
    /// Unwraps the error or fails with the given context.
    fn must_err(self, context: &str) -> E;
}

impl<T: std::fmt::Debug, E: std::fmt::Debug> MustErr<T, E> for Result<T, E> {
    fn must_err(self, context: &str) -> E {
        match self {
            Ok(value) => {
                panic!("{context}: the call was expected to fail, but it returned {value:?}")
            }
            Err(error) => error,
        }
    }
}
