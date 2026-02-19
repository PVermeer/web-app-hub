use crate::desktop_file::key::Key;
use std::fmt::Display;

#[derive(Debug)]
pub enum DesktopFileError {
    ValidationError(ValidationError),
    Other(anyhow::Error),
}
impl From<ValidationError> for DesktopFileError {
    fn from(e: ValidationError) -> Self {
        DesktopFileError::ValidationError(e)
    }
}
impl From<anyhow::Error> for DesktopFileError {
    fn from(e: anyhow::Error) -> Self {
        DesktopFileError::Other(e)
    }
}
impl Display for DesktopFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ValidationError(validation_error) => {
                write!(f, "{validation_error}")
            }
            Self::Other(error) => {
                write!(f, "{error}")
            }
        }
    }
}
impl std::error::Error for DesktopFileError {}
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: Key,
    pub message: String,
}
impl ValidationError {
    pub fn to_string_ui(&self) -> String {
        format!("{}: {}", self.message, self.field.to_ui_string())
    }
}
impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.message, self.field)
    }
}
impl std::error::Error for ValidationError {}
