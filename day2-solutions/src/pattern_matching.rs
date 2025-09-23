// Chapter 9: Pattern Matching Solutions

// ==========================
// Exercise 1: HTTP Status Handler
// ==========================

#[derive(Debug)]
pub enum HttpStatus {
    Ok,                    // 200
    NotFound,             // 404
    ServerError,          // 500
    Custom(u16),          // Any other code
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
}

pub fn handle_response(response: HttpResponse) -> String {
    match response {
        HttpResponse { status: HttpStatus::Ok, body: Some(content), .. } => {
            format!("Success: {}", content)
        }
        HttpResponse { status: HttpStatus::Ok, body: None, .. } => {
            "Success: No content".to_string()
        }
        HttpResponse { status: HttpStatus::NotFound, .. } => {
            "Error: Resource not found".to_string()
        }
        HttpResponse { status: HttpStatus::ServerError, .. } => {
            "Error: Internal server error".to_string()
        }
        HttpResponse { status: HttpStatus::Custom(code), .. } if code < 400 => {
            format!("Info: Status {}", code)
        }
        HttpResponse { status: HttpStatus::Custom(code), .. } => {
            format!("Error: Status {}", code)
        }
    }
}

// ==========================
// Exercise 2: Configuration Parser
// ==========================

#[derive(Debug, PartialEq)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    InvalidNumber(String),
    UnknownType,
}

pub fn parse_config_line(line: &str) -> Result<(String, ConfigValue), ParseError> {
    // Split by '=' to separate key from value
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() != 2 {
        return Err(ParseError::InvalidFormat);
    }

    let key_part = parts[0];
    let value_part = parts[1];

    // Check if key has type specification (key:type format)
    let (key, type_hint) = if key_part.contains(':') {
        let key_type_parts: Vec<&str> = key_part.split(':').collect();
        if key_type_parts.len() != 2 {
            return Err(ParseError::InvalidFormat);
        }
        (key_type_parts[0].to_string(), Some(key_type_parts[1]))
    } else {
        (key_part.to_string(), None)
    };

    // Parse value based on type hint or infer type
    let config_value = match type_hint {
        Some("string") => ConfigValue::String(value_part.to_string()),
        Some("int") => {
            match value_part.parse::<i64>() {
                Ok(num) => ConfigValue::Integer(num),
                Err(_) => return Err(ParseError::InvalidNumber(value_part.to_string())),
            }
        }
        Some("bool") => {
            match value_part {
                "true" => ConfigValue::Boolean(true),
                "false" => ConfigValue::Boolean(false),
                _ => return Err(ParseError::InvalidFormat),
            }
        }
        Some("array") => {
            let array_items: Vec<ConfigValue> = value_part
                .split(',')
                .map(|s| ConfigValue::String(s.trim().to_string()))
                .collect();
            ConfigValue::Array(array_items)
        }
        Some(_) => return Err(ParseError::UnknownType),
        None => {
            // Infer type from value
            if let Ok(num) = value_part.parse::<i64>() {
                ConfigValue::Integer(num)
            } else if value_part == "true" || value_part == "false" {
                ConfigValue::Boolean(value_part == "true")
            } else {
                ConfigValue::String(value_part.to_string())
            }
        }
    };

    Ok((key, config_value))
}

// ==========================
// Exercise 3: State Machine
// ==========================

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Idle,
    Processing { progress: u8 },
    Error { message: String, recoverable: bool },
    Complete,
}

#[derive(Debug)]
pub enum Event {
    Start,
    Progress(u8),
    Error(String, bool),
    Reset,
    Finish,
}

pub fn transition_state(current: State, event: Event) -> State {
    match (current, event) {
        // From Idle
        (State::Idle, Event::Start) => State::Processing { progress: 0 },

        // From Processing
        (State::Processing { .. }, Event::Progress(n)) => State::Processing { progress: n },
        (State::Processing { .. }, Event::Finish) => State::Complete,
        (State::Processing { .. }, Event::Error(msg, recoverable)) => {
            State::Error { message: msg, recoverable }
        }

        // From Error
        (State::Error { recoverable: true, .. }, Event::Reset) => State::Idle,
        (error_state @ State::Error { recoverable: false, .. }, Event::Reset) => error_state,

        // From Complete
        (State::Complete, Event::Reset) => State::Idle,

        // Invalid transitions - keep current state
        (current_state, _) => current_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_status_handler() {
        // Test successful response with body
        let response = HttpResponse {
            status: HttpStatus::Ok,
            body: Some("Hello World".to_string()),
            headers: vec![],
        };
        assert_eq!(handle_response(response), "Success: Hello World");

        // Test successful response without body
        let response = HttpResponse {
            status: HttpStatus::Ok,
            body: None,
            headers: vec![],
        };
        assert_eq!(handle_response(response), "Success: No content");

        // Test not found
        let response = HttpResponse {
            status: HttpStatus::NotFound,
            body: None,
            headers: vec![],
        };
        assert_eq!(handle_response(response), "Error: Resource not found");

        // Test server error
        let response = HttpResponse {
            status: HttpStatus::ServerError,
            body: None,
            headers: vec![],
        };
        assert_eq!(handle_response(response), "Error: Internal server error");

        // Test custom status < 400
        let response = HttpResponse {
            status: HttpStatus::Custom(202),
            body: None,
            headers: vec![],
        };
        assert_eq!(handle_response(response), "Info: Status 202");

        // Test custom status >= 400
        let response = HttpResponse {
            status: HttpStatus::Custom(403),
            body: None,
            headers: vec![],
        };
        assert_eq!(handle_response(response), "Error: Status 403");
    }

    #[test]
    fn test_config_parser() {
        // Test string without type hint
        let result = parse_config_line("name=John").unwrap();
        assert_eq!(result.0, "name");
        assert_eq!(result.1, ConfigValue::String("John".to_string()));

        // Test explicit string type
        let result = parse_config_line("title:string=Manager").unwrap();
        assert_eq!(result.0, "title");
        assert_eq!(result.1, ConfigValue::String("Manager".to_string()));

        // Test integer with type hint
        let result = parse_config_line("port:int=8080").unwrap();
        assert_eq!(result.0, "port");
        assert_eq!(result.1, ConfigValue::Integer(8080));

        // Test integer without type hint (inferred)
        let result = parse_config_line("timeout=30").unwrap();
        assert_eq!(result.0, "timeout");
        assert_eq!(result.1, ConfigValue::Integer(30));

        // Test boolean
        let result = parse_config_line("debug:bool=true").unwrap();
        assert_eq!(result.0, "debug");
        assert_eq!(result.1, ConfigValue::Boolean(true));

        // Test array
        let result = parse_config_line("tags:array=tag1,tag2,tag3").unwrap();
        assert_eq!(result.0, "tags");
        assert_eq!(result.1, ConfigValue::Array(vec![
            ConfigValue::String("tag1".to_string()),
            ConfigValue::String("tag2".to_string()),
            ConfigValue::String("tag3".to_string()),
        ]));

        // Test error cases
        assert_eq!(parse_config_line("invalid"), Err(ParseError::InvalidFormat));
        assert_eq!(parse_config_line("port:int=abc"), Err(ParseError::InvalidNumber("abc".to_string())));
        assert_eq!(parse_config_line("key:unknown=value"), Err(ParseError::UnknownType));
    }

    #[test]
    fn test_state_machine() {
        // Test transitions from Idle
        let state = transition_state(State::Idle, Event::Start);
        assert_eq!(state, State::Processing { progress: 0 });

        // Test transitions from Processing
        let state = transition_state(State::Processing { progress: 0 }, Event::Progress(50));
        assert_eq!(state, State::Processing { progress: 50 });

        let state = transition_state(State::Processing { progress: 50 }, Event::Finish);
        assert_eq!(state, State::Complete);

        let state = transition_state(
            State::Processing { progress: 30 },
            Event::Error("Connection failed".to_string(), true)
        );
        assert_eq!(state, State::Error {
            message: "Connection failed".to_string(),
            recoverable: true
        });

        // Test transitions from Error
        let state = transition_state(
            State::Error { message: "Test error".to_string(), recoverable: true },
            Event::Reset
        );
        assert_eq!(state, State::Idle);

        let error_state = State::Error { message: "Fatal error".to_string(), recoverable: false };
        let state = transition_state(error_state.clone(), Event::Reset);
        assert_eq!(state, error_state);

        // Test transitions from Complete
        let state = transition_state(State::Complete, Event::Reset);
        assert_eq!(state, State::Idle);

        // Test invalid transitions
        let state = transition_state(State::Idle, Event::Finish);
        assert_eq!(state, State::Idle); // Should stay in same state
    }
}