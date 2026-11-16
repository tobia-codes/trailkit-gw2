//! Displays an error with all its causes (ROADMAP R92).

use std::{error::Error, fmt};

/// Displays `error` followed by every cause: `cannot save: Access is denied.
/// (os error 5)`. A cause whose text already ends the previous text (as with
/// `#[error("cannot write: {0}")]` plus `#[from]`) is not repeated.
///
/// Log it as `error:% = ErrorChain(&error)`.
#[derive(Debug, Clone, Copy)]
pub struct ErrorChain<'a>(pub &'a (dyn Error + 'static));

impl fmt::Display for ErrorChain<'_> {
    /// Writes the error and its causes, separated by `: `.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut previous = self.0.to_string();
        formatter.write_str(&previous)?;
        let mut cause = self.0.source();
        while let Some(error) = cause {
            let text = error.to_string();
            if !previous.ends_with(&text) {
                write!(formatter, ": {text}")?;
            }
            previous = text;
            cause = error.source();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    /// An error that wraps another one.
    #[derive(Debug)]
    struct Wrapper {
        /// Text of this level.
        text: &'static str,
        /// The cause.
        cause: io::Error,
    }

    impl fmt::Display for Wrapper {
        /// Writes `text`.
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.text)
        }
    }

    impl Error for Wrapper {
        /// The wrapped I/O error.
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.cause)
        }
    }

    #[test]
    fn appends_every_cause() {
        let error = Wrapper {
            text: "cannot save notes.json",
            cause: io::Error::other("disk full"),
        };
        assert_eq!(
            ErrorChain(&error).to_string(),
            "cannot save notes.json: disk full"
        );
    }

    #[test]
    fn does_not_repeat_a_cause_already_in_the_text() {
        let error = Wrapper {
            text: "cannot write document: disk full",
            cause: io::Error::other("disk full"),
        };
        assert_eq!(
            ErrorChain(&error).to_string(),
            "cannot write document: disk full"
        );
    }
}
