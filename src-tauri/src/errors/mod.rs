use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{message}")]
    NotFound { message: String },

    #[error("{message}")]
    ValidationError { message: String },

    #[error("{message}")]
    InvalidTransition { message: String },

    #[error("{message}")]
    DbError { message: String },

    #[error("{message}")]
    InternalError { message: String },

    #[error("{message}")]
    Conflict { message: String },
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let (code, message) = match self {
            AppError::NotFound { message } => ("NOT_FOUND", message.as_str()),
            AppError::ValidationError { message } => ("VALIDATION_ERROR", message.as_str()),
            AppError::InvalidTransition { message } => ("INVALID_TRANSITION", message.as_str()),
            AppError::DbError { message } => ("DB_ERROR", message.as_str()),
            AppError::InternalError { message } => ("INTERNAL_ERROR", message.as_str()),
            AppError::Conflict { message } => ("CONFLICT", message.as_str()),
        };

        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", code)?;
        state.serialize_field("message", message)?;
        state.end()
    }
}
