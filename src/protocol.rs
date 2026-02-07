// Agent Text Protocol parser and formatter
// T035-T041: Complete protocol implementation

use thiserror::Error;

// T036: SslFlag enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SslFlag {
    Ssl,
    NoSsl,
}

// T035: HealthCheckRequest struct
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequest {
    pub backend_server: String,
    pub backend_port: u16,
    pub ssl_flag: SslFlag,
    pub proxy_host_name: String,
}

// T038: HealthStatus enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Up,
    Down,
}

impl HealthStatus {
    pub fn to_protocol_string(self) -> String {
        match self {
            HealthStatus::Up => "up\n".to_string(),
            HealthStatus::Down => "down\n".to_string(),
        }
    }
}

// T037: HealthCheckResponse struct
#[derive(Debug, Clone)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
}

impl HealthCheckResponse {
    pub fn new(status: HealthStatus) -> Self {
        HealthCheckResponse { status }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.status.to_protocol_string()
    }
}

// T041: ParseError enum
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid field count: expected 4, got {0}")]
    InvalidFieldCount(usize),

    #[error("Invalid port: {0}")]
    InvalidPort(String),

    #[error("Invalid SSL flag: {0} (expected 'ssl' or 'no-ssl')")]
    InvalidSslFlag(String),

    #[error("Empty field: {0}")]
    EmptyField(String),
}

// T039: parse_request function
pub fn parse_request(line: &str) -> Result<HealthCheckRequest, ParseError> {
    let trimmed = line.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    // T040: Validate field count
    if parts.len() != 4 {
        return Err(ParseError::InvalidFieldCount(parts.len()));
    }

    let backend_server = parts[0];
    let backend_port_str = parts[1];
    let ssl_flag_str = parts[2];
    let proxy_host_name = parts[3];

    // Validate backend_server not empty
    if backend_server.is_empty() {
        return Err(ParseError::EmptyField("backend_server".to_string()));
    }

    // T040: Validate and parse port
    let backend_port: u16 = backend_port_str
        .parse()
        .map_err(|_| ParseError::InvalidPort(backend_port_str.to_string()))?;

    if backend_port == 0 {
        return Err(ParseError::InvalidPort(
            "port must be between 1 and 65535".to_string(),
        ));
    }

    // T040: Validate and parse ssl_flag
    let ssl_flag = match ssl_flag_str {
        "ssl" => SslFlag::Ssl,
        "no-ssl" => SslFlag::NoSsl,
        _ => return Err(ParseError::InvalidSslFlag(ssl_flag_str.to_string())),
    };

    // Validate proxy_host_name not empty
    if proxy_host_name.is_empty() {
        return Err(ParseError::EmptyField("proxy_host_name".to_string()));
    }

    Ok(HealthCheckRequest {
        backend_server: backend_server.to_string(),
        backend_port,
        ssl_flag,
        proxy_host_name: proxy_host_name.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_to_string() {
        assert_eq!(HealthStatus::Up.to_protocol_string(), "up\n");
        assert_eq!(HealthStatus::Down.to_protocol_string(), "down\n");
    }

    #[test]
    fn test_health_check_response() {
        let response = HealthCheckResponse::new(HealthStatus::Up);
        assert_eq!(response.to_string(), "up\n");
    }

    // T042: Unit test for parse_request with valid input
    #[test]
    fn test_parse_request_valid() {
        let input = "backend.example.com 50051 no-ssl backend.example.com";
        let result = parse_request(input);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.backend_server, "backend.example.com");
        assert_eq!(request.backend_port, 50051);
        assert_eq!(request.ssl_flag, SslFlag::NoSsl);
        assert_eq!(request.proxy_host_name, "backend.example.com");
    }

    #[test]
    fn test_parse_request_valid_ssl() {
        let input = "secure.example.com 443 ssl secure.example.com";
        let result = parse_request(input);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.backend_server, "secure.example.com");
        assert_eq!(request.backend_port, 443);
        assert_eq!(request.ssl_flag, SslFlag::Ssl);
        assert_eq!(request.proxy_host_name, "secure.example.com");
    }

    #[test]
    fn test_parse_request_valid_ip_address() {
        let input = "192.168.1.100 9090 no-ssl api.internal";
        let result = parse_request(input);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.backend_server, "192.168.1.100");
        assert_eq!(request.backend_port, 9090);
        assert_eq!(request.ssl_flag, SslFlag::NoSsl);
        assert_eq!(request.proxy_host_name, "api.internal");
    }

    // T043: Unit test for parse_request with invalid field count
    #[test]
    fn test_parse_request_invalid_field_count_too_few() {
        let input = "backend.example.com 50051";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidFieldCount(count) => assert_eq!(count, 2),
            _ => panic!("Expected InvalidFieldCount error"),
        }
    }

    #[test]
    fn test_parse_request_invalid_field_count_too_many() {
        let input = "backend.example.com 50051 no-ssl proxy.host extra field";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidFieldCount(count) => assert_eq!(count, 6),
            _ => panic!("Expected InvalidFieldCount error"),
        }
    }

    #[test]
    fn test_parse_request_empty_input() {
        let input = "";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidFieldCount(count) => assert_eq!(count, 0),
            _ => panic!("Expected InvalidFieldCount error"),
        }
    }

    // T044: Unit test for parse_request with invalid port
    #[test]
    fn test_parse_request_invalid_port_not_number() {
        let input = "backend.example.com invalid no-ssl proxy.host";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidPort(_) => {}
            _ => panic!("Expected InvalidPort error"),
        }
    }

    #[test]
    fn test_parse_request_invalid_port_zero() {
        let input = "backend.example.com 0 no-ssl proxy.host";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidPort(_) => {}
            _ => panic!("Expected InvalidPort error"),
        }
    }

    #[test]
    fn test_parse_request_invalid_port_too_large() {
        let input = "backend.example.com 65536 no-ssl proxy.host";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidPort(_) => {}
            _ => panic!("Expected InvalidPort error"),
        }
    }

    // T045: Unit test for parse_request with invalid ssl_flag
    #[test]
    fn test_parse_request_invalid_ssl_flag() {
        let input = "backend.example.com 50051 invalid-ssl proxy.host";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidSslFlag(flag) => assert_eq!(flag, "invalid-ssl"),
            _ => panic!("Expected InvalidSslFlag error"),
        }
    }

    #[test]
    fn test_parse_request_invalid_ssl_flag_typo() {
        let input = "backend.example.com 50051 no_ssl proxy.host";
        let result = parse_request(input);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidSslFlag(_) => {}
            _ => panic!("Expected InvalidSslFlag error"),
        }
    }

    #[test]
    fn test_parse_request_with_whitespace() {
        let input = "  backend.example.com   50051   no-ssl   proxy.host  ";
        let result = parse_request(input);

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.backend_server, "backend.example.com");
        assert_eq!(request.backend_port, 50051);
    }
}
