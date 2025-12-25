use crate::models::{GameServer, GameServerTestResult, GameServerError};
use crate::packet::{parse_packet_commands, parse_response_commands, build_packet, parse_response, hex_dump};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::time::{timeout, Duration, Instant};
use tokio::net::{UdpSocket, TcpStream};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub async fn test_game_server(server: &GameServer) -> GameServerTestResult {
    let start_time = Instant::now();

    // Parse packet commands
    let packet_commands = match parse_packet_commands(&server.pseudo_code) {
        Ok(cmds) => cmds,
        Err(e) => {
            return GameServerTestResult {
                success: false,
                response_time_ms: 0,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "SyntaxError".to_string(),
                    message: format!("Packet parsing error: {}", e),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            };
        }
    };

    // Build packet
    let packet = match build_packet(&packet_commands) {
        Ok(pkt) => pkt,
        Err(e) => {
            return GameServerTestResult {
                success: false,
                response_time_ms: 0,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "SyntaxError".to_string(),
                    message: format!("Packet building error: {}", e),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            };
        }
    };

    // Parse response commands
    let response_commands = match parse_response_commands(&server.pseudo_code) {
        Ok(cmds) => cmds,
        Err(e) => {
            return GameServerTestResult {
                success: false,
                response_time_ms: 0,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "SyntaxError".to_string(),
                    message: format!("Response parsing error: {}", e),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            };
        }
    };

    // Send packet and receive response
    let result = match server.protocol.as_str() {
        "UDP" => test_udp(&server.address, server.port, &packet, Duration::from_millis(server.timeout_ms)).await,
        "TCP" => test_tcp(&server.address, server.port, &packet, Duration::from_millis(server.timeout_ms)).await,
        _ => {
            return GameServerTestResult {
                success: false,
                response_time_ms: 0,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "SyntaxError".to_string(),
                    message: format!("Invalid protocol: {}. Must be UDP or TCP", server.protocol),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            };
        }
    };

    let response_time_ms = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(response_data) => {
            let raw_response = hex_dump(&response_data);
            
            // Parse response
            match parse_response(&response_commands, &response_data) {
                Ok((vars, validation_error)) => {
                    if let Some((line, msg)) = validation_error {
                        return GameServerTestResult {
                            success: false,
                            response_time_ms,
                            raw_response: Some(raw_response),
                            parsed_values: serde_json::json!(vars),
                            error: Some(GameServerError {
                                error_type: "ValidationError".to_string(),
                                message: msg,
                                line: Some(line),
                            }),
                            output_labels_success: Vec::new(),
                            output_labels_error: Vec::new(),
                        };
                    }

                    // Convert HashMap to JSON object
                    let parsed_json: serde_json::Value = vars.into_iter().collect();

                    GameServerTestResult {
                        success: true,
                        response_time_ms,
                        raw_response: Some(raw_response),
                        parsed_values: parsed_json,
                        error: None,
                        output_labels_success: Vec::new(),
                        output_labels_error: Vec::new(),
                    }
                }
                Err(e) => {
                    GameServerTestResult {
                        success: false,
                        response_time_ms,
                        raw_response: Some(raw_response),
                        parsed_values: serde_json::json!({}),
                        error: Some(GameServerError {
                            error_type: "ParseError".to_string(),
                            message: e.to_string(),
                            line: None,
                        }),
                        output_labels_success: Vec::new(),
                        output_labels_error: Vec::new(),
                    }
                }
            }
        }
        Err(e) => {
            GameServerTestResult {
                success: false,
                response_time_ms,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "NetworkError".to_string(),
                    message: e.to_string(),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            }
        }
    }
}

async fn test_udp(address: &str, port: u16, packet: &[u8], timeout_duration: Duration) -> Result<Vec<u8>> {
    let addr = format!("{}:{}", address, port);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(&addr).await?;

    let send_result = timeout(timeout_duration, socket.send(packet)).await;
    send_result??;

    let mut buf = vec![0u8; 16384];
    let recv_result = timeout(timeout_duration, socket.recv(&mut buf)).await;
    let size = recv_result??;
    
    buf.truncate(size);
    Ok(buf)
}

async fn test_tcp(address: &str, port: u16, packet: &[u8], timeout_duration: Duration) -> Result<Vec<u8>> {
    let addr = format!("{}:{}", address, port);
    let stream_result = timeout(timeout_duration, TcpStream::connect(&addr)).await;
    let mut stream = stream_result??;

    stream.write_all(packet).await?;
    stream.flush().await?;

    let mut buf = Vec::new();
    let mut temp_buf = vec![0u8; 16384];
    
    // Try to read with timeout
    let read_result = timeout(timeout_duration, stream.read(&mut temp_buf)).await;
    match read_result {
        Ok(Ok(size)) if size > 0 => {
            buf.extend_from_slice(&temp_buf[..size]);
            
            // Try to read more if available (non-blocking)
            loop {
                match stream.try_read(&mut temp_buf) {
                    Ok(size) if size > 0 => {
                        buf.extend_from_slice(&temp_buf[..size]);
                    }
                    _ => break,
                }
            }
        }
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => return Err(anyhow::anyhow!("Read timeout")),
    }

    Ok(buf)
}

