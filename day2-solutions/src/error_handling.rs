// Chapter 10: Error Handling Solutions

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::path::Path;

// ==========================
// Exercise 1: Configuration Parser
// ==========================

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(String),
    ValidationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let mut settings = HashMap::new();
        
        for line in contents.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse key=value pairs
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(ConfigError::ParseError(
                    format!("Invalid line format: {}", line)
                ));
            }
            
            settings.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }
        
        // Validate required keys (example requirement)
        if !settings.contains_key("version") {
            return Err(ConfigError::ValidationError(
                "Missing required key: version".to_string()
            ));
        }
        
        Ok(Config { settings })
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }
    
    pub fn get_required(&self, key: &str) -> Result<&String, ConfigError> {
        self.settings
            .get(key)
            .ok_or_else(|| ConfigError::ValidationError(format!("Missing key: {}", key)))
    }
    
    pub fn get_int(&self, key: &str) -> Result<i32, ConfigError> {
        let value = self.get_required(key)?;
        value
            .parse()
            .map_err(|e: ParseIntError| ConfigError::ParseError(e.to_string()))
    }
}

// ==========================
// Exercise 2: Multi-Error Handler
// ==========================

#[derive(Debug)]
pub enum ProcessError {
    FileError { path: String, error: io::Error },
    ParseError { line: usize, error: ParseIntError },
    ValidationError(String),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessError::FileError { path, error } => {
                write!(f, "File error at {}: {}", path, error)
            },
            ProcessError::ParseError { line, error } => {
                write!(f, "Parse error at line {}: {}", line, error)
            },
            ProcessError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

pub struct DataProcessor {
    pub errors: Vec<ProcessError>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { errors: Vec::new() }
    }
    
    pub fn process_file(&mut self, path: &str) -> Result<Vec<i32>, ProcessError> {
        let contents = fs::read_to_string(path).map_err(|e| {
            ProcessError::FileError {
                path: path.to_string(),
                error: e,
            }
        })?;
        
        let mut numbers = Vec::new();
        
        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            match line.parse::<i32>() {
                Ok(num) => {
                    if num < 0 {
                        self.errors.push(ProcessError::ValidationError(
                            format!("Negative number {} at line {}", num, line_num + 1)
                        ));
                    } else {
                        numbers.push(num);
                    }
                },
                Err(e) => {
                    self.errors.push(ProcessError::ParseError {
                        line: line_num + 1,
                        error: e,
                    });
                }
            }
        }
        
        Ok(numbers)
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn report_errors(&self) {
        for error in &self.errors {
            eprintln!("{}", error);
        }
    }
}

// ==========================
// Exercise 3: Result Chain Builder
// ==========================

#[derive(Debug)]
pub struct EmailBuilder {
    to: Option<String>,
    from: Option<String>,
    subject: Option<String>,
    body: Option<String>,
}

#[derive(Debug)]
pub enum EmailError {
    MissingField(&'static str),
    InvalidEmail(String),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmailError::MissingField(field) => write!(f, "Missing required field: {}", field),
            EmailError::InvalidEmail(email) => write!(f, "Invalid email format: {}", email),
        }
    }
}

impl std::error::Error for EmailError {}

impl EmailBuilder {
    pub fn new() -> Self {
        EmailBuilder {
            to: None,
            from: None,
            subject: None,
            body: None,
        }
    }
    
    fn validate_email(email: &str) -> Result<(), EmailError> {
        if !email.contains('@') {
            Err(EmailError::InvalidEmail(email.to_string()))
        } else {
            Ok(())
        }
    }
    
    pub fn to(mut self, email: &str) -> Result<Self, EmailError> {
        Self::validate_email(email)?;
        self.to = Some(email.to_string());
        Ok(self)
    }
    
    pub fn from(mut self, email: &str) -> Result<Self, EmailError> {
        Self::validate_email(email)?;
        self.from = Some(email.to_string());
        Ok(self)
    }
    
    pub fn subject(mut self, subject: &str) -> Result<Self, EmailError> {
        if subject.is_empty() {
            return Err(EmailError::MissingField("subject cannot be empty"));
        }
        self.subject = Some(subject.to_string());
        Ok(self)
    }
    
    pub fn body(mut self, body: &str) -> Result<Self, EmailError> {
        self.body = Some(body.to_string());
        Ok(self)
    }
    
    pub fn build(self) -> Result<Email, EmailError> {
        let to = self.to.ok_or(EmailError::MissingField("to"))?;
        let from = self.from.ok_or(EmailError::MissingField("from"))?;
        let subject = self.subject.ok_or(EmailError::MissingField("subject"))?;
        let body = self.body.ok_or(EmailError::MissingField("body"))?;
        
        Ok(Email {
            to,
            from,
            subject,
            body,
        })
    }
}

pub struct Email {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub body: String,
}

impl Email {
    pub fn send(&self) -> Result<(), EmailError> {
        println!("Sending email from {} to {}", self.from, self.to);
        println!("Subject: {}", self.subject);
        println!("Body: {}", self.body);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_builder() {
        let email_result = EmailBuilder::new()
            .to("user@example.com")
            .and_then(|b| b.from("sender@example.com"))
            .and_then(|b| b.subject("Test"))
            .and_then(|b| b.body("Hello"))
            .and_then(|b| b.build());
        
        assert!(email_result.is_ok());
    }
    
    #[test]
    fn test_invalid_email() {
        let result = EmailBuilder::new().to("invalid");
        assert!(result.is_err());
    }
}