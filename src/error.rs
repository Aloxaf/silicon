use font_kit::error::{FontLoadingError, SelectionError};
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum FontError {
    SelectionError(SelectionError),
    FontLoadingError(FontLoadingError),
}

impl Error for FontError {}

impl Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FontError::SelectionError(e) => write!(f, "Font error: {}", e),
            FontError::FontLoadingError(e) => write!(f, "Font error: {}", e),
        }
    }
}

impl From<SelectionError> for FontError {
    fn from(e: SelectionError) -> Self {
        FontError::SelectionError(e)
    }
}

impl From<FontLoadingError> for FontError {
    fn from(e: FontLoadingError) -> Self {
        FontError::FontLoadingError(e)
    }
}
