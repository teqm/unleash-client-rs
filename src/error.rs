use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct TokenParseError;

impl Error for TokenParseError {}

impl Display for TokenParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not parse authorization token")
    }
}

#[derive(Debug)]
pub struct EntityTagParseError;

impl Error for EntityTagParseError {}

impl Display for EntityTagParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "could not parse etag")
    }
}
