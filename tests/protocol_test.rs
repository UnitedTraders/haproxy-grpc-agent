// Unit tests for protocol parsing
// T042-T045: Protocol parser tests

use haproxy_grpc_agent::protocol::{ParseError, SslFlag, parse_request};

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
