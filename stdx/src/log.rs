#[derive(Debug, Default, Clone)]
pub struct Logger {
    constraint_level: Level,
    level: Level,
    // TODO: To link list?
    context_stack: Vec<String>,
}

impl Logger {
    pub fn with(&self, context: &str) -> Self {
        let mut new_logger = self.clone();
        new_logger.context_stack.push(format!("[{}]", context));
        new_logger
    }

    pub fn with_constraint(&self, constraint_level: Level) -> Self {
        let mut new_logger = self.clone();
        new_logger.constraint_level = constraint_level;
        new_logger
    }

    pub fn with_level(&self, level: Level) -> Self {
        let mut new_logger = self.clone();
        new_logger.level = level;
        new_logger
    }

    pub fn log(&self, message: &str) {
        if self.constraint_level > self.level {
            return;
        }

        println!(
            "{} {} {}",
            self.level.to_string(),
            self.full_context(),
            message
        );
    }

    fn full_context(&self) -> String {
        self.context_stack.join("")
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Level {
    Debug,
    Info,
    Error,

    System,
    SystemError,
}

impl Default for Level {
    fn default() -> Self {
        Level::System
    }
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match self {
            Level::System => "SYSTEM",
            Level::SystemError => "SYSTEM_ERROR",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Error => "ERROR",
        }
        .to_string()
    }
}
