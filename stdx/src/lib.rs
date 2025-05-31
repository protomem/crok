use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    context: Vec<String>,
    origin: String,
}

impl Error {
    pub fn wrap(mut self, message: &str) -> Self {
        self.context.push(message.to_string());
        self
    }

    pub fn unwrap(self) -> String {
        self.origin
    }

    fn full_context(&self) -> String {
        self.context.join(": ")
    }

    fn origin_with_context(&self) -> String {
        format!("{}: {}", self.full_context(), self.origin)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error {
            origin: err,
            context: vec![],
        }
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::from(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::from(err.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.origin_with_context())
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
    }
}

#[derive(Debug, Default, Clone)]
pub struct Logger {
    context_stack: Vec<String>,
}

impl Logger {
    pub fn with(&self, context: &str) -> Self {
        let mut new_logger = self.clone();
        new_logger.context_stack.push(format!("[{}]", context));
        new_logger
    }

    pub fn log(&self, message: &str) {
        println!("{} {}", self.full_context(), message);
    }

    fn full_context(&self) -> String {
        self.context_stack.join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = Error::from("foo").wrap("bar").wrap("baz");
        assert_eq!(err.to_string(), "baz: bar: foo");
    }
}
