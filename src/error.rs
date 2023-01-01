use std::fmt::Display;

#[derive(Debug)]
pub struct ApiError {
    messages: Vec<String>,
    err_count: usize,
}

impl ApiError {
    pub fn new(messages: Vec<String>) -> Self {
        let len = messages.len();
        ApiError {
            messages,
            err_count: len,
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg_fmt = self
            .messages
            .iter()
            .fold(String::new(), |acc, i| acc + i + "\n");
        write!(f, "Error count: {}\n{}", self.err_count, msg_fmt)
    }
}

impl std::error::Error for ApiError {}
