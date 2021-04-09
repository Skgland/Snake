use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum MainError {
    SerializeError(ron::error::Error),
    DeserializeError(ron::de::Error),
    IOError(std::io::Error),
    Custom(String),
}

impl From<ron::
error::Error> for MainError {
    fn from(se: ron::error::Error) -> Self {
        MainError::SerializeError(se)
    }
}

impl From<std::io::Error> for MainError {
    fn from(io: std::io::Error) -> Self {
        MainError::IOError(io)
    }
}

impl From<String> for MainError {
    fn from(str: String) -> Self {
        MainError::Custom(str)
    }
}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            MainError::DeserializeError(e) => Display::fmt(e, f),
            MainError::SerializeError(e) => Display::fmt(e, f),
            MainError::IOError(e) => Display::fmt(e, f),
            MainError::Custom(e) => Display::fmt(e, f),
        }
    }
}
