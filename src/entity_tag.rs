use crate::error::EntityTagParseError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct EntityTag {
    tag: String,
}

impl Display for EntityTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "W/\"{}\"", self.tag)
    }
}

impl FromStr for EntityTag {
    type Err = EntityTagParseError;

    fn from_str(slice: &str) -> Result<Self, Self::Err> {
        let length = slice.len();

        if length >= 4 && slice.starts_with("W/\"") {
            Ok(EntityTag {
                tag: slice[3..length - 1].to_owned(),
            })
        } else {
            Err(EntityTagParseError)
        }
    }
}
