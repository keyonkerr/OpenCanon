use canon_core::Error as CoreError;
use canon_store::Error as StoreError;

#[derive(Debug)]
pub enum CliError {
    InvalidJson {
        message: String,
    },
    Io {
        message: String,
    },
    Validation {
        index: usize,
        field: Option<String>,
        message: String,
    },
    Core(CoreError),
    Store(StoreError),
    AtomNotFound {
        id: String,
        index: Option<usize>,
    },
}

impl From<CoreError> for CliError {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl CliError {
    pub fn invalid_json(message: impl Into<String>) -> Self {
        Self::InvalidJson {
            message: message.into(),
        }
    }

    pub fn validation(
        index: usize,
        field: impl Into<Option<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self::Validation {
            index,
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn from_store(err: StoreError, index: Option<usize>) -> Self {
        match err {
            StoreError::NotFound { id } => Self::AtomNotFound { id, index },
            other => Self::Store(other),
        }
    }
}
