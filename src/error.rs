pub type BoxResult<T> = Result<T, Box<std::error::Error>>;

#[derive(Debug, Clone)]
pub struct SimpleError {
    message: String
}

impl From<String> for SimpleError {
    fn from(message: String) -> SimpleError {
        SimpleError { message }
    }
}

impl std::fmt::Display for SimpleError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "SimpleError: {}", self.message)
    }
}

impl std::error::Error for SimpleError {}

pub type SimpleResult<T> = Result<T, SimpleError>;

pub fn error_if(value: bool, message: impl Into<String>)
-> SimpleResult<()> {
    if value {
        Err(message.into().into())
    } else {
        Ok(())
    }
}

pub fn error_if_not(value: bool, message: impl Into<String>)
-> SimpleResult<()> {
    error_if(!value, message)
}

