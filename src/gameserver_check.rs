use crate::models::{GameServer, Protocol, GameServerTestResult, GameServerError};
use crate::packet_parser::{build_packets, parse_response, parse_script, execute_code_blocks, OutputBlock, OutputCommand, OutputStatus};
use anyhow::{Context, Result};
use serde_json::Value;
use indexmap::IndexMap;
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
                variables: serde_json::json!({}),
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
                variables: serde_json::json!({}),
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
    let mut all_parsed_vars = IndexMap::new();
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
                        variables: serde_json::json!({}),
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
            // Create TCP connection and manage it per pair (may be closed/reopened)
            use tokio::net::TcpStream;
            use tokio::time::{timeout, Duration};
            
            let addr = format!("{}:{}", server.address, server.port);
            let timeout_duration = Duration::from_millis(server.timeout_ms);
            
            let mut stream: Option<TcpStream> = None;
            let mut results = Vec::new();
            
            // Track packet index across all pairs
            let mut global_packet_idx = 0;
            
            for (pair_idx, pair) in script.pairs.iter().enumerate() {
                println!("[CHECK] Executing pair {} of {} ({} packet(s))...", pair_idx + 1, script.pairs.len(), pair.packets.len());
                
                // Check if we need to close connection before this pair
                if pair.close_connection_before {
                    if stream.take().is_some() {
                        println!("[TCP] Closing connection before pair {}", pair_idx + 1);
                        // Connection is closed when dropped
                    }
                }
                
                // Check if we need to open a new connection
                if stream.is_none() {
                    println!("[TCP] Connecting to {} (timeout: {}ms)...", addr, server.timeout_ms);
                    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
                        Ok(Ok(s)) => {
                            stream = Some(s);
                            println!("[TCP] Connected successfully");
                        },
                        Ok(Err(e)) => {
                            results.push(Err(anyhow::anyhow!("Failed to connect to server: {}", e)));
                            break;
                        },
                        Err(_) => {
                            results.push(Err(anyhow::anyhow!("Connection timeout")));
                            break;
                        }
                    }
                }
                
                // Send all packets for this pair (without waiting for responses)
                match stream.as_mut() {
                    Some(s) => {
                        for (packet_in_pair_idx, _packet_commands) in pair.packets.iter().enumerate() {
                            let packet = &built_packets[global_packet_idx];
                            println!("[TCP] Sending packet {} of pair {} (packet {} total)...", 
                                     packet_in_pair_idx + 1, pair_idx + 1, global_packet_idx + 1);
                            match send_packet_tcp_no_response(s, packet).await {
                                Ok(_) => {
                                    println!("[TCP] Packet {} sent successfully", global_packet_idx + 1);
                                },
                                Err(e) => {
                                    results.push(Err(anyhow::anyhow!("Failed to send packet {}: {}", global_packet_idx + 1, e)));
                                    stream = None; // Connection is likely broken
                                    break;
                                }
                            }
                            global_packet_idx += 1;
                        }
                        
                        // After all packets are sent, wait for response (only if there's a response defined)
                        if !pair.response.is_empty() {
                            if let Some(s) = stream.as_mut() {
                                println!("[TCP] All packets for pair {} sent, waiting for response...", pair_idx + 1);
                                match receive_packet_tcp(s, timeout_duration).await {
                                    Ok(response) => {
                                        println!("[TCP] Response received: {} bytes", response.len());
                                        results.push(Ok(response));
                                    },
                                    Err(e) => {
                                        results.push(Err(e));
                                        stream = None; // Connection is likely broken
                                        break;
                                    }
                                }
                            }
                        } else {
                            // No response expected, push empty result
                            results.push(Ok(Vec::new()));
                        }
                    },
                    None => {
                        results.push(Err(anyhow::anyhow!("No connection available")));
                        break;
                    }
                }
            }
            
            // Close connection if still open
            if stream.is_some() {
                println!("[TCP] All pairs complete, closing connection");
            }
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

    // Execute code blocks (variables from CODE_START/CODE_END)
    // Do this even if there's an error, so variables are available for error output
    let code_variables = match execute_code_blocks(&script.code_blocks, &mut all_parsed_vars) {
        Ok(vars) => {
            println!("[CHECK] Code blocks executed: {} variables created", vars.len());
            vars
        }
        Err(e) => {
            println!("[CHECK] Code block execution failed: {}", e);
            // Continue anyway, but log the error
            IndexMap::new()
        }
    };

    let code_vars_count = code_variables.len();

    // Merge code variables into parsed vars for output block evaluation
    // Code variables can override parsed vars if they have the same name
    let mut all_vars = all_parsed_vars.clone();
    for (key, value) in code_variables.iter() {
        all_vars.insert(key.clone(), value.clone());
    }

    if let Some(err) = last_error {
        let error_labels = evaluate_output_labels(&script, OutputStatus::Error, &mut all_vars.clone(), server, Some(&err));
        return GameServerTestResult {
            success: false,
            response_time_ms,
            raw_response: Some(raw_response_hex),
            parsed_values: serde_json::json!({}),
            variables: serde_json::json!({}),
            error: Some(err),
            output_labels_success: Vec::new(),
            output_labels_error: error_labels,
        };
    }

    // All pairs succeeded
    let success_labels = evaluate_output_labels(&script, OutputStatus::Success, &mut all_vars.clone(), server, None);
    strip_placeholder_vars(&mut all_parsed_vars);
    let parsed_values: serde_json::Value = all_parsed_vars.clone().into_iter().collect();
    let variables: serde_json::Value = code_variables.into_iter().collect();

    println!("[CHECK] All pairs executed successfully: {} parsed values, {} code variables in {}ms", 
             all_parsed_vars.len(), code_vars_count, response_time_ms);
    GameServerTestResult {
        success: true,
        response_time_ms,
        raw_response: Some(raw_response_hex),
        parsed_values,
        variables,
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

async fn send_packet_udp_no_response(
    socket: &tokio::net::UdpSocket,
    addr: &str,
    packet: &[u8],
) -> Result<()> {
    println!("[UDP] Sending packet ({} bytes) to {}...", packet.len(), addr);
    socket
        .send_to(packet, addr)
        .await
        .context("Failed to send UDP packet")?;
    println!("[UDP] Packet sent successfully");
    Ok(())
}

async fn receive_packet_udp(
    socket: &tokio::net::UdpSocket,
    timeout_ms: u64,
) -> Result<Vec<u8>> {
    use tokio::time::{timeout, Duration};

    println!("[UDP] Waiting for response (timeout: {}ms)...", timeout_ms);
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
    send_packet_udp_no_response(socket, addr, packet).await?;
    receive_packet_udp(socket, timeout_ms).await
}

async fn send_packet_tcp_no_response(
    stream: &mut tokio::net::TcpStream,
    packet: &[u8],
) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    println!("[TCP] Sending packet ({} bytes)...", packet.len());
    stream.write_all(packet)
        .await
        .context("Failed to write packet")?;
    println!("[TCP] Packet sent successfully");
    Ok(())
}

async fn receive_packet_tcp(
    stream: &mut tokio::net::TcpStream,
    timeout_duration: tokio::time::Duration,
) -> Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    use tokio::time::timeout;

    println!("[TCP] Waiting for response...");
    let mut buf = vec![0u8; 16384];
    let size = timeout(timeout_duration, stream.read(&mut buf))
        .await
        .context("Read timeout")?
        .context("Failed to read response")?;
    println!("[TCP] Response received: {} bytes", size);
    Ok(buf[..size].to_vec())
}

async fn send_packet_tcp(
    stream: &mut tokio::net::TcpStream,
    packet: &[u8],
    timeout_duration: tokio::time::Duration,
) -> Result<Vec<u8>> {
    send_packet_tcp_no_response(stream, packet).await?;
    receive_packet_tcp(stream, timeout_duration).await
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
    vars: &mut IndexMap<String, Value>,
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
    vars: &mut IndexMap<String, Value>,
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
    vars: &mut IndexMap<String, Value>,
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

fn handle_json_output(var: &str, vars: &mut IndexMap<String, Value>) -> Result<()> {
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
    vars: &IndexMap<String, Value>,
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

    // Remove outer quotes if present (for quoted strings)
    let mut template_str = template.trim();
    let mut was_quoted = false;
    if (template_str.starts_with('"') && template_str.ends_with('"')) ||
       (template_str.starts_with('\'') && template_str.ends_with('\'')) {
        template_str = &template_str[1..template_str.len() - 1];
        was_quoted = true;
    }

    // Check if the entire template (after removing quotes) is just a variable name
    if is_valid_var_name(template_str) {
        // Entire template is a variable name, output as "varname=value"
        if let Some(value) = resolve_var_path(template_str, vars) {
            let result = format!("{}=\"{}\"", template_str, value);
            println!("[CHECK] format_return: Entire template is variable '{}', result: '{}'", template_str, result);
            return result;
        }
    }

    // Now process the template and substitute variables
    // Support both simple variable names and dot-notation paths (e.g., JSON_PAYLOAD.version.protocol)
    let mut result = String::new();
    let mut current_token = String::new();
    let mut i = 0;
    let chars: Vec<char> = template_str.chars().collect();
    
    while i < chars.len() {
        let ch = chars[i];
        
        if is_token_char(ch) {
            current_token.push(ch);
        } else {
            // Not a token character, resolve any pending token
            if !current_token.is_empty() {
                // Try to resolve as a variable path (supports dot notation)
                // First check if it's a simple variable name, then try as a path
                if is_valid_var_name(&current_token) || current_token.contains('.') {
                    // Try resolving as a variable path (supports dot notation like JSON_PAYLOAD.version.protocol)
                    match resolve_var_path(&current_token, vars) {
                        Some(value) => {
                            println!("[CHECK] format_return: Resolved path '{}' to '{}'", current_token, value);
                            result.push_str(&value);
                        },
                        None => {
                            // Not found as path, try as simple token (for special tokens like HOST, PORT)
                            let resolved = resolve_token(&current_token, vars, server);
                            result.push_str(&resolved);
                        }
                    }
                } else {
                    // Not a variable name or path, output as-is
                    result.push_str(&current_token);
                }
                current_token.clear();
            }
            result.push(ch);
        }
        i += 1;
    }
    
    // Handle any remaining token at the end
    if !current_token.is_empty() {
        // Try to resolve as a variable path (supports dot notation)
        if is_valid_var_name(&current_token) || current_token.contains('.') {
            match resolve_var_path(&current_token, vars) {
                Some(value) => {
                    println!("[CHECK] format_return: Resolved path '{}' to '{}'", current_token, value);
                    result.push_str(&value);
                },
                None => {
                    // Not found as path, try as simple token
                    let resolved = resolve_token(&current_token, vars, server);
                    result.push_str(&resolved);
                }
            }
        } else {
            // Not a variable name or path, output as-is
            result.push_str(&current_token);
        }
    }
    
    // If it was originally quoted, return as quoted string
    if was_quoted {
        result = format!("\"{}\"", result);
    }
    
    println!("[CHECK] format_return: Final result: '{}'", result);
    result
}

fn is_valid_var_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    // First character must be letter or underscore
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
    }
    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn resolve_token(token: &str, vars: &IndexMap<String, Value>, server: &GameServer) -> String {
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

fn resolve_var_path(path: &str, vars: &IndexMap<String, Value>) -> Option<String> {
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

fn insert_server_placeholders(vars: &mut IndexMap<String, Value>, server: &GameServer) {
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

fn strip_placeholder_vars(vars: &mut IndexMap<String, Value>) {
    for key in &["HOST", "IP", "HOST_LEN", "IP_LEN", "IP_LEN_HEX", "PORT"] {
        vars.shift_remove(*key);
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

