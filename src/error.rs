use thiserror::Error;

/// Errors surfaced while running a [`Console`](crate::Console).
#[derive(Debug, Error)]
pub enum Error {
    /// I/O failure, such as reading a script file.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// The line editor failed.
    #[error("readline: {0}")]
    Readline(String),

    /// A script failed to run.
    #[error("script: {0}")]
    Script(String),
}

/// Convenience alias for a `Result` whose error is [`enum@Error`].
pub type Result<T> = std::result::Result<T, Error>;
