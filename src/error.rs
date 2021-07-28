use font_kit::error::{FontLoadingError, SelectionError};
use std::error::Error;
use std::fmt::{self, Display};
use std::num::ParseIntError;

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

#[derive(Debug, Eq, PartialEq)]
pub enum ParseColorError {
    InvalidLength,
    InvalidDigit,
}

impl Error for ParseColorError {}

impl Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseColorError::InvalidDigit => write!(f, "Invalid digit"),
            ParseColorError::InvalidLength => write!(f, "Invalid length"),
        }
    }
}

impl From<ParseIntError> for ParseColorError {
    fn from(_e: ParseIntError) -> Self {
        ParseColorError::InvalidDigit
    }
}

#[derive(Debug)]
pub struct GetLocError;

impl std::fmt::Display for GetLocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "end cannot be smaller than start")
    }
}
impl std::error::Error for GetLocError{}