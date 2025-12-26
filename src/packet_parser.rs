use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub enum PacketCommand {
    WriteByte(u8),
    WriteShort(u16, bool), // value, big_endian
    WriteInt(u32, bool),   // value, big_endian
    WriteIntVar(String, bool), // variable name, big_endian - resolved at build time
    WriteShortVar(String, bool), // variable name, big_endian - resolved at build time
    WriteByteVar(String), // variable name - resolved at build time
    WriteVarIntVar(String), // variable name - resolved at build time
    WriteString(String, Option<usize>), // value, optional fixed length
    WriteStringVar(String, Option<usize>), // variable name, optional fixed length - resolved at build time
    WriteBytes(Vec<u8>),
    WriteVarInt(u64),
    WriteVarIntLen,
    WriteIntLen(bool), // big_endian flag for length placeholder
}

#[derive(Debug, Clone)]
pub enum ResponseCommand {
    ReadByte(String),
    ReadShort(String, bool), // var_name, big_endian
    ReadInt(String, bool),   // var_name, big_endian
    ReadString(String, Option<usize>), // var_name, optional fixed length
    ReadStringNull(String),
    SkipBytes(usize),
    ExpectByte(u8),
    ExpectMagic(Vec<u8>),
    ReadVarInt(String),
    // HTTP-specific response commands
    ExpectStatus(u16),
    ExpectHeader { key: String, value: String },
    ReadBodyJson(String),
    ReadBody(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputStatus {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub enum OutputCommand {
    JsonOutput(String),
    Return(String),
}

#[derive(Debug, Clone)]
pub struct OutputBlock {
    pub status: OutputStatus,
    pub commands: Vec<OutputCommand>,
}

#[derive(Debug, Clone)]
pub enum VariableType {
    String,
    Int,
    Byte,
    Float,
    Array,
}

#[derive(Debug, Clone)]
pub enum CodeCommand {
    // Variable declarations
    DeclareVar {
        var_type: VariableType,
        name: String,
        value: Expression,
    },
    // Assign to existing variable
    AssignVar {
        name: String,
        value: Expression,
    },
    // Control flow
    ForLoop {
        var_name: String,
        range_start: Expression,
        range_end: Expression,
        body: Vec<CodeCommand>,
    },
    ForInArray {
        var_name: String,
        array_name: String,
        body: Vec<CodeCommand>,
    },
    IfStatement {
        condition: Condition,
        body: Vec<CodeCommand>,
        else_if: Vec<(Condition, Vec<CodeCommand>)>,
        else_body: Option<Vec<CodeCommand>>,
    },
    // String functions
    Split {
        var_name: String,
        source_expr: Expression,
        delimiter: String,
    },
    Replace {
        var_name: String,
        source_expr: Expression,
        search: String,
        replace: String,
    },
    // Control flow
    Break,
    // Execute packet/response commands (nested)
    ExecutePacketCommand(PacketCommand),
    ExecuteResponseCommand(ResponseCommand),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(JsonValue),
    Variable(String),
    ArrayIndex {
        array_name: String,
        index: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Condition {
    Equals(Expression, Expression),
    NotEquals(Expression, Expression),
    GreaterThan(Expression, Expression),
    LessThan(Expression, Expression),
    GreaterOrEqual(Expression, Expression),
    LessOrEqual(Expression, Expression),
    Contains(Expression, Expression), // string contains substring
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum HttpBodyType {
    Form,
    Raw,
}

#[derive(Debug, Clone)]
pub enum HttpCommand {
    HttpStart { method: HttpMethod, path: String },
    Param { key: String, value: String },
    Header { key: String, value: String },
    BodyStart { body_type: HttpBodyType },
    Data { content: String },
    BodyEnd,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub params: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body_type: Option<HttpBodyType>,
    pub body_data: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub commands: Vec<CodeCommand>,
}

#[derive(Debug, Clone)]
pub struct PacketResponsePair {
    pub packets: Vec<Vec<PacketCommand>>, // Binary packets (empty if HTTP request is used)
    pub http_request: Option<HttpRequest>, // HTTP request (None if binary packets are used)
    pub response: Vec<ResponseCommand>,
    pub close_connection_before: bool, // If true, close connection before this pair
}

#[derive(Debug)]
pub struct PacketScript {
    pub pairs: Vec<PacketResponsePair>,
    pub output_blocks: Vec<OutputBlock>,
    pub code_blocks: Vec<CodeBlock>,
}

pub fn parse_script(script: &str) -> Result<PacketScript> {
    println!("[PARSER] Starting script parsing...");
    let lines: Vec<&str> = script.lines().collect();
    let mut pairs = Vec::new();
    let mut current_packets = Vec::new(); // Accumulate multiple packets
    let mut current_packet = Vec::new(); // Current packet being built
    let mut current_http_request: Option<HttpRequest> = None; // Current HTTP request being built
    let mut current_http_commands = Vec::new(); // HTTP commands for current request
    let mut current_response = Vec::new();
    let mut output_blocks = Vec::new();
    let mut current_output: Option<OutputBlock> = None;
    let mut code_blocks = Vec::new();
    let mut current_code = Vec::new();
    let mut in_packet = false;
    let mut in_http = false;
    let mut in_response = false;
    let mut in_code = false;
    let mut close_connection_before_next = false; // Track if CONNECTION_CLOSE was seen

    let mut line_num = 0;
    let mut processed_lines = std::collections::HashSet::new();
    
    while line_num < lines.len() {
        if processed_lines.contains(&line_num) {
            line_num += 1;
            continue;
        }
        
        let line = lines[line_num].trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            if line.starts_with('#') {
                println!("[PARSER] Line {}: Comment skipped: {}", line_num + 1, line);
            }
            line_num += 1;
            continue;
        }

        // Connection close command
        if line == "CONNECTION_CLOSE" {
            println!("[PARSER] Line {}: CONNECTION_CLOSE command", line_num + 1);
            close_connection_before_next = true;
            line_num += 1;
            continue;
        }

        // HTTP section
        if line.starts_with("HTTP_START REQUEST ") {
            println!("[PARSER] Line {}: Entering HTTP_START section", line_num + 1);
            
            // Parse HTTP_START REQUEST <METHOD> <PATH>
            let rest = line.strip_prefix("HTTP_START REQUEST ").unwrap();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() < 2 {
                anyhow::bail!("HTTP_START REQUEST requires method and path at line {}", line_num + 1);
            }
            
            let method_str = parts[0].to_uppercase();
            let method = match method_str.as_str() {
                "GET" => HttpMethod::Get,
                "POST" => HttpMethod::Post,
                "PUT" => HttpMethod::Put,
                "DELETE" => HttpMethod::Delete,
                _ => {
                    // Check if it's "Custom <method> <path>" format
                    if method_str == "CUSTOM" && parts.len() >= 3 {
                        HttpMethod::Custom(parts[1].to_string())
                    } else {
                        HttpMethod::Custom(parts[0].to_string())
                    }
                },
            };
            let path = if method_str == "CUSTOM" && parts.len() >= 3 {
                parts[2].to_string()
            } else {
                parts[1].to_string()
            };
            
            current_http_request = Some(HttpRequest {
                method,
                path,
                params: Vec::new(),
                headers: Vec::new(),
                body_type: None,
                body_data: Vec::new(),
            });
            current_http_commands.clear();
            in_http = true;
            in_packet = false;
            in_response = false;
            line_num += 1;
            continue;
        }
        if line == "HTTP_END" {
            println!("[PARSER] Line {}: Exiting HTTP section", line_num + 1);
            // Build the HTTP request from accumulated commands
            if let Some(mut http_req) = current_http_request.take() {
                http_req = build_http_request_from_commands(http_req, &current_http_commands)?;
                current_http_request = Some(http_req);
            }
            in_http = false;
            line_num += 1;
            continue;
        }
        
        // Packet section
        if line == "PACKET_START" {
            println!("[PARSER] Line {}: Entering PACKET_START section", line_num + 1);
            // If we were already in a packet, save the current packet to the packets list
            if in_packet && !current_packet.is_empty() {
                println!("[PARSER] Saving packet with {} commands to packets list", current_packet.len());
                current_packets.push(current_packet.clone());
                current_packet.clear();
            }
            // Mark this new pair to close connection before it if CONNECTION_CLOSE was seen
            let should_close = close_connection_before_next;
            close_connection_before_next = false; // Reset flag
            if should_close {
                println!("[PARSER] This PACKET_START will close connection before sending");
            }
            in_packet = true;
            in_http = false;
            in_response = false;
            line_num += 1;
            continue;
        }
        if line == "PACKET_END" {
            println!("[PARSER] Line {}: Exiting PACKET section (found {} commands)", line_num + 1, current_packet.len());
            if !current_packet.is_empty() {
                // Save this packet to the packets list
                current_packets.push(current_packet.clone());
                current_packet.clear();
                println!("[PARSER] Packet saved, total packets in current group: {}", current_packets.len());
            }
            in_packet = false;
            line_num += 1;
            continue;
        }
        
        // Response section
        if line == "RESPONSE_START" {
            println!("[PARSER] Line {}: Entering RESPONSE_START section", line_num + 1);
            in_response = true;
            in_packet = false;
            in_code = false;
            line_num += 1;
            continue;
        }
        if line == "RESPONSE_END" {
            println!("[PARSER] Line {}: Exiting RESPONSE section (found {} commands)", line_num + 1, current_response.len());
            // When response ends, save all accumulated packets or HTTP request with the response
            let should_close = close_connection_before_next;
            close_connection_before_next = false; // Reset flag
            
            if !current_packets.is_empty() {
                println!("[PARSER] Saving packet/response pair ({} packet(s), response: {} commands)", 
                         current_packets.len(), current_response.len());
                if should_close {
                    println!("[PARSER] This pair will close connection before sending");
                }
                pairs.push(PacketResponsePair {
                    packets: current_packets.clone(),
                    http_request: None,
                    response: current_response.clone(),
                    close_connection_before: should_close,
                });
                current_packets.clear();
            } else if current_http_request.is_some() {
                let mut http_req = current_http_request.take().unwrap();
                http_req = build_http_request_from_commands(http_req, &current_http_commands)?;
                println!("[PARSER] Saving HTTP request/response pair (response: {} commands)", current_response.len());
                if should_close {
                    println!("[PARSER] This pair will close connection before sending");
                }
                pairs.push(PacketResponsePair {
                    packets: Vec::new(),
                    http_request: Some(http_req),
                    response: current_response.clone(),
                    close_connection_before: should_close,
                });
                current_http_commands.clear();
            }
            current_response.clear();
            in_response = false;
            line_num += 1;
            continue;
        }

        // Code section
        if line == "CODE_START" {
            println!("[PARSER] Line {}: Entering CODE_START section", line_num + 1);
            // If we have accumulated packets but no response yet, we need to save them first
            // This can happen if CODE_START appears after PACKET_END but before RESPONSE_START
            if !current_packets.is_empty() && current_response.is_empty() {
                println!("[PARSER] WARNING: CODE_START encountered with {} accumulated packet(s) but no response. Packets will be lost!", current_packets.len());
                current_packets.clear();
            }
            in_code = true;
            in_packet = false;
            in_response = false;
            current_code.clear();
            line_num += 1;
            continue;
        }
        if line == "CODE_END" {
            println!("[PARSER] Line {}: Exiting CODE section (found {} commands)", line_num + 1, current_code.len());
            if !current_code.is_empty() {
                code_blocks.push(CodeBlock {
                    commands: current_code.clone(),
                });
                current_code.clear();
            }
            in_code = false;
            line_num += 1;
            continue;
        }

        if in_http {
            println!("[PARSER] Line {}: Parsing HTTP command: {}", line_num + 1, line);
            let cmd = parse_http_command(line, line_num + 1)?;
            current_http_commands.push(cmd);
            line_num += 1;
        } else if in_packet {
            println!("[PARSER] Line {}: Parsing packet command: {}", line_num + 1, line);
            current_packet.push(parse_packet_command(line, line_num + 1)?);
            line_num += 1;
        } else if in_response {
            println!("[PARSER] Line {}: Parsing response command: {}", line_num + 1, line);
            current_response.push(parse_response_command(line, line_num + 1)?);
            line_num += 1;
        } else if in_code {
            println!("[PARSER] Line {}: Parsing code command: {}", line_num + 1, line);
            let indent_level = lines[line_num].len() - lines[line_num].trim_start().len();
            
            if line.ends_with(':') && (line.starts_with("FOR ") || line.starts_with("IF ")) {
                // Parse multi-line control flow statement
                let (cmd, lines_consumed) = parse_control_flow(&lines, line_num, indent_level)?;
                current_code.push(cmd);
                // Mark all consumed lines as processed
                for i in 0..lines_consumed {
                    processed_lines.insert(line_num + i);
                }
                line_num += lines_consumed;
            } else if indent_level > 0 {
                // This is an indented line, skip it (it's part of a control flow body we already parsed)
                line_num += 1;
            } else {
                current_code.push(parse_code_command(line, line_num + 1)?);
                line_num += 1;
            }
        } else {
            handle_output_line(line, line_num + 1, &mut current_output, &mut output_blocks)?;
            line_num += 1;
        }
    }

    // Save any remaining packet/response pair
    if !current_packets.is_empty() {
        println!("[PARSER] Saving final packet/response pair ({} packet(s), response: {} commands)", 
                 current_packets.len(), current_response.len());
        pairs.push(PacketResponsePair {
            packets: current_packets,
            http_request: None,
            response: current_response,
            close_connection_before: close_connection_before_next,
        });
    } else if current_http_request.is_some() {
        let mut http_req = current_http_request.take().unwrap();
        http_req = build_http_request_from_commands(http_req, &current_http_commands)?;
        println!("[PARSER] Saving final HTTP request/response pair (response: {} commands)", current_response.len());
        pairs.push(PacketResponsePair {
            packets: Vec::new(),
            http_request: Some(http_req),
            response: current_response,
            close_connection_before: close_connection_before_next,
        });
    }

    if let Some(block) = current_output.take() {
        output_blocks.push(block);
    }

    // Save any remaining code block
    if !current_code.is_empty() {
        code_blocks.push(CodeBlock {
            commands: current_code,
        });
    }

    println!("[PARSER] Script parsing complete: {} packet/response pair(s), {} code block(s)", pairs.len(), code_blocks.len());
    Ok(PacketScript {
        pairs,
        output_blocks,
        code_blocks,
    })
}

fn parse_packet_command(line: &str, line_num: usize) -> Result<PacketCommand> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty command at line {}", line_num);
    }

    match parts[0] {
        "WRITE_BYTE" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_BYTE requires value at line {}", line_num))?;
            if is_variable_name(token) {
                Ok(PacketCommand::WriteByteVar(token.to_string()))
            } else {
                let value = parse_byte_value(Some(token))?;
                Ok(PacketCommand::WriteByte(value))
            }
        }
        "WRITE_SHORT" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_SHORT requires value at line {}", line_num))?;
            if is_variable_name(token) {
                Ok(PacketCommand::WriteShortVar(token.to_string(), false))
            } else {
                let value = parse_short_value(Some(token))?;
                Ok(PacketCommand::WriteShort(value, false))
            }
        }
        "WRITE_SHORT_BE" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_SHORT_BE requires value at line {}", line_num))?;
            if is_variable_name(token) {
                Ok(PacketCommand::WriteShortVar(token.to_string(), true))
            } else {
                let value = parse_short_value(Some(token))?;
                Ok(PacketCommand::WriteShort(value, true))
            }
        }
        "WRITE_INT" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_INT requires value at line {}", line_num))?;
            if token.eq_ignore_ascii_case("PACKET_LEN") {
                Ok(PacketCommand::WriteIntLen(false)) // little-endian by default
            } else if is_variable_name(token) {
                Ok(PacketCommand::WriteIntVar(token.to_string(), false))
            } else {
                let value = parse_int_value(Some(token))?;
                Ok(PacketCommand::WriteInt(value, false))
            }
        }
        "WRITE_INT_BE" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_INT_BE requires value at line {}", line_num))?;
            if token.eq_ignore_ascii_case("PACKET_LEN") {
                Ok(PacketCommand::WriteIntLen(true)) // big-endian
            } else if is_variable_name(token) {
                Ok(PacketCommand::WriteIntVar(token.to_string(), true))
            } else {
                let value = parse_int_value(Some(token))?;
                Ok(PacketCommand::WriteInt(value, true))
            }
        }
        "WRITE_STRING" => {
            // Handle quoted strings with spaces by finding the closing quote
            if let Some(rest) = line.strip_prefix("WRITE_STRING ") {
                if let Some(quote_start) = rest.find('"') {
                    // Find the closing quote after the opening one
                    if let Some(quote_end) = rest[quote_start + 1..].find('"') {
                        let text = rest[quote_start + 1..quote_start + 1 + quote_end].to_string();
                        Ok(PacketCommand::WriteString(text, None))
                    } else {
                        // Fallback: treat rest as unquoted string
                        let text = parse_string_value(Some(rest))?;
                        Ok(PacketCommand::WriteString(text, None))
                    }
                } else {
                    // No quotes, check if it's a variable name
                    let token = rest.trim();
                    if is_variable_name(token) {
                        Ok(PacketCommand::WriteStringVar(token.to_string(), None))
                    } else {
                        let text = parse_string_value(Some(token))?;
                        Ok(PacketCommand::WriteString(text, None))
                    }
                }
            } else {
                anyhow::bail!("WRITE_STRING requires text at line {}", line_num);
            }
        }
        "WRITE_STRING_LEN" => {
            // Handle quoted strings with spaces by finding the closing quote
            if let Some(rest) = line.strip_prefix("WRITE_STRING_LEN ") {
                // Find the opening quote
                if let Some(quote_start) = rest.find('"') {
                    // Find the closing quote after the opening one
                    if let Some(quote_end) = rest[quote_start + 1..].find('"') {
                        let text = rest[quote_start + 1..quote_start + 1 + quote_end].to_string();
                        // Get the length value after the closing quote
                        let length_part = rest[quote_start + 1 + quote_end + 1..].trim();
                        let length: usize = length_part.split_whitespace().next()
                            .ok_or_else(|| anyhow::anyhow!("WRITE_STRING_LEN requires length at line {}", line_num))?
                            .parse()
                            .with_context(|| format!("Invalid length at line {}", line_num))?;
                        Ok(PacketCommand::WriteString(text, Some(length)))
                    } else {
                        anyhow::bail!("Unclosed string in WRITE_STRING_LEN at line {}", line_num);
                    }
                } else {
                    // Fallback to old parsing if no quotes
                    if parts.len() < 3 {
                        anyhow::bail!("WRITE_STRING_LEN requires text and length at line {}", line_num);
                    }
                    let text = parse_string_value(parts.get(1).copied())?;
                    let length: usize = parts[2].parse()
                        .with_context(|| format!("Invalid length at line {}", line_num))?;
                    Ok(PacketCommand::WriteString(text, Some(length)))
                }
            } else {
                anyhow::bail!("WRITE_STRING_LEN requires text and length at line {}", line_num);
            }
        }
        "WRITE_VARINT" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_VARINT requires value at line {}", line_num))?;
            if token.eq_ignore_ascii_case("PACKET_LEN") {
                Ok(PacketCommand::WriteVarIntLen)
            } else if is_variable_name(token) {
                Ok(PacketCommand::WriteVarIntVar(token.to_string()))
            } else {
                let value = parse_literal_value(token)
                    .with_context(|| format!("Invalid varint value at line {}", line_num))?;
                Ok(PacketCommand::WriteVarInt(value))
            }
        }
        "WRITE_BYTES" => {
            let hex = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_BYTES requires hex string at line {}", line_num))?;
            let bytes = hex::decode(hex.replace("0x", "").replace("0X", ""))
                .with_context(|| format!("Invalid hex string at line {}", line_num))?;
            Ok(PacketCommand::WriteBytes(bytes))
        }
        _ => anyhow::bail!("Unknown packet command: {} at line {}", parts[0], line_num),
    }
}

fn parse_response_command(line: &str, line_num: usize) -> Result<ResponseCommand> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty command at line {}", line_num);
    }

    match parts[0] {
        "READ_BYTE" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_BYTE requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadByte(var.to_string()))
        }
        "READ_SHORT" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_SHORT requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadShort(var.to_string(), false))
        }
        "READ_SHORT_BE" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_SHORT_BE requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadShort(var.to_string(), true))
        }
        "READ_INT" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_INT requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadInt(var.to_string(), false))
        }
        "READ_INT_BE" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_INT_BE requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadInt(var.to_string(), true))
        }
        "READ_STRING" => {
            if parts.len() < 3 {
                anyhow::bail!("READ_STRING requires variable name and length at line {}", line_num);
            }
            let var = parts[1].to_string();
            let length: usize = parts[2].parse()
                .with_context(|| format!("Invalid length at line {}", line_num))?;
            Ok(ResponseCommand::ReadString(var, Some(length)))
        }
        "READ_STRING_NULL" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_STRING_NULL requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadStringNull(var.to_string()))
        }
        "READ_VARINT" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_VARINT requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadVarInt(var.to_string()))
        }
        "SKIP_BYTES" => {
            let count: usize = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("SKIP_BYTES requires count at line {}", line_num))?
                .parse()
                .with_context(|| format!("Invalid count at line {}", line_num))?;
            Ok(ResponseCommand::SkipBytes(count))
        }
        "EXPECT_BYTE" => {
            let value = parse_byte_value(parts.get(1).copied())?;
            Ok(ResponseCommand::ExpectByte(value))
        }
        "EXPECT_MAGIC" => {
            let hex = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("EXPECT_MAGIC requires hex string at line {}", line_num))?;
            let bytes = hex::decode(hex.replace("0x", "").replace("0X", ""))
                .with_context(|| format!("Invalid hex string at line {}", line_num))?;
            Ok(ResponseCommand::ExpectMagic(bytes))
        }
        "EXPECT_STATUS" => {
            let status_code: u16 = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("EXPECT_STATUS requires status code at line {}", line_num))?
                .parse()
                .with_context(|| format!("Invalid status code at line {}", line_num))?;
            Ok(ResponseCommand::ExpectStatus(status_code))
        }
        "EXPECT_HEADER" => {
            if parts.len() < 3 {
                anyhow::bail!("EXPECT_HEADER requires header key and value at line {}", line_num);
            }
            let key = parts[1].to_string();
            let value = parts[2..].join(" "); // Handle values with spaces
            Ok(ResponseCommand::ExpectHeader { key, value })
        }
        "READ_BODY_JSON" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_BODY_JSON requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadBodyJson(var.to_string()))
        }
        "READ_BODY" => {
            let var = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("READ_BODY requires variable name at line {}", line_num))?;
            Ok(ResponseCommand::ReadBody(var.to_string()))
        }
        _ => anyhow::bail!("Unknown response command: {} at line {}", parts[0], line_num),
    }
}

fn parse_http_command(line: &str, line_num: usize) -> Result<HttpCommand> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty HTTP command at line {}", line_num);
    }

    match parts[0] {
        "PARAM" => {
            if parts.len() < 3 {
                anyhow::bail!("PARAM requires key and value at line {}", line_num);
            }
            let key = parts[1].to_string();
            let value = parts[2..].join(" "); // Handle values with spaces
            Ok(HttpCommand::Param { key, value })
        }
        "HEADER" => {
            if parts.len() < 3 {
                anyhow::bail!("HEADER requires key and value at line {}", line_num);
            }
            let key = parts[1].to_string();
            let value = parts[2..].join(" "); // Handle values with spaces
            Ok(HttpCommand::Header { key, value })
        }
        "BODY_START" => {
            if parts.len() < 3 || parts[1] != "TYPE" {
                anyhow::bail!("BODY_START requires TYPE and body type (FORM or RAW) at line {}", line_num);
            }
            let body_type_str = parts[2].to_uppercase();
            let body_type = match body_type_str.as_str() {
                "FORM" => HttpBodyType::Form,
                "RAW" => HttpBodyType::Raw,
                _ => anyhow::bail!("BODY_START TYPE must be FORM or RAW at line {}", line_num),
            };
            Ok(HttpCommand::BodyStart { body_type })
        }
        "DATA" => {
            // DATA content can span the rest of the line, and may include JSON or other content
            // For JSON, we need to handle multiline JSON objects. For now, handle single line.
            // The content is the rest of the line after "DATA "
            if parts.len() < 2 {
                anyhow::bail!("DATA requires content at line {}", line_num);
            }
            let content = parts[1..].join(" "); // Join all remaining parts
            Ok(HttpCommand::Data { content })
        }
        "BODY_END" => {
            Ok(HttpCommand::BodyEnd)
        }
        _ => anyhow::bail!("Unknown HTTP command: {} at line {}", parts[0], line_num),
    }
}

fn build_http_request_from_commands(
    mut request: HttpRequest,
    commands: &[HttpCommand],
) -> Result<HttpRequest> {
    for cmd in commands {
        match cmd {
            HttpCommand::Param { key, value } => {
                request.params.push((key.clone(), value.clone()));
            }
            HttpCommand::Header { key, value } => {
                request.headers.push((key.clone(), value.clone()));
            }
            HttpCommand::BodyStart { body_type } => {
                request.body_type = Some(body_type.clone());
            }
            HttpCommand::Data { content } => {
                request.body_data.push(content.clone());
            }
            HttpCommand::BodyEnd => {
                // No-op, just marks the end
            }
            HttpCommand::HttpStart { .. } => {
                // Already handled
            }
        }
    }
    Ok(request)
}

fn handle_output_line(
    line: &str,
    line_num: usize,
    current_output: &mut Option<OutputBlock>,
    output_blocks: &mut Vec<OutputBlock>,
) -> Result<()> {
    match line {
        "OUTPUT_SUCCESS" => {
            if current_output.is_some() {
                anyhow::bail!("OUTPUT_SUCCESS without closing previous block at line {}", line_num);
            }
            *current_output = Some(OutputBlock {
                status: OutputStatus::Success,
                commands: Vec::new(),
            });
            Ok(())
        }
        "OUTPUT_ERROR" => {
            if current_output.is_some() {
                anyhow::bail!("OUTPUT_ERROR without closing previous block at line {}", line_num);
            }
            *current_output = Some(OutputBlock {
                status: OutputStatus::Error,
                commands: Vec::new(),
            });
            Ok(())
        }
        "OUTPUT_END" => {
            if let Some(block) = current_output.take() {
                output_blocks.push(block);
                Ok(())
            } else {
                anyhow::bail!("OUTPUT_END without active block at line {}", line_num);
            }
        }
        _ => {
            let block = current_output
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("Output command outside block at line {}", line_num))?;
            block.commands.push(parse_output_command(line, line_num)?);
            Ok(())
        }
    }
}

fn parse_output_command(line: &str, line_num: usize) -> Result<OutputCommand> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("JSON_OUTPUT") {
        let var = rest.trim();
        if var.is_empty() {
            anyhow::bail!("JSON_OUTPUT requires variable name at line {}", line_num);
        }
        return Ok(OutputCommand::JsonOutput(var.to_string()));
    }
    if let Some(rest) = trimmed.strip_prefix("RETURN") {
        let argument = rest.trim();
        if argument.is_empty() {
            anyhow::bail!("RETURN requires value at line {}", line_num);
        }
        return Ok(OutputCommand::Return(strip_quotes(argument)));
    }
    anyhow::bail!("Unknown output command at line {}: {}", line_num, line);
}

fn strip_quotes(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.len() >= 2 {
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            trimmed[1..trimmed.len() - 1].to_string()
        } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

fn parse_function_args(args_str: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut chars = args_str.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = ch;
                    current_arg.push(ch);
                } else if ch == quote_char {
                    in_quotes = false;
                    quote_char = '\0';
                    current_arg.push(ch);
                } else {
                    current_arg.push(ch);
                }
            }
            ',' => {
                if in_quotes {
                    current_arg.push(ch);
                } else {
                    args.push(current_arg.trim().to_string());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }
    
    if !current_arg.is_empty() {
        args.push(current_arg.trim().to_string());
    }
    
    Ok(args)
}

fn find_comment_position(text: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut chars = text.char_indices();
    
    while let Some((pos, ch)) = chars.next() {
        match ch {
            '"' | '\'' => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = ch;
                } else if ch == quote_char {
                    in_quotes = false;
                    quote_char = '\0';
                }
            }
            '#' => {
                if !in_quotes {
                    return Some(pos);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_byte_value(s: Option<&str>) -> Result<u8> {
    let s = s.ok_or_else(|| anyhow::anyhow!("Missing value"))?;
    if s.starts_with("0x") || s.starts_with("0X") {
        u8::from_str_radix(&s[2..], 16)
            .with_context(|| format!("Invalid hex byte: {}", s))
    } else {
        s.parse::<u8>()
            .with_context(|| format!("Invalid byte value: {}", s))
    }
}

fn parse_short_value(s: Option<&str>) -> Result<u16> {
    let s = s.ok_or_else(|| anyhow::anyhow!("Missing value"))?;
    if s.starts_with("0x") || s.starts_with("0X") {
        u16::from_str_radix(&s[2..], 16)
            .with_context(|| format!("Invalid hex short: {}", s))
    } else {
        s.parse::<u16>()
            .with_context(|| format!("Invalid short value: {}", s))
    }
}

fn parse_int_value(s: Option<&str>) -> Result<u32> {
    let s = s.ok_or_else(|| anyhow::anyhow!("Missing value"))?;
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16)
            .with_context(|| format!("Invalid hex int: {}", s))
    } else {
        s.parse::<u32>()
            .with_context(|| format!("Invalid int value: {}", s))
    }
}

fn parse_string_value(s: Option<&str>) -> Result<String> {
    let s = s.ok_or_else(|| anyhow::anyhow!("Missing value"))?;
    // Remove quotes if present
    let s = s.trim_matches('"');
    Ok(s.to_string())
}

fn parse_literal_value(token: &str) -> Result<u64> {
    if token.starts_with("0x") || token.starts_with("0X") {
        u64::from_str_radix(&token[2..], 16)
            .with_context(|| format!("Invalid hex value: {}", token))
    } else {
        token
            .parse::<u64>()
            .with_context(|| format!("Invalid numeric value: {}", token))
    }
}

/// Check if a token looks like a variable name (not a literal value)
/// Variable names start with a letter or underscore and contain only alphanumeric/underscore
fn is_variable_name(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    // Must start with letter or underscore
    let first_char = token.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }
    // All characters must be alphanumeric or underscore
    token.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn parse_code_command(line: &str, line_num: usize) -> Result<CodeCommand> {
    let trimmed = line.trim();
    
    // Handle indented lines (for loops, if statements) - they're handled by the caller
    // This function handles single-line commands
    
    // Check for control flow statements that end with ':'
    if trimmed.ends_with(':') {
        // This is a control flow statement start - will be handled by multi-line parser
        anyhow::bail!("Control flow statements must be handled with proper indentation at line {}", line_num);
    }
    
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty code command at line {}", line_num);
    }
    
    // Variable declarations: TYPE VAR_NAME = VALUE
    // Also handle: TYPE VAR_NAME = SPLIT(...) or TYPE VAR_NAME = REPLACE(...)
    if parts.len() >= 4 && parts[2] == "=" {
        let var_type_str = parts[0].to_uppercase();
        let var_name = parts[1].to_string();
        let mut value_str = parts[3..].join(" ");
        
        // Strip inline comments (everything after # that's not in quotes)
        if let Some(comment_pos) = find_comment_position(&value_str) {
            value_str = value_str[..comment_pos].trim().to_string();
        }
        
        let var_type = match var_type_str.as_str() {
            "STRING" => VariableType::String,
            "INT" => VariableType::Int,
            "BYTE" => VariableType::Byte,
            "FLOAT" => VariableType::Float,
            "ARRAY" => VariableType::Array,
            _ => anyhow::bail!("Unknown variable type: {} at line {}", var_type_str, line_num),
        };
        
        // Check if value_str is a SPLIT function call
        if value_str.trim().starts_with("SPLIT(") && value_str.trim().ends_with(')') {
            let func_call = value_str.trim().strip_prefix("SPLIT").unwrap_or("").trim();
            if let Some(args) = func_call.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                let args_parts = parse_function_args(args)?;
                if args_parts.len() != 2 {
                    anyhow::bail!("SPLIT requires 2 arguments: SPLIT(source_var, 'delimiter') at line {}", line_num);
                }
                let source_expr = parse_expression(&args_parts[0], line_num)?;
                let delimiter = strip_quotes(&args_parts[1]);
                return Ok(CodeCommand::Split {
                    var_name,
                    source_expr,
                    delimiter,
                });
            }
        }
        
        // Check if value_str is a REPLACE function call
        if value_str.trim().starts_with("REPLACE(") && value_str.trim().ends_with(')') {
            let func_call = value_str.trim().strip_prefix("REPLACE").unwrap_or("").trim();
            if let Some(args) = func_call.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                let args_parts = parse_function_args(args)?;
                if args_parts.len() != 3 {
                    anyhow::bail!("REPLACE requires 3 arguments: REPLACE(source_var, 'search', 'replace') at line {}", line_num);
                }
                let source_expr = parse_expression(&args_parts[0], line_num)?;
                let search = strip_quotes(&args_parts[1]);
                let replace = strip_quotes(&args_parts[2]);
                return Ok(CodeCommand::Replace {
                    var_name,
                    source_expr,
                    search,
                    replace,
                });
            }
        }
        
        // Regular expression evaluation
        let value = parse_expression(&value_str, line_num)?;
        return Ok(CodeCommand::DeclareVar {
            var_type,
            name: var_name,
            value,
        });
    }
    
    // Variable assignment: VAR_NAME = VALUE
    if parts.len() >= 3 && parts[1] == "=" {
        let var_name = parts[0].to_string();
        let mut value_str = parts[2..].join(" ");
        
        // Strip inline comments (everything after # that's not in quotes)
        if let Some(comment_pos) = find_comment_position(&value_str) {
            value_str = value_str[..comment_pos].trim().to_string();
        }
        
        let value = parse_expression(&value_str, line_num)?;
        return Ok(CodeCommand::AssignVar {
            name: var_name,
            value,
        });
    }
    
    // SPLIT function: SPLIT(VAR_NAME, 'DELIMITER')
    if parts[0] == "SPLIT" {
        // Parse: SPLIT(VAR_NAME, 'DELIMITER')
        let func_call = trimmed.strip_prefix("SPLIT").unwrap_or("").trim();
        if let Some(args) = func_call.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
            let args_parts = parse_function_args(args)?;
            if args_parts.len() != 2 {
                anyhow::bail!("SPLIT requires 2 arguments: SPLIT(var_name, 'delimiter') at line {}", line_num);
            }
            let source_expr = parse_expression(&args_parts[0], line_num)?;
            let var_name = if let Expression::Variable(name) = &source_expr {
                name.clone()
            } else {
                anyhow::bail!("SPLIT as standalone command requires variable name, not expression at line {}", line_num);
            };
            let delimiter = strip_quotes(&args_parts[1]);
            return Ok(CodeCommand::Split {
                var_name,
                source_expr,
                delimiter,
            });
        }
        anyhow::bail!("Invalid SPLIT syntax at line {}", line_num);
    }
    
    // REPLACE function: REPLACE(VAR_NAME, 'SEARCH', 'REPLACE')
    if parts[0] == "REPLACE" {
        let func_call = trimmed.strip_prefix("REPLACE").unwrap_or("").trim();
        if let Some(args) = func_call.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
            let args_parts = parse_function_args(args)?;
            if args_parts.len() != 3 {
                anyhow::bail!("REPLACE requires 3 arguments: REPLACE(var_name, 'search', 'replace') at line {}", line_num);
            }
            let source_expr = parse_expression(&args_parts[0], line_num)?;
            let var_name = if let Expression::Variable(name) = &source_expr {
                name.clone()
            } else {
                anyhow::bail!("REPLACE as standalone command requires variable name, not expression at line {}", line_num);
            };
            let search = strip_quotes(&args_parts[1]);
            let replace = strip_quotes(&args_parts[2]);
            return Ok(CodeCommand::Replace {
                var_name,
                source_expr,
                search,
                replace,
            });
        }
        anyhow::bail!("Invalid REPLACE syntax at line {}", line_num);
    }
    
    // BREAK command
    if parts[0] == "BREAK" {
        return Ok(CodeCommand::Break);
    }
    
    // Try to parse as packet/response command (for nested execution)
    if let Ok(packet_cmd) = parse_packet_command(line, line_num) {
        return Ok(CodeCommand::ExecutePacketCommand(packet_cmd));
    }
    if let Ok(response_cmd) = parse_response_command(line, line_num) {
        return Ok(CodeCommand::ExecuteResponseCommand(response_cmd));
    }
    
    anyhow::bail!("Unknown code command: {} at line {}", parts[0], line_num);
}

fn parse_control_flow(
    lines: &[&str],
    start_line: usize,
    base_indent: usize,
) -> Result<(CodeCommand, usize)> {
    let line = lines[start_line].trim();
    
    if line.starts_with("FOR ") {
        // FOR var_name IN array_name:
        let rest = line.strip_prefix("FOR ").unwrap_or("").trim();
        if let Some(in_pos) = rest.find(" IN ") {
            let var_name = rest[..in_pos].trim().to_string();
            let array_part = rest[in_pos + 4..].trim();
            if array_part.ends_with(':') {
                let array_name = array_part[..array_part.len() - 1].trim().to_string();
                
                // Parse the indented body
                let body_indent = base_indent + 2; // Assume 2-space indentation
                let (body, lines_consumed) = parse_indented_body(lines, start_line + 1, body_indent)?;
                
                return Ok((CodeCommand::ForInArray {
                    var_name,
                    array_name,
                    body,
                }, lines_consumed + 1));
            }
        }
        anyhow::bail!("Invalid FOR syntax: FOR var_name IN array_name: at line {}", start_line + 1);
    } else if line.starts_with("IF ") {
        // IF condition:
        let rest = line.strip_prefix("IF ").unwrap_or("").trim();
        if rest.ends_with(':') {
            let cond_str = rest[..rest.len() - 1].trim();
            let condition = parse_condition(cond_str, start_line + 1)?;
            
            // Parse the indented body
            let body_indent = base_indent + 2; // Assume 2-space indentation
            let (body, lines_consumed) = parse_indented_body(lines, start_line + 1, body_indent)?;
            
            return Ok((CodeCommand::IfStatement {
                condition,
                body,
                else_if: Vec::new(),
                else_body: None,
            }, lines_consumed + 1));
        }
        anyhow::bail!("Invalid IF syntax: IF condition: at line {}", start_line + 1);
    }
    
    anyhow::bail!("Not a control flow statement at line {}", start_line + 1);
}

fn parse_indented_body(
    lines: &[&str],
    start_line: usize,
    expected_indent: usize,
) -> Result<(Vec<CodeCommand>, usize)> {
    let mut body = Vec::new();
    let mut line_idx = start_line;
    
    while line_idx < lines.len() {
        let line = lines[line_idx];
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            line_idx += 1;
            continue;
        }
        
        // Check indentation
        let indent = line.len() - line.trim_start().len();
        if indent < expected_indent {
            // Less indented, end of body
            break;
        }
        
        // This line is part of the body
        let line_content = line[expected_indent..].trim();
        
        // Check if it's a control flow statement
        if line_content.ends_with(':') && (line_content.starts_with("FOR ") || line_content.starts_with("IF ")) {
            let (cmd, consumed) = parse_control_flow(lines, line_idx, expected_indent)?;
            body.push(cmd);
            line_idx += consumed;
        } else {
            // Regular command
            body.push(parse_code_command(line_content, line_idx + 1)?);
            line_idx += 1;
        }
    }
    
    Ok((body, line_idx - start_line))
}

fn parse_expression(expr: &str, line_num: usize) -> Result<Expression> {
    let expr = expr.trim();
    
    // Check if it's a quoted string
    if expr.starts_with('"') && expr.ends_with('"') {
        let value = strip_quotes(expr);
        return Ok(Expression::Literal(JsonValue::String(value)));
    }
    
    // Check if it's an array literal: [expr1, expr2, ...]
    if expr.starts_with('[') && expr.ends_with(']') {
        let inner = expr[1..expr.len() - 1].trim();
        let elements: Vec<Expression> = if inner.is_empty() {
            Vec::new()
        } else {
            // Parse comma-separated expressions, but handle nested arrays and function calls
            let mut elements = Vec::new();
            let mut current = String::new();
            let mut depth = 0; // Track bracket/paren depth
            let mut in_quotes = false;
            let mut quote_char = '\0';
            
            for ch in inner.chars() {
                match ch {
                    '"' | '\'' => {
                        if !in_quotes {
                            in_quotes = true;
                            quote_char = ch;
                        } else if ch == quote_char {
                            in_quotes = false;
                            quote_char = '\0';
                        }
                        current.push(ch);
                    }
                    '[' | '(' => {
                        if !in_quotes {
                            depth += 1;
                        }
                        current.push(ch);
                    }
                    ']' | ')' => {
                        if !in_quotes {
                            depth -= 1;
                        }
                        current.push(ch);
                    }
                    ',' => {
                        if !in_quotes && depth == 0 {
                            // This comma is a separator
                            if !current.trim().is_empty() {
                                elements.push(parse_expression(current.trim(), line_num)?);
                            }
                            current.clear();
                        } else {
                            current.push(ch);
                        }
                    }
                    _ => {
                        current.push(ch);
                    }
                }
            }
            if !current.trim().is_empty() {
                elements.push(parse_expression(current.trim(), line_num)?);
            }
            elements
        };
        return Ok(Expression::FunctionCall {
            name: "__array_literal__".to_string(),
            args: elements,
        });
    }
    
    // Check if it's a number
    if let Ok(num) = expr.parse::<i64>() {
        return Ok(Expression::Literal(JsonValue::Number(num.into())));
    }
    if let Ok(num) = expr.parse::<f64>() {
        return Ok(Expression::Literal(JsonValue::Number(
            serde_json::Number::from_f64(num).ok_or_else(|| anyhow::anyhow!("Invalid float at line {}", line_num))?
        )));
    }
    
    // Check if it's a hex number
    if expr.starts_with("0x") || expr.starts_with("0X") {
        if let Ok(num) = u64::from_str_radix(&expr[2..], 16) {
            return Ok(Expression::Literal(JsonValue::Number(num.into())));
        }
    }
    
    // Check if it's an array index: var_name[index]
    // This must come after array literal check to avoid conflicts
    if let Some(bracket_pos) = expr.find('[') {
        if expr.ends_with(']') && !expr.starts_with('[') {
            let array_name = expr[..bracket_pos].trim();
            let index_str = expr[bracket_pos + 1..expr.len() - 1].trim();
            
            // Validate array name (alphanumeric and underscores)
            if array_name.chars().all(|c| c.is_alphanumeric() || c == '_') && !array_name.is_empty() {
                let index_expr = parse_expression(index_str, line_num)?;
                return Ok(Expression::ArrayIndex {
                    array_name: array_name.to_string(),
                    index: Box::new(index_expr),
                });
            }
        }
    }
    
    // Check if it's a variable reference
    if expr.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Ok(Expression::Variable(expr.to_string()));
    }
    
    // Check if it's a function call
    if let Some(func_name) = expr.split('(').next() {
        if expr.contains('(') && expr.ends_with(')') {
            let args_str = expr[func_name.len() + 1..expr.len() - 1].trim();
            let args: Vec<Expression> = if args_str.is_empty() {
                Vec::new()
            } else {
                args_str.split(',').map(|a| parse_expression(a.trim(), line_num)).collect::<Result<_>>()?
            };
            return Ok(Expression::FunctionCall {
                name: func_name.trim().to_string(),
                args,
            });
        }
    }
    
    anyhow::bail!("Invalid expression: {} at line {}", expr, line_num);
}

fn parse_condition(cond_str: &str, line_num: usize) -> Result<Condition> {
    let cond_str = cond_str.trim();
    
    // Parse CONTAINS operator (check before other operators to avoid conflicts)
    if cond_str.contains(" CONTAINS ") {
        let parts: Vec<&str> = cond_str.split(" CONTAINS ").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::Contains(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    
    // Parse comparison operators: ==, !=, >, <, >=, <=
    if cond_str.contains("==") {
        let parts: Vec<&str> = cond_str.split("==").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::Equals(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    if cond_str.contains("!=") {
        let parts: Vec<&str> = cond_str.split("!=").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::NotEquals(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    if cond_str.contains(">=") {
        let parts: Vec<&str> = cond_str.split(">=").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::GreaterOrEqual(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    if cond_str.contains("<=") {
        let parts: Vec<&str> = cond_str.split("<=").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::LessOrEqual(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    if cond_str.contains('>') {
        let parts: Vec<&str> = cond_str.split('>').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::GreaterThan(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    if cond_str.contains('<') {
        let parts: Vec<&str> = cond_str.split('<').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            return Ok(Condition::LessThan(
                parse_expression(parts[0], line_num)?,
                parse_expression(parts[1], line_num)?,
            ));
        }
    }
    
    anyhow::bail!("Invalid condition: {} at line {}", cond_str, line_num);
}

/// Helper function to resolve a variable value from the variables map
fn resolve_var_value(vars: &IndexMap<String, JsonValue>, var_name: &str) -> Result<JsonValue> {
    vars.get(var_name)
        .ok_or_else(|| anyhow::anyhow!("Variable '{}' not found in variables map", var_name))
        .cloned()
}

/// Helper function to get a numeric value (u32) from a JSON value
fn get_u32_from_json(value: &JsonValue) -> Result<u32> {
    if let Some(n) = value.as_u64() {
        Ok(n as u32)
    } else if let Some(n) = value.as_i64() {
        Ok(n as u32)
    } else if let Some(s) = value.as_str() {
        // Try parsing as hex or decimal
        if s.starts_with("0x") || s.starts_with("0X") {
            u32::from_str_radix(&s[2..], 16)
                .with_context(|| format!("Invalid hex string: {}", s))
        } else {
            s.parse::<u32>()
                .with_context(|| format!("Invalid number string: {}", s))
        }
    } else {
        anyhow::bail!("Cannot convert value to u32: {:?}", value)
    }
}

/// Helper function to get a numeric value (u16) from a JSON value
fn get_u16_from_json(value: &JsonValue) -> Result<u16> {
    if let Some(n) = value.as_u64() {
        Ok(n as u16)
    } else if let Some(n) = value.as_i64() {
        Ok(n as u16)
    } else if let Some(s) = value.as_str() {
        if s.starts_with("0x") || s.starts_with("0X") {
            u16::from_str_radix(&s[2..], 16)
                .with_context(|| format!("Invalid hex string: {}", s))
        } else {
            s.parse::<u16>()
                .with_context(|| format!("Invalid number string: {}", s))
        }
    } else {
        anyhow::bail!("Cannot convert value to u16: {:?}", value)
    }
}

/// Helper function to get a numeric value (u8) from a JSON value
fn get_u8_from_json(value: &JsonValue) -> Result<u8> {
    if let Some(n) = value.as_u64() {
        Ok(n as u8)
    } else if let Some(n) = value.as_i64() {
        Ok(n as u8)
    } else if let Some(s) = value.as_str() {
        if s.starts_with("0x") || s.starts_with("0X") {
            u8::from_str_radix(&s[2..], 16)
                .with_context(|| format!("Invalid hex string: {}", s))
        } else {
            s.parse::<u8>()
                .with_context(|| format!("Invalid number string: {}", s))
        }
    } else {
        anyhow::bail!("Cannot convert value to u8: {:?}", value)
    }
}

/// Helper function to get a numeric value (u64) from a JSON value
fn get_u64_from_json(value: &JsonValue) -> Result<u64> {
    if let Some(n) = value.as_u64() {
        Ok(n)
    } else if let Some(n) = value.as_i64() {
        Ok(n as u64)
    } else if let Some(s) = value.as_str() {
        if s.starts_with("0x") || s.starts_with("0X") {
            u64::from_str_radix(&s[2..], 16)
                .with_context(|| format!("Invalid hex string: {}", s))
        } else {
            s.parse::<u64>()
                .with_context(|| format!("Invalid number string: {}", s))
        }
    } else {
        anyhow::bail!("Cannot convert value to u64: {:?}", value)
    }
}

pub fn build_packets(script: &PacketScript) -> Result<Vec<Vec<u8>>> {
    build_packets_with_vars(script, &IndexMap::new())
}

pub fn build_packets_with_vars(script: &PacketScript, vars: &IndexMap<String, JsonValue>) -> Result<Vec<Vec<u8>>> {
    println!("[BUILD] Starting packet construction with {} pair(s)", script.pairs.len());
    let mut built_packets = Vec::new();

    for (pair_idx, pair) in script.pairs.iter().enumerate() {
        // Build all packets for this pair
        for (packet_in_pair_idx, packet_commands) in pair.packets.iter().enumerate() {
            println!("[BUILD] Building packet {} (pair {}, packet {}) with {} commands", 
                     built_packets.len() + 1, pair_idx + 1, packet_in_pair_idx + 1, packet_commands.len());
            let packet_idx = built_packets.len() + 1; // For logging compatibility
            let mut packet = Vec::new();
            let mut varint_placeholders = Vec::new();
        let mut int_placeholders = Vec::new(); // (position, big_endian)

        for (idx, cmd) in packet_commands.iter().enumerate() {
            match cmd {
                PacketCommand::WriteByte(v) => {
                    println!("[BUILD] Packet {} Command {}: WRITE_BYTE 0x{:02X} ({})", packet_idx + 1, idx + 1, v, v);
                    packet.push(*v);
                }
                PacketCommand::WriteByteVar(var_name) => {
                    let value = get_u8_from_json(&resolve_var_value(vars, var_name)?)?;
                    println!("[BUILD] Packet {} Command {}: WRITE_BYTE {} (var: {}) = 0x{:02X} ({})", 
                             packet_idx + 1, idx + 1, var_name, var_name, value, value);
                    packet.push(value);
                }
                PacketCommand::WriteShort(v, big_endian) => {
                    let bytes = if *big_endian {
                        v.to_be_bytes()
                    } else {
                        v.to_le_bytes()
                    };
                    println!("[BUILD] Packet {} Command {}: WRITE_SHORT{} {} (bytes: {:02X} {:02X})", 
                             packet_idx + 1, idx + 1, if *big_endian { "_BE" } else { "" }, v, bytes[0], bytes[1]);
                    packet.extend_from_slice(&bytes);
                }
                PacketCommand::WriteShortVar(var_name, big_endian) => {
                    let value = get_u16_from_json(&resolve_var_value(vars, var_name)?)?;
                    let bytes = if *big_endian {
                        value.to_be_bytes()
                    } else {
                        value.to_le_bytes()
                    };
                    println!("[BUILD] Packet {} Command {}: WRITE_SHORT{} {} (var: {}) = {} (bytes: {:02X} {:02X})", 
                             packet_idx + 1, idx + 1, if *big_endian { "_BE" } else { "" }, var_name, var_name, value, bytes[0], bytes[1]);
                    packet.extend_from_slice(&bytes);
                }
                PacketCommand::WriteInt(v, big_endian) => {
                    let bytes = if *big_endian {
                        v.to_be_bytes()
                    } else {
                        v.to_le_bytes()
                    };
                    println!("[BUILD] Packet {} Command {}: WRITE_INT{} {} (bytes: {:02X} {:02X} {:02X} {:02X})", 
                             packet_idx + 1, idx + 1, if *big_endian { "_BE" } else { "" }, v, 
                             bytes[0], bytes[1], bytes[2], bytes[3]);
                    packet.extend_from_slice(&bytes);
                }
                PacketCommand::WriteIntVar(var_name, big_endian) => {
                    let value = get_u32_from_json(&resolve_var_value(vars, var_name)?)?;
                    let bytes = if *big_endian {
                        value.to_be_bytes()
                    } else {
                        value.to_le_bytes()
                    };
                    println!("[BUILD] Packet {} Command {}: WRITE_INT{} {} (var: {}) = {} (bytes: {:02X} {:02X} {:02X} {:02X})", 
                             packet_idx + 1, idx + 1, if *big_endian { "_BE" } else { "" }, var_name, var_name, value,
                             bytes[0], bytes[1], bytes[2], bytes[3]);
                    packet.extend_from_slice(&bytes);
                }
                PacketCommand::WriteString(text, length_opt) => {
                    if let Some(length) = length_opt {
                        let mut bytes = text.as_bytes().to_vec();
                        bytes.resize(*length, 0);
                        println!("[BUILD] Packet {} Command {}: WRITE_STRING_LEN \"{}\" {} ({} bytes, padded)", 
                                 packet_idx + 1, idx + 1, text, length, length);
                        packet.extend_from_slice(&bytes[..*length]);
                    } else {
                        println!("[BUILD] Packet {} Command {}: WRITE_STRING \"{}\" ({} bytes + null terminator)", 
                                 packet_idx + 1, idx + 1, text, text.len());
                        packet.extend_from_slice(text.as_bytes());
                        packet.push(0); // Null terminator
                    }
                }
                PacketCommand::WriteStringVar(var_name, length_opt) => {
                    let value = resolve_var_value(vars, var_name)?;
                    let text = value.as_str()
                        .ok_or_else(|| anyhow::anyhow!("Variable '{}' is not a string", var_name))?;
                    if let Some(length) = length_opt {
                        let mut bytes = text.as_bytes().to_vec();
                        bytes.resize(*length, 0);
                        println!("[BUILD] Packet {} Command {}: WRITE_STRING_LEN {} (var: {}) = \"{}\" {} ({} bytes, padded)", 
                                 packet_idx + 1, idx + 1, var_name, var_name, text, length, length);
                        packet.extend_from_slice(&bytes[..*length]);
                    } else {
                        println!("[BUILD] Packet {} Command {}: WRITE_STRING {} (var: {}) = \"{}\" ({} bytes + null terminator)", 
                                 packet_idx + 1, idx + 1, var_name, var_name, text, text.len());
                        packet.extend_from_slice(text.as_bytes());
                        packet.push(0); // Null terminator
                    }
                }
                PacketCommand::WriteBytes(bytes) => {
                    println!("[BUILD] Packet {} Command {}: WRITE_BYTES {} bytes (hex: {})", 
                             packet_idx + 1, idx + 1, bytes.len(), hex::encode(bytes));
                    packet.extend_from_slice(bytes);
                }
                PacketCommand::WriteVarInt(value) => {
                    let encoded = encode_varint(*value);
                    println!("[BUILD] Packet {} Command {}: WRITE_VARINT {} (bytes: {})",
                             packet_idx + 1, idx + 1, value, hex::encode(&encoded));
                    packet.extend_from_slice(&encoded);
                }
                PacketCommand::WriteVarIntVar(var_name) => {
                    let value = get_u64_from_json(&resolve_var_value(vars, var_name)?)?;
                    let encoded = encode_varint(value);
                    println!("[BUILD] Packet {} Command {}: WRITE_VARINT {} (var: {}) = {} (bytes: {})",
                             packet_idx + 1, idx + 1, var_name, var_name, value, hex::encode(&encoded));
                    packet.extend_from_slice(&encoded);
                }
                PacketCommand::WriteVarIntLen => {
                    println!("[BUILD] Packet {} Command {}: WRITE_VARINT PACKET_LEN placeholder inserted at {}",
                             packet_idx + 1, idx + 1, packet.len());
                    varint_placeholders.push(packet.len());
                }
                PacketCommand::WriteIntLen(big_endian) => {
                    println!("[BUILD] Packet {} Command {}: WRITE_INT{} PACKET_LEN placeholder inserted at {}",
                             packet_idx + 1, idx + 1, if *big_endian { "_BE" } else { "" }, packet.len());
                    int_placeholders.push((packet.len(), *big_endian));
                    // Reserve 4 bytes for the length field
                    packet.extend_from_slice(&[0u8; 4]);
                }
            }
            println!("[BUILD] Packet {} size after command {}: {} bytes", packet_idx + 1, idx + 1, packet.len());
        }

        println!("[BUILD] Packet {} construction complete: {} total bytes (hex: {})", 
                 packet_idx + 1, packet.len(), hex::encode(&packet));
        
        // Replace VarInt placeholders (in reverse order to maintain positions)
        for &placeholder_pos in varint_placeholders.iter().rev() {
            let suffix_len = packet.len() - placeholder_pos;
            let encoded = encode_varint(suffix_len as u64);
            println!(
                "[BUILD] Packet {} placeholder at {} replaced with VarInt {} (bytes: {})",
                packet_idx + 1,
                placeholder_pos,
                suffix_len,
                hex::encode(&encoded)
            );
            packet.splice(placeholder_pos..placeholder_pos, encoded.iter().cloned());
        }
        
        // Replace fixed Int placeholders (in reverse order to maintain positions)
        for &(placeholder_pos, big_endian) in int_placeholders.iter().rev() {
            // Calculate length: everything after the 4-byte length field itself
            // If placeholder is at position 0, length = packet.len() - 4
            // If placeholder is at position N, length = packet.len() - N - 4
            let length = packet.len() - placeholder_pos - 4;
            let bytes = if big_endian {
                (length as u32).to_be_bytes()
            } else {
                (length as u32).to_le_bytes()
            };
            println!(
                "[BUILD] Packet {} placeholder at {} replaced with Int{} {} (bytes: {:02X} {:02X} {:02X} {:02X})",
                packet_idx + 1,
                placeholder_pos,
                if big_endian { "_BE" } else { "" },
                length,
                bytes[0], bytes[1], bytes[2], bytes[3]
            );
            packet[placeholder_pos..placeholder_pos + 4].copy_from_slice(&bytes);
        }
        
            println!("[BUILD] Packet {} construction complete: {} payload bytes (hex: {})", 
                     packet_idx + 1, packet.len(), hex::encode(&packet));
            built_packets.push(packet);
        }
    }

    println!("[BUILD] All packets built: {} packet(s) total", built_packets.len());
    Ok(built_packets)
}

fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut temp = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0x80;
        }
        bytes.push(temp);
        if value == 0 {
            break;
        }
    }
    bytes
}

pub fn parse_response(
    response_commands: &[ResponseCommand],
    response: &[u8],
) -> Result<(IndexMap<String, serde_json::Value>, usize)> {
    println!("[PARSE] Starting response parsing: {} bytes received (hex: {})", 
             response.len(), hex::encode(response));
    let mut vars = IndexMap::new();
    let mut cursor = 0;

    for (idx, cmd) in response_commands.iter().enumerate() {
        match cmd {
            ResponseCommand::ReadByte(var) => {
                if cursor >= response.len() {
                    anyhow::bail!("Insufficient data: need 1 byte, have {}", response.len() - cursor);
                }
                let value = response[cursor];
                println!("[PARSE] Command {}: READ_BYTE {} -> 0x{:02X} ({}) at offset {}", 
                         idx + 1, var, value, value, cursor);
                vars.insert(var.clone(), serde_json::Value::Number(value.into()));
                cursor += 1;
            }
            ResponseCommand::ReadShort(var, big_endian) => {
                if cursor + 2 > response.len() {
                    anyhow::bail!("Insufficient data: need 2 bytes, have {}", response.len() - cursor);
                }
                let value = if *big_endian {
                    u16::from_be_bytes([response[cursor], response[cursor + 1]])
                } else {
                    u16::from_le_bytes([response[cursor], response[cursor + 1]])
                };
                println!("[PARSE] Command {}: READ_SHORT{} {} -> {} at offset {} (bytes: {:02X} {:02X})", 
                         idx + 1, if *big_endian { "_BE" } else { "" }, var, value, cursor,
                         response[cursor], response[cursor + 1]);
                vars.insert(var.clone(), serde_json::Value::Number(value.into()));
                cursor += 2;
            }
            ResponseCommand::ReadInt(var, big_endian) => {
                if cursor + 4 > response.len() {
                    anyhow::bail!("Insufficient data: need 4 bytes, have {}", response.len() - cursor);
                }
                let value = if *big_endian {
                    u32::from_be_bytes([
                        response[cursor],
                        response[cursor + 1],
                        response[cursor + 2],
                        response[cursor + 3],
                    ])
                } else {
                    u32::from_le_bytes([
                        response[cursor],
                        response[cursor + 1],
                        response[cursor + 2],
                        response[cursor + 3],
                    ])
                };
                println!("[PARSE] Command {}: READ_INT{} {} -> {} at offset {} (bytes: {:02X} {:02X} {:02X} {:02X})", 
                         idx + 1, if *big_endian { "_BE" } else { "" }, var, value, cursor,
                         response[cursor], response[cursor + 1], response[cursor + 2], response[cursor + 3]);
                vars.insert(var.clone(), serde_json::Value::Number(value.into()));
                cursor += 4;
            }
            ResponseCommand::ReadVarInt(var) => {
                let start = cursor;
                let value = read_varint(response, &mut cursor)?;
                println!("[PARSE] Command {}: READ_VARINT {} -> {} at offsets {}-{}", idx + 1, var, value, start, cursor);
                vars.insert(var.clone(), serde_json::Value::Number(value.into()));
            }
            ResponseCommand::ReadString(var, length_opt) => {
                if let Some(length) = length_opt {
                    if cursor + length > response.len() {
                        anyhow::bail!("Insufficient data: need {} bytes, have {}", length, response.len() - cursor);
                    }
                    let bytes = &response[cursor..cursor + length];
                    let text = String::from_utf8_lossy(bytes).trim_end_matches('\0').to_string();
                    println!("[PARSE] Command {}: READ_STRING {} {} -> \"{}\" at offset {}", 
                             idx + 1, var, length, text, cursor);
                    vars.insert(var.clone(), serde_json::Value::String(text));
                    cursor += length;
                } else {
                    anyhow::bail!("READ_STRING requires length");
                }
            }
            ResponseCommand::ReadStringNull(var) => {
                let start = cursor;
                while cursor < response.len() && response[cursor] != 0 {
                    cursor += 1;
                }
                let bytes = &response[start..cursor];
                let text = String::from_utf8_lossy(bytes).to_string();
                println!("[PARSE] Command {}: READ_STRING_NULL {} -> \"{}\" at offset {} (length: {})", 
                         idx + 1, var, text, start, text.len());
                vars.insert(var.clone(), serde_json::Value::String(text));
                if cursor < response.len() {
                    cursor += 1; // Skip null terminator
                }
            }
            ResponseCommand::SkipBytes(count) => {
                if cursor + count > response.len() {
                    anyhow::bail!("Insufficient data: need {} bytes, have {}", count, response.len() - cursor);
                }
                println!("[PARSE] Command {}: SKIP_BYTES {} at offset {}", idx + 1, count, cursor);
                cursor += count;
            }
            ResponseCommand::ExpectByte(expected) => {
                if cursor >= response.len() {
                    anyhow::bail!("Insufficient data: need 1 byte for EXPECT_BYTE, have {}", response.len() - cursor);
                }
                let actual = response[cursor];
                println!("[PARSE] Command {}: EXPECT_BYTE 0x{:02X} at offset {} (got: 0x{:02X})", 
                         idx + 1, expected, cursor, actual);
                if actual != *expected {
                    anyhow::bail!("Expected byte 0x{:02X}, got 0x{:02X}", expected, actual);
                }
                cursor += 1;
            }
            ResponseCommand::ExpectMagic(expected) => {
                if cursor + expected.len() > response.len() {
                    anyhow::bail!("Insufficient data: need {} bytes for EXPECT_MAGIC, have {}", expected.len(), response.len() - cursor);
                }
                let actual = &response[cursor..cursor + expected.len()];
                println!("[PARSE] Command {}: EXPECT_MAGIC {} at offset {} (got: {})", 
                         idx + 1, hex::encode(expected), cursor, hex::encode(actual));
                if actual != expected.as_slice() {
                    anyhow::bail!("Expected magic bytes {:?}, got {:?}", hex::encode(expected), hex::encode(actual));
                }
                cursor += expected.len();
            }
            ResponseCommand::ExpectStatus(_) => {
                anyhow::bail!("EXPECT_STATUS is only valid for HTTP responses, not binary responses");
            }
            ResponseCommand::ExpectHeader { .. } => {
                anyhow::bail!("EXPECT_HEADER is only valid for HTTP responses, not binary responses");
            }
            ResponseCommand::ReadBodyJson(_) => {
                anyhow::bail!("READ_BODY_JSON is only valid for HTTP responses, not binary responses");
            }
            ResponseCommand::ReadBody(_) => {
                anyhow::bail!("READ_BODY is only valid for HTTP responses, not binary responses");
            }
        }
        println!("[PARSE] Cursor position after command {}: {} / {}", idx + 1, cursor, response.len());
    }

    println!("[PARSE] Response parsing complete: {} variables extracted, {} bytes consumed", 
             vars.len(), cursor);
    Ok((vars, cursor))
}

pub fn execute_code_blocks(
    code_blocks: &[CodeBlock],
    parsed_vars: &mut IndexMap<String, JsonValue>,
) -> Result<IndexMap<String, JsonValue>> {
    println!("[EXEC] Executing {} code block(s)...", code_blocks.len());
    println!("[EXEC] Available parsed variables: {:?}", parsed_vars.keys().collect::<Vec<_>>());
    let mut code_vars = IndexMap::new();
    
    for (block_idx, block) in code_blocks.iter().enumerate() {
        println!("[EXEC] Executing code block {} with {} commands...", block_idx + 1, block.commands.len());
        
        for (cmd_idx, cmd) in block.commands.iter().enumerate() {
            println!("[EXEC] Block {} Command {}: {:?}", block_idx + 1, cmd_idx + 1, cmd);
            match execute_code_command(cmd, parsed_vars, &mut code_vars) {
                Ok(()) => {
                    println!("[EXEC] Block {} Command {} executed successfully", block_idx + 1, cmd_idx + 1);
                }
                Err(e) => {
                    println!("[EXEC] Block {} Command {} failed: {}", block_idx + 1, cmd_idx + 1, e);
                    return Err(e);
                }
            }
        }
    }
    
    println!("[EXEC] Code execution complete: {} variables created: {:?}", 
             code_vars.len(), code_vars.keys().collect::<Vec<_>>());
    Ok(code_vars)
}

fn execute_code_command(
    cmd: &CodeCommand,
    parsed_vars: &IndexMap<String, JsonValue>,
    code_vars: &mut IndexMap<String, JsonValue>,
) -> Result<()> {
    match cmd {
        CodeCommand::DeclareVar { var_type, name, value } => {
            let evaluated = evaluate_expression(value, parsed_vars, code_vars)?;
            println!("[EXEC] Declaring variable {} ({:?}) = {:?}", name, var_type, evaluated);
            code_vars.insert(name.clone(), evaluated);
        }
        CodeCommand::AssignVar { name, value } => {
            let evaluated = evaluate_expression(value, parsed_vars, code_vars)?;
            println!("[EXEC] Assigning variable {} = {:?}", name, evaluated);
            // Update in code_vars if exists, otherwise create
            code_vars.insert(name.clone(), evaluated);
        }
        CodeCommand::Split { var_name, source_expr, delimiter } => {
            let source_value = evaluate_expression(source_expr, parsed_vars, code_vars)?;
            let source_str = source_value.as_str()
                .ok_or_else(|| anyhow::anyhow!("SPLIT source expression is not a string"))?;
            
            let parts: Vec<JsonValue> = source_str
                .split(delimiter)
                .map(|s| JsonValue::String(s.to_string()))
                .collect();
            
            println!("[EXEC] SPLIT expression by '{}' -> {} parts", delimiter, parts.len());
            code_vars.insert(var_name.clone(), JsonValue::Array(parts));
        }
        CodeCommand::Replace { var_name, source_expr, search, replace } => {
            let source_value = evaluate_expression(source_expr, parsed_vars, code_vars)?;
            let source_str = source_value.as_str()
                .ok_or_else(|| anyhow::anyhow!("REPLACE source expression is not a string"))?;
            
            let result = source_str.replace(search, replace);
            println!("[EXEC] REPLACE in expression: '{}' -> '{}'", search, replace);
            code_vars.insert(var_name.clone(), JsonValue::String(result));
        }
        CodeCommand::ForLoop { .. } => {
            // TODO: Implement FOR loop execution
            println!("[EXEC] FOR loop execution not yet implemented");
        }
        CodeCommand::ForInArray { var_name, array_name, body } => {
            println!("[EXEC] FOR {} IN {}: executing loop", var_name, array_name);
            let array_value = get_variable_value(array_name, parsed_vars, code_vars)?;
            let array = array_value.as_array()
                .ok_or_else(|| anyhow::anyhow!("Variable '{}' is not an array", array_name))?;
            
            for (idx, item) in array.iter().enumerate() {
                println!("[EXEC] FOR loop iteration {}: {} = {:?}", idx, var_name, item);
                // Set the loop variable
                code_vars.insert(var_name.clone(), item.clone());
                
                // Execute body
                let mut should_break = false;
                for body_cmd in body {
                    match execute_code_command(body_cmd, parsed_vars, code_vars) {
                        Ok(()) => {}
                        Err(e) if e.to_string().contains("BREAK") => {
                            should_break = true;
                            break;
                        }
                        Err(e) => return Err(e),
                    }
                }
                
                if should_break {
                    println!("[EXEC] FOR loop broken");
                    break;
                }
            }
        }
        CodeCommand::IfStatement { condition, body, else_if, else_body } => {
            println!("[EXEC] IF statement: evaluating condition");
            let condition_result = evaluate_condition(condition, parsed_vars, code_vars)?;
            
            if condition_result {
                println!("[EXEC] IF condition true, executing body");
                for body_cmd in body {
                    execute_code_command(body_cmd, parsed_vars, code_vars)?;
                }
            } else {
                // Check else-if conditions
                let mut matched = false;
                for (else_cond, else_body_cmds) in else_if {
                    if evaluate_condition(else_cond, parsed_vars, code_vars)? {
                        println!("[EXEC] ELSE-IF condition true, executing body");
                        for body_cmd in else_body_cmds {
                            execute_code_command(body_cmd, parsed_vars, code_vars)?;
                        }
                        matched = true;
                        break;
                    }
                }
                
                // Execute else body if no else-if matched
                if !matched {
                    if let Some(else_body_cmds) = else_body {
                        println!("[EXEC] IF condition false, executing else body");
                        for body_cmd in else_body_cmds {
                            execute_code_command(body_cmd, parsed_vars, code_vars)?;
                        }
                    }
                }
            }
        }
        CodeCommand::Break => {
            println!("[EXEC] BREAK encountered");
            return Err(anyhow::anyhow!("BREAK"));
        }
        CodeCommand::ExecutePacketCommand(_) => {
            // TODO: Nested packet command execution
            println!("[EXEC] Nested packet command execution not yet implemented");
        }
        CodeCommand::ExecuteResponseCommand(_) => {
            // TODO: Nested response command execution
            println!("[EXEC] Nested response command execution not yet implemented");
        }
    }
    Ok(())
}

fn evaluate_condition(
    condition: &Condition,
    parsed_vars: &IndexMap<String, JsonValue>,
    code_vars: &IndexMap<String, JsonValue>,
) -> Result<bool> {
    match condition {
        Condition::Equals(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            Ok(left_val == right_val)
        }
        Condition::NotEquals(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            Ok(left_val != right_val)
        }
        Condition::GreaterThan(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            compare_values(&left_val, &right_val, |a, b| a > b)
        }
        Condition::LessThan(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            compare_values(&left_val, &right_val, |a, b| a < b)
        }
        Condition::GreaterOrEqual(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            compare_values(&left_val, &right_val, |a, b| a >= b)
        }
        Condition::LessOrEqual(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            compare_values(&left_val, &right_val, |a, b| a <= b)
        }
        Condition::Contains(left, right) => {
            let left_val = evaluate_expression(left, parsed_vars, code_vars)?;
            let right_val = evaluate_expression(right, parsed_vars, code_vars)?;
            let left_str = left_val.as_str()
                .ok_or_else(|| anyhow::anyhow!("CONTAINS left operand must be a string"))?;
            let right_str = right_val.as_str()
                .ok_or_else(|| anyhow::anyhow!("CONTAINS right operand must be a string"))?;
            Ok(left_str.contains(right_str))
        }
    }
}

fn compare_values<F>(left: &JsonValue, right: &JsonValue, cmp: F) -> Result<bool>
where
    F: FnOnce(f64, f64) -> bool,
{
    let left_num = left.as_f64()
        .or_else(|| left.as_u64().map(|n| n as f64))
        .or_else(|| left.as_i64().map(|n| n as f64))
        .ok_or_else(|| anyhow::anyhow!("Left operand must be a number"))?;
    let right_num = right.as_f64()
        .or_else(|| right.as_u64().map(|n| n as f64))
        .or_else(|| right.as_i64().map(|n| n as f64))
        .ok_or_else(|| anyhow::anyhow!("Right operand must be a number"))?;
    Ok(cmp(left_num, right_num))
}

fn evaluate_expression(
    expr: &Expression,
    parsed_vars: &IndexMap<String, JsonValue>,
    code_vars: &IndexMap<String, JsonValue>,
) -> Result<JsonValue> {
    match expr {
        Expression::Literal(value) => Ok(value.clone()),
        Expression::Variable(name) => {
            get_variable_value(name, parsed_vars, code_vars)
        }
        Expression::ArrayIndex { array_name, index } => {
            // Get the array value
            let array_value = get_variable_value(array_name, parsed_vars, code_vars)?;
            let array = array_value.as_array()
                .ok_or_else(|| anyhow::anyhow!("Variable '{}' is not an array", array_name))?;
            
            // Evaluate the index expression
            let index_value = evaluate_expression(index, parsed_vars, code_vars)?;
            let index_num = index_value.as_u64()
                .or_else(|| index_value.as_i64().map(|i| i as u64))
                .ok_or_else(|| anyhow::anyhow!("Array index must be a number, got: {:?}", index_value))?;
            
            // Get the element at the index
            let idx = index_num as usize;
            if idx >= array.len() {
                anyhow::bail!("Array index {} out of bounds for array of length {}", idx, array.len());
            }
            
            Ok(array[idx].clone())
        }
        Expression::FunctionCall { name, args } => {
            // Handle array literals
            if name == "__array_literal__" {
                let elements: Result<Vec<JsonValue>> = args.iter()
                    .map(|arg| evaluate_expression(arg, parsed_vars, code_vars))
                    .collect();
                return Ok(JsonValue::Array(elements?));
            }
            
            // Evaluate function calls
            let _evaluated_args: Result<Vec<JsonValue>> = args.iter()
                .map(|arg| evaluate_expression(arg, parsed_vars, code_vars))
                .collect();
            
            // Handle built-in functions
            match name.as_str() {
                // Add more functions as needed
                _ => anyhow::bail!("Unknown function: {}", name),
            }
        }
    }
}

fn get_variable_value(
    name: &str,
    parsed_vars: &IndexMap<String, JsonValue>,
    code_vars: &IndexMap<String, JsonValue>,
) -> Result<JsonValue> {
    // Check code_vars first (most recent), then parsed_vars
    if let Some(value) = code_vars.get(name) {
        Ok(value.clone())
    } else if let Some(value) = parsed_vars.get(name) {
        Ok(value.clone())
    } else {
        anyhow::bail!("Variable '{}' not found", name)
    }
}

fn read_varint(response: &[u8], cursor: &mut usize) -> Result<u64> {
    let mut value = 0u64;
    let mut shift = 0;
    loop {
        if *cursor >= response.len() {
            anyhow::bail!("Insufficient data reading VarInt");
        }
        let byte = response[*cursor];
        *cursor += 1;
        value |= ((byte & 0x7F) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift >= 35 {
            anyhow::bail!("VarInt too large");
        }
    }
}

/// HTTP request data prepared for sending
#[derive(Debug, Clone)]
pub struct PreparedHttpRequest {
    pub method: String,
    pub path: String,
    pub params: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub body: Option<(String, Vec<u8>)>, // (content_type, body_bytes)
}

/// Prepare HTTP request from HttpRequest struct, substituting variables
pub fn prepare_http_request_with_vars(
    http_req: &HttpRequest,
    vars: &IndexMap<String, JsonValue>,
) -> Result<PreparedHttpRequest> {
    // Resolve path (substitute variables)
    let path = resolve_string_value(&http_req.path, vars)?;
    
    // Resolve params
    let mut resolved_params = Vec::new();
    for (key, value) in &http_req.params {
        let resolved_key = resolve_string_value(key, vars)?;
        let resolved_value = resolve_string_value(value, vars)?;
        resolved_params.push((resolved_key, resolved_value));
    }
    
    // Resolve headers
    let mut resolved_headers = Vec::new();
    for (key, value) in &http_req.headers {
        let resolved_key = resolve_string_value(key, vars)?;
        let resolved_value = resolve_string_value(value, vars)?;
        resolved_headers.push((resolved_key, resolved_value));
    }
    
    // Get HTTP method string
    let method_str = match &http_req.method {
        HttpMethod::Get => "GET".to_string(),
        HttpMethod::Post => "POST".to_string(),
        HttpMethod::Put => "PUT".to_string(),
        HttpMethod::Delete => "DELETE".to_string(),
        HttpMethod::Custom(m) => m.clone(),
    };
    
    // Build body
    let body = if let Some(body_type) = &http_req.body_type {
        let content_type = match body_type {
            HttpBodyType::Form => "application/x-www-form-urlencoded".to_string(),
            HttpBodyType::Raw => {
                // Check if Content-Type header is set, otherwise default to application/json
                resolved_headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("Content-Type"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "application/json".to_string())
            }
        };
        
        let body_bytes = match body_type {
            HttpBodyType::Form => {
                // Build form data from body_data (key=value pairs)
                let form_parts: Vec<String> = http_req.body_data
                    .iter()
                    .map(|data| {
                        // Each DATA entry might be "key=value" or just "value"
                        // For form, we expect "key=value" format
                        resolve_string_value(data, vars).unwrap_or_else(|_| data.clone())
                    })
                    .collect();
                form_parts.join("&").into_bytes()
            }
            HttpBodyType::Raw => {
                // Join all body data and try to parse as JSON, then stringify
                // This allows users to write JSON directly
                let combined = http_req.body_data.join("\n");
                let resolved = resolve_string_value(&combined, vars)?;
                
                // Try to parse as JSON to validate and pretty-print, then convert back to string
                // This handles the automatic JSON stringification mentioned in the spec
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&resolved) {
                    // Valid JSON - stringify it
                    serde_json::to_string(&json_value)?.into_bytes()
                } else {
                    // Not valid JSON, use as-is
                    resolved.into_bytes()
                }
            }
        };
        
        Some((content_type, body_bytes))
    } else {
        None
    };
    
    Ok(PreparedHttpRequest {
        method: method_str,
        path,
        params: resolved_params,
        headers: resolved_headers,
        body,
    })
}

/// Helper to resolve string values, substituting variables
fn resolve_string_value(s: &str, vars: &IndexMap<String, JsonValue>) -> Result<String> {
    // Simple variable substitution: if the string matches a variable name exactly, use it
    // Otherwise, return as-is (future: could support embedded variables like "Bearer {token}")
    if let Some(value) = vars.get(s) {
        Ok(value.as_str().unwrap_or(&value.to_string()).to_string())
    } else {
        Ok(s.to_string())
    }
}

/// Parse HTTP response using response commands
pub fn parse_http_response(
    response_commands: &[ResponseCommand],
    status_code: u16,
    headers: &reqwest::header::HeaderMap,
    body: &[u8],
) -> Result<IndexMap<String, serde_json::Value>> {
    let mut vars = IndexMap::new();
    
    // Store status code as a variable
    vars.insert("STATUS_CODE".to_string(), serde_json::json!(status_code));
    
    // Store headers as variables (HEADER_<Key>)
    for (key, value) in headers.iter() {
        let header_name = format!("HEADER_{}", key.as_str().replace("-", "_"));
        if let Ok(value_str) = value.to_str() {
            vars.insert(header_name, serde_json::json!(value_str));
        }
    }
    
    for cmd in response_commands {
        match cmd {
            ResponseCommand::ExpectStatus(expected) => {
                if status_code != *expected {
                    anyhow::bail!("Expected status code {}, got {}", expected, status_code);
                }
            }
            ResponseCommand::ExpectHeader { key, value } => {
                let header_value = headers
                    .get(key)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| anyhow::anyhow!("Header '{}' not found or invalid", key))?;
                
                if header_value != value.as_str() {
                    anyhow::bail!("Expected header '{}' to be '{}', got '{}'", key, value, header_value);
                }
            }
            ResponseCommand::ReadBodyJson(var_name) => {
                let json_value: serde_json::Value = serde_json::from_slice(body)
                    .context("Failed to parse response body as JSON")?;
                vars.insert(var_name.clone(), json_value);
            }
            ResponseCommand::ReadBody(var_name) => {
                let body_text = String::from_utf8(body.to_vec())
                    .context("Failed to parse response body as UTF-8 text")?;
                vars.insert(var_name.clone(), serde_json::json!(body_text));
            }
            _ => {
                // Other commands are not valid for HTTP responses
                anyhow::bail!("Command {:?} is not valid for HTTP responses", cmd);
            }
        }
    }
    
    Ok(vars)
}

