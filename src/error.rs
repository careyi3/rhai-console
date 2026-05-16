use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("readline: {0}")]
    Readline(String),

    #[error("script: {0}")]
    Script(String),
}

pub type Result<T> = std::result::Result<T, Error>;
