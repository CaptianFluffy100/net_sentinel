use crate::models::{GameServer, Protocol, GameServerTestResult, GameServerError};
use crate::packet_parser::{build_packets, parse_response, parse_script, OutputBlock, OutputCommand, OutputStatus};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

pub async fn check_game_server(server: &GameServer) -> GameServerTestResult {
    println!("[CHECK] Starting game server check for {} ({:?}://{}:{})", 
             server.name, server.protocol, server.address, server.port);
    let start = Instant::now();

    // Parse the pseudo-code script
    println!("[CHECK] Step 1: Parsing pseudo-code script...");
    let resolved_code = replace_placeholders(&server.pseudo_code, server);
    let script = match parse_script(&resolved_code) {
        Ok(s) => {
            println!("[CHECK] Script parsed successfully");
            s
        },
        Err(e) => {
            println!("[CHECK] Script parsing failed: {}", e);
            return GameServerTestResult {
                success: false,
                response_time_ms: 0,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "SyntaxError".to_string(),
                    message: e.to_string(),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            };
        }
    };

    // Build all packets first (for length calculations)
    println!("[CHECK] Step 2: Building {} packet/response pair(s)...", script.pairs.len());
    let built_packets = match build_packets(&script) {
        Ok(p) => {
            let total_bytes: usize = p.iter().map(|packet| packet.len()).sum();
            println!("[CHECK] Packets built successfully: {} packet(s), {} total bytes", p.len(), total_bytes);
            p
        },
        Err(e) => {
            println!("[CHECK] Packet building failed: {}", e);
            return GameServerTestResult {
                success: false,
                response_time_ms: 0,
                raw_response: None,
                parsed_values: serde_json::json!({}),
                error: Some(GameServerError {
                    error_type: "SyntaxError".to_string(),
                    message: e.to_string(),
                    line: None,
                }),
                output_labels_success: Vec::new(),
                output_labels_error: Vec::new(),
            };
        }
    };

    // Execute pairs sequentially: send packet, receive response, parse response
    println!("[CHECK] Step 3: Executing {} pair(s) sequentially via {:?} to {}:{} (timeout: {}ms)...", 
             script.pairs.len(), server.protocol, server.address, server.port, server.timeout_ms);
    
    let mut all_responses = Vec::new();
    let mut all_parsed_vars = HashMap::new();
    let mut last_error: Option<GameServerError> = None;

    // Create and maintain connection/socket for all pairs
    let pair_results: Vec<Result<Vec<u8>, anyhow::Error>> = match server.protocol {
        Protocol::Udp => {
            // Create UDP socket once and reuse for all pairs
            use tokio::net::UdpSocket;
            let addr = format!("{}:{}", server.address, server.port);
            println!("[UDP] Binding UDP socket...");
            let socket = match UdpSocket::bind("0.0.0.0:0").await {
                Ok(s) => s,
                Err(e) => {
                    return GameServerTestResult {
                        success: false,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        raw_response: None,
                        parsed_values: serde_json::json!({}),
                        error: Some(GameServerError {
                            error_type: "NetworkError".to_string(),
                            message: format!("Failed to create UDP socket: {}", e),
                            line: None,
                        }),
                        output_labels_success: Vec::new(),
                        output_labels_error: Vec::new(),
                    };
                }
            };
            println!("[UDP] Socket created, will be reused for all {} pair(s)", script.pairs.len());
            
            // Execute all pairs with the same socket
            let mut results = Vec::new();
            for (pair_idx, _pair) in script.pairs.iter().enumerate() {
                println!("[CHECK] Executing pair {} of {}...", pair_idx + 1, script.pairs.len());
                let packet = &built_packets[pair_idx];
                match send_packet_udp(&socket, &addr, packet, server.timeout_ms).await {
                    Ok(response) => results.push(Ok(response)),
                    Err(e) => {
                        results.push(Err(e));
                        break;
                    }
                }
            }
            results
        },
        Protocol::Tcp => {
            // Create TCP connection once and reuse for all pairs
            use tokio::net::TcpStream;
            use tokio::time::{timeout, Duration};
            
            let addr = format!("{}:{}", server.address, server.port);
            let timeout_duration = Duration::from_millis(server.timeout_ms);
            
            println!("[TCP] Connecting to {} (timeout: {}ms)...", addr, server.timeout_ms);
            let mut stream = match timeout(timeout_duration, TcpStream::connect(&addr)).await {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    return GameServerTestResult {
                        success: false,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        raw_response: None,
                        parsed_values: serde_json::json!({}),
                        error: Some(GameServerError {
                            error_type: "NetworkError".to_string(),
                            message: format!("Failed to connect to server: {}", e),
                            line: None,
                        }),
                        output_labels_success: Vec::new(),
                        output_labels_error: Vec::new(),
                    };
                }
                Err(_) => {
                    return GameServerTestResult {
                        success: false,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        raw_response: None,
                        parsed_values: serde_json::json!({}),
                        error: Some(GameServerError {
                            error_type: "NetworkError".to_string(),
                            message: "Connection timeout".to_string(),
                            line: None,
                        }),
                        output_labels_success: Vec::new(),
                        output_labels_error: Vec::new(),
                    };
                }
            };
            println!("[TCP] Connected successfully, connection will be reused for all {} pair(s)", script.pairs.len());
            
            // Execute all pairs with the same connection
            let mut results = Vec::new();
            for (pair_idx, _pair) in script.pairs.iter().enumerate() {
                println!("[CHECK] Executing pair {} of {}...", pair_idx + 1, script.pairs.len());
                let packet = &built_packets[pair_idx];
                match send_packet_tcp(&mut stream, packet, timeout_duration).await {
                    Ok(response) => results.push(Ok(response)),
                    Err(e) => {
                        results.push(Err(e));
                        break;
                    }
                }
            }
            
            // Connection will be closed when stream goes out of scope
            println!("[TCP] All pairs complete, closing connection");
            results
        },
    };

    for (pair_idx, pair_result) in pair_results.iter().enumerate() {
        let pair = &script.pairs[pair_idx];
        
        let response = match pair_result {
            Ok(r) => {
                println!("[CHECK] Pair {} response received: {} bytes", pair_idx + 1, r.len());
                r.clone()
            },
            Err(e) => {
                println!("[CHECK] Pair {} network error: {}", pair_idx + 1, e);
                last_error = Some(GameServerError {
                    error_type: "NetworkError".to_string(),
                    message: format!("Pair {}: {}", pair_idx + 1, e),
                    line: None,
                });
                break;
            }
        };

        all_responses.push(response.clone());
        
        // Parse the response with this pair's response commands
        println!("[CHECK] Parsing pair {} response with {} response commands...", pair_idx + 1, pair.response.len());
        match parse_response(&pair.response, &response) {
            Ok((vars, _bytes_read)) => {
                println!("[CHECK] Pair {} response parsing successful: {} variables extracted", 
                         pair_idx + 1, vars.len());
                // Merge variables into all_parsed_vars (later pairs can override earlier ones)
                all_parsed_vars.extend(vars);
            }
            Err(e) => {
                println!("[CHECK] Pair {} response parsing failed: {}", pair_idx + 1, e);
                last_error = Some(GameServerError {
                    error_type: "ParseError".to_string(),
                    message: format!("Pair {}: {}", pair_idx + 1, e),
                    line: None,
                });
                break;
            }
        }
    }

    let response_time_ms = start.elapsed().as_millis() as u64;
    let raw_response_hex = if all_responses.len() == 1 {
        hex::encode(&all_responses[0])
    } else {
        // Multiple responses - concatenate hex strings
        all_responses.iter().map(|r| hex::encode(r)).collect::<Vec<_>>().join(" ")
    };

    if let Some(err) = last_error {
        let error_labels = evaluate_output_labels(&script, OutputStatus::Error, &mut HashMap::new(), server, Some(&err));
        return GameServerTestResult {
            success: false,
            response_time_ms,
            raw_response: Some(raw_response_hex),
            parsed_values: serde_json::json!({}),
            error: Some(err),
            output_labels_success: Vec::new(),
            output_labels_error: error_labels,
        };
    }

    // All pairs succeeded
    let success_labels = evaluate_output_labels(&script, OutputStatus::Success, &mut all_parsed_vars.clone(), server, None);
    strip_placeholder_vars(&mut all_parsed_vars);
    let parsed_values: serde_json::Value = all_parsed_vars.clone().into_iter().collect();

    println!("[CHECK] All pairs executed successfully: {} total variables extracted in {}ms", 
             all_parsed_vars.len(), response_time_ms);
    GameServerTestResult {
        success: true,
        response_time_ms,
        raw_response: Some(raw_response_hex),
        parsed_values,
        error: None,
        output_labels_success: success_labels,
        output_labels_error: Vec::new(),
    }
}

async fn send_single_udp_packet(
    address: &str,
    port: u16,
    packet: &[u8],
    timeout_ms: u64,
) -> Result<Vec<u8>> {
    use tokio::net::UdpSocket;
    use tokio::time::{timeout, Duration};

    let addr = format!("{}:{}", address, port);
    println!("[UDP] Binding UDP socket...");
    let socket = UdpSocket::bind("0.0.0.0:0").await
        .context("Failed to create UDP socket")?;

    println!("[UDP] Sending packet ({} bytes) to {}...", packet.len(), addr);
    socket
        .send_to(packet, &addr)
        .await
        .context("Failed to send UDP packet")?;
    println!("[UDP] Packet sent successfully, waiting for response (timeout: {}ms)...", timeout_ms);

    let mut buf = vec![0u8; 16384];
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, socket.recv_from(&mut buf)).await {
        Ok(Ok((size, _))) => {
            println!("[UDP] Response received: {} bytes", size);
            Ok(buf[..size].to_vec())
        },
        Ok(Err(e)) => {
            Err(anyhow::anyhow!("Failed to receive UDP response: {}", e))
        },
        Err(_) => {
            println!("[UDP] Request timed out after {}ms", timeout_ms);
            Err(anyhow::anyhow!("UDP request timed out after {}ms", timeout_ms))
        },
    }
}

async fn send_packet_udp(
    socket: &tokio::net::UdpSocket,
    addr: &str,
    packet: &[u8],
    timeout_ms: u64,
) -> Result<Vec<u8>> {
    use tokio::time::{timeout, Duration};

    println!("[UDP] Sending packet ({} bytes) to {}...", packet.len(), addr);
    socket
        .send_to(packet, addr)
        .await
        .context("Failed to send UDP packet")?;
    println!("[UDP] Packet sent successfully, waiting for response (timeout: {}ms)...", timeout_ms);

    let mut buf = vec![0u8; 16384];
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, socket.recv_from(&mut buf)).await {
        Ok(Ok((size, _))) => {
            println!("[UDP] Response received: {} bytes", size);
            Ok(buf[..size].to_vec())
        },
        Ok(Err(e)) => {
            Err(anyhow::anyhow!("Failed to receive UDP response: {}", e))
        },
        Err(_) => {
            println!("[UDP] Request timed out after {}ms", timeout_ms);
            Err(anyhow::anyhow!("UDP request timed out after {}ms", timeout_ms))
        },
    }
}

async fn send_packet_tcp(
    stream: &mut tokio::net::TcpStream,
    packet: &[u8],
    timeout_duration: tokio::time::Duration,
) -> Result<Vec<u8>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::time::timeout;

    println!("[TCP] Sending packet ({} bytes)...", packet.len());
    timeout(timeout_duration, stream.write_all(packet))
        .await
        .context("Send timeout")?
        .context("Failed to write packet")?;
    println!("[TCP] Packet sent successfully, waiting for response...");

    // Read response
    let mut buf = vec![0u8; 16384];
    let size = timeout(timeout_duration, stream.read(&mut buf))
        .await
        .context("Read timeout")?
        .context("Failed to read response")?;
    println!("[TCP] Response received: {} bytes", size);
    Ok(buf[..size].to_vec())
}

async fn send_udp_packets(
    address: &str,
    port: u16,
    packets: &[Vec<u8>],
    timeout_ms: u64,
) -> Result<Vec<u8>> {
    use tokio::net::UdpSocket;
    use tokio::time::{timeout, Duration};

    let addr = format!("{}:{}", address, port);
    println!("[UDP] Binding UDP socket...");
    let socket = UdpSocket::bind("0.0.0.0:0").await
        .context("Failed to create UDP socket")?;

    // Send all packets sequentially
    for (idx, packet) in packets.iter().enumerate() {
        println!("[UDP] Sending packet {} of {} ({} bytes) to {}...", idx + 1, packets.len(), packet.len(), addr);
        socket
            .send_to(packet, &addr)
            .await
            .context(format!("Failed to send UDP packet {}", idx + 1))?;
        println!("[UDP] Packet {} sent successfully", idx + 1);
    }
    
    println!("[UDP] All packets sent, waiting for response (timeout: {}ms)...", timeout_ms);

    let mut buf = vec![0u8; 16384];
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, socket.recv_from(&mut buf)).await {
        Ok(Ok((size, _))) => {
            println!("[UDP] Response received: {} bytes", size);
            Ok(buf[..size].to_vec())
        },
        Ok(Err(e)) => {
            println!("[UDP] Failed to receive response: {}", e);
            Err(anyhow::anyhow!("Failed to receive UDP response: {}", e))
        },
        Err(_) => {
            println!("[UDP] Request timed out after {}ms", timeout_ms);
            Err(anyhow::anyhow!("UDP request timed out after {}ms", timeout_ms))
        },
    }
}

async fn send_single_tcp_packet(
    address: &str,
    port: u16,
    packet: &[u8],
    timeout_ms: u64,
) -> Result<Vec<u8>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::{timeout, Duration};

    let addr = format!("{}:{}", address, port);
    let timeout_duration = Duration::from_millis(timeout_ms);

    println!("[TCP] Connecting to {} (timeout: {}ms)...", addr, timeout_ms);
    let mut stream = timeout(timeout_duration, TcpStream::connect(&addr))
        .await
        .context("Connection timeout")?
        .context("Failed to connect to server")?;
    println!("[TCP] Connected successfully");

    println!("[TCP] Sending packet ({} bytes)...", packet.len());
    timeout(timeout_duration, stream.write_all(packet))
        .await
        .context("Send timeout")?
        .context("Failed to write packet")?;
    println!("[TCP] Packet sent successfully, waiting for response (timeout: {}ms)...", timeout_ms);

    // Read response
    let mut buf = vec![0u8; 16384];
    let size = timeout(timeout_duration, stream.read(&mut buf))
        .await
        .context("Receive timeout")?
        .context("Failed to read response")?;

    println!("[TCP] Response received: {} bytes", size);
    Ok(buf[..size].to_vec())
}

fn evaluate_output_labels(
    script: &crate::packet_parser::PacketScript,
    status: OutputStatus,
    vars: &mut HashMap<String, Value>,
    server: &GameServer,
    error: Option<&GameServerError>,
) -> Vec<String> {
    insert_server_placeholders(vars, server);
    match process_output_blocks(&script.output_blocks, status, vars, server, error) {
        Ok(lines) => lines,
        Err(e) => {
            println!("[CHECK] Output formatting error: {}", e);
            Vec::new()
        }
    }
}

fn process_output_blocks(
    blocks: &[OutputBlock],
    status: OutputStatus,
    vars: &mut HashMap<String, Value>,
    server: &GameServer,
    error: Option<&GameServerError>,
) -> Result<Vec<String>> {
    let mut labels = Vec::new();
    for block in blocks.iter().filter(|block| block.status == status) {
        labels.extend(evaluate_output_block(block, vars, server, error)?);
    }
    Ok(labels)
}

fn evaluate_output_block(
    block: &OutputBlock,
    vars: &mut HashMap<String, Value>,
    server: &GameServer,
    error: Option<&GameServerError>,
) -> Result<Vec<String>> {
    let mut results = Vec::new();
    println!("[CHECK] Evaluating output block with {} commands", block.commands.len());
    
    // Print all current variables for debugging
    println!("[CHECK] Variables before processing output block: {:?}", 
             vars.keys().collect::<Vec<_>>());
    
    for (idx, command) in block.commands.iter().enumerate() {
        match command {
            OutputCommand::JsonOutput(var) => {
                println!("[CHECK] Command {}: JSON_OUTPUT {}", idx + 1, var);
                handle_json_output(var, vars)?;
                // Print variable after JSON_OUTPUT
                if let Some(value) = vars.get(var) {
                    println!("[CHECK] Variable {} after JSON_OUTPUT: type={:?}, preview={:?}", 
                             var, 
                             if value.is_string() { "String" } 
                             else if value.is_object() { "Object" } 
                             else if value.is_array() { "Array" } 
                             else { "Other" },
                             if value.is_string() { 
                                 value.as_str().map(|s| if s.len() > 50 { format!("{}...", &s[..50]) } else { s.to_string() })
                             } else { 
                                 Some(format!("{}", value))
                             });
                }
            },
            OutputCommand::Return(template) => {
                println!("[CHECK] Command {}: RETURN with template: {}", idx + 1, template);
                let result = format_return(template, vars, server, error);
                println!("[CHECK] RETURN resolved to: {}", result);
                results.push(result);
            }
        }
    }
    Ok(results)
}

fn handle_json_output(var: &str, vars: &mut HashMap<String, Value>) -> Result<()> {
    println!("[CHECK] JSON_OUTPUT: Looking for variable '{}'", var);
    println!("[CHECK] JSON_OUTPUT: Available variables: {:?}", vars.keys().collect::<Vec<_>>());
    
    if let Some(value) = vars.get(var).cloned() {
        println!("[CHECK] JSON_OUTPUT {}: Found variable, type: {:?}", var, 
                 if value.is_string() { "String" } 
                 else if value.is_object() { "Object" } 
                 else { "Other" });
        
        if let Some(text) = value.as_str() {
            println!("[CHECK] JSON_OUTPUT {}: Parsing JSON string (length: {}): {}", 
                     var, text.len(), 
                     if text.len() > 100 { format!("{}...", &text[..100]) } else { text.to_string() });
            
            // Parse JSON string into JSON object
            let parsed: Value = serde_json::from_str(text)
                .with_context(|| format!("Failed to parse JSON for variable {}: {}", var, 
                    if text.len() > 200 { format!("{}...", &text[..200]) } else { text.to_string() }))?;
            
            vars.insert(var.to_string(), parsed.clone());
            println!("[CHECK] JSON_OUTPUT {}: Successfully parsed JSON string into object: {}", 
                     var, parsed);
        } else {
            // Already a JSON object, no need to parse
            println!("[CHECK] JSON_OUTPUT {}: Variable is already a JSON object: {}", var, value);
        }
    } else {
        println!("[CHECK] JSON_OUTPUT {}: ERROR - variable not found in vars!", var);
        println!("[CHECK] JSON_OUTPUT: Available variable names: {:?}", vars.keys().collect::<Vec<_>>());
    }
    Ok(())
}

fn format_return(
    template: &str,
    vars: &HashMap<String, Value>,
    server: &GameServer,
    error: Option<&GameServerError>,
) -> String {
    println!("[CHECK] format_return: Processing template: '{}'", template);
    println!("[CHECK] format_return: Available variables: {:?}", vars.keys().collect::<Vec<_>>());
    
    // Replace error placeholders first
    let mut template = template.to_string();
    if let Some(err) = error {
        template = template.replace("<ERROR REASON>", &err.message);
        template = template.replace("ERROR", &err.message);
    } else {
        template = template.replace("<ERROR REASON>", "");
        template = template.replace("ERROR", "");
    }

    let mut result = String::new();
    let mut token = String::new();
    let mut in_quoted_placeholder = false;
    let mut quoted_token = String::new();
    
    let mut chars = template.chars().peekable();
    
    while let Some(ch) = chars.next() {
        // Handle quoted placeholders like 'HOST' or 'JSON_PAYLOAD.version.protocol'
        if ch == '\'' && !in_quoted_placeholder {
            // Check if this looks like a quoted placeholder (starts with letter or underscore)
            // Also skip consecutive quotes that might be typos
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '\'' {
                    // Double quote - skip this one and check the next
                    continue; // Skip this quote, will check the next iteration
                } else if next_ch.is_ascii_alphabetic() || next_ch == '_' {
                    in_quoted_placeholder = true;
                    continue; // Skip the opening quote
                }
            }
            result.push(ch);
        } else if ch == '\'' && in_quoted_placeholder {
            // End of quoted placeholder, resolve it and skip the closing quote
            if !quoted_token.is_empty() {
                result.push_str(&resolve_token(&quoted_token, vars, server));
                quoted_token.clear();
            }
            in_quoted_placeholder = false;
            continue; // Skip the closing quote
        } else if in_quoted_placeholder {
            // Building quoted token
            if is_token_char(ch) {
                quoted_token.push(ch);
            } else {
                // Not a valid token char, end the quoted placeholder
                if !quoted_token.is_empty() {
                    result.push_str(&resolve_token(&quoted_token, vars, server));
                    quoted_token.clear();
                }
                in_quoted_placeholder = false;
                result.push(ch);
            }
        } else if is_token_char(ch) {
            // Regular unquoted token
            token.push(ch);
        } else {
            // Not a token character, resolve any pending token
            if !token.is_empty() {
                println!("[CHECK] format_return: Found non-token char '{}', resolving token: '{}'", ch, token);
                result.push_str(&resolve_token(&token, vars, server));
                token.clear();
            }
            result.push(ch);
        }
    }
    
    // Handle any remaining tokens
    if !token.is_empty() {
        println!("[CHECK] format_return: Resolving remaining token: '{}'", token);
        result.push_str(&resolve_token(&token, vars, server));
    }
    if !quoted_token.is_empty() {
        println!("[CHECK] format_return: Resolving remaining quoted token: '{}'", quoted_token);
        result.push_str(&resolve_token(&quoted_token, vars, server));
    }
    
    println!("[CHECK] format_return: Final result: '{}'", result);
    result
}

fn resolve_token(token: &str, vars: &HashMap<String, Value>, server: &GameServer) -> String {
    match token {
        "HOST_LEN" | "IP_LEN" => server.address.len().to_string(),
        "HOST" | "IP" => server.address.clone(),
        "PORT" => server.port.to_string(),
        other => {
            // Try to resolve as a variable path (supports dot notation)
            match resolve_var_path(other, vars) {
                Some(value) => {
                    println!("[CHECK] Resolved token '{}' to '{}'", other, value);
                    value
                }
                None => {
                    println!("[CHECK] Token '{}' not found in vars, returning as-is", other);
                    other.to_string()
                }
            }
        }
    }
}

fn resolve_var_path(path: &str, vars: &HashMap<String, Value>) -> Option<String> {
    let mut segments = path.split('.');
    let mut value = vars.get(segments.next()?);
    for segment in segments {
        value = value?.get(segment);
    }
    value.map(value_to_string)
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

fn insert_server_placeholders(vars: &mut HashMap<String, Value>, server: &GameServer) {
    vars.entry("HOST".to_string())
        .or_insert_with(|| Value::String(server.address.clone()));
    vars.entry("IP".to_string())
        .or_insert_with(|| Value::String(server.address.clone()));
    vars.entry("HOST_LEN".to_string())
        .or_insert_with(|| Value::Number(server.address.len().into()));
    vars.entry("IP_LEN".to_string())
        .or_insert_with(|| Value::Number(server.address.len().into()));
    vars.entry("PORT".to_string())
        .or_insert_with(|| Value::Number(server.port.into()));
}

fn strip_placeholder_vars(vars: &mut HashMap<String, Value>) {
    for key in &["HOST", "IP", "HOST_LEN", "IP_LEN", "IP_LEN_HEX", "PORT"] {
        vars.remove(*key);
    }
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_' || ch == '.'
}

fn replace_placeholders(code: &str, server: &GameServer) -> String {
    let host = server.address.clone();
    let host_len = host.len();
    let ip_len_hex = format!("{:X}", host_len);
    let mut replaced = code.replace("IP_LEN_HEX", &ip_len_hex);
    replaced = replaced.replace("HOST_LEN", &host_len.to_string());
    replaced = replaced.replace("IP_LEN", &host_len.to_string());
    replaced = replaced.replace("PORT", &server.port.to_string());
    replaced = replaced.replace("IP", &host);
    replaced = replaced.replace("HOST", &host);
    replaced
}

