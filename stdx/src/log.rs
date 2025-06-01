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
