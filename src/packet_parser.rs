use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PacketCommand {
    WriteByte(u8),
    WriteShort(u16, bool), // value, big_endian
    WriteInt(u32, bool),   // value, big_endian
    WriteString(String, Option<usize>), // value, optional fixed length
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
pub struct PacketResponsePair {
    pub packet: Vec<PacketCommand>,
    pub response: Vec<ResponseCommand>,
}

#[derive(Debug)]
pub struct PacketScript {
    pub pairs: Vec<PacketResponsePair>,
    pub output_blocks: Vec<OutputBlock>,
}

pub fn parse_script(script: &str) -> Result<PacketScript> {
    println!("[PARSER] Starting script parsing...");
    let lines: Vec<&str> = script.lines().collect();
    let mut pairs = Vec::new();
    let mut current_packet = Vec::new();
    let mut current_response = Vec::new();
    let mut output_blocks = Vec::new();
    let mut current_output: Option<OutputBlock> = None;
    let mut in_packet = false;
    let mut in_response = false;

    for (line_num, line) in lines.iter().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            if line.starts_with('#') {
                println!("[PARSER] Line {}: Comment skipped: {}", line_num + 1, line);
            }
            continue;
        }

        // Packet section
        if line == "PACKET_START" {
            println!("[PARSER] Line {}: Entering PACKET_START section", line_num + 1);
            // If we were already in a packet, save the pair before starting a new one
            if in_packet && !current_packet.is_empty() {
                println!("[PARSER] Saving previous packet/response pair (packet: {} commands, response: {} commands)", 
                         current_packet.len(), current_response.len());
                pairs.push(PacketResponsePair {
                    packet: current_packet.clone(),
                    response: current_response.clone(),
                });
                current_packet.clear();
                current_response.clear();
            }
            in_packet = true;
            in_response = false;
            continue;
        }
        if line == "PACKET_END" {
            println!("[PARSER] Line {}: Exiting PACKET section (found {} commands)", line_num + 1, current_packet.len());
            in_packet = false;
            continue;
        }
        
        // Response section
        if line == "RESPONSE_START" {
            println!("[PARSER] Line {}: Entering RESPONSE_START section", line_num + 1);
            in_response = true;
            in_packet = false;
            continue;
        }
        if line == "RESPONSE_END" {
            println!("[PARSER] Line {}: Exiting RESPONSE section (found {} commands)", line_num + 1, current_response.len());
            // When response ends, save the packet/response pair
            if !current_packet.is_empty() {
                println!("[PARSER] Saving packet/response pair (packet: {} commands, response: {} commands)", 
                         current_packet.len(), current_response.len());
                pairs.push(PacketResponsePair {
                    packet: current_packet.clone(),
                    response: current_response.clone(),
                });
                current_packet.clear();
                current_response.clear();
            }
            in_response = false;
            continue;
        }

        if in_packet {
            println!("[PARSER] Line {}: Parsing packet command: {}", line_num + 1, line);
            current_packet.push(parse_packet_command(line, line_num + 1)?);
        } else if in_response {
            println!("[PARSER] Line {}: Parsing response command: {}", line_num + 1, line);
            current_response.push(parse_response_command(line, line_num + 1)?);
        } else {
            handle_output_line(line, line_num + 1, &mut current_output, &mut output_blocks)?;
        }
    }

    // Save any remaining packet/response pair
    if !current_packet.is_empty() {
        println!("[PARSER] Saving final packet/response pair (packet: {} commands, response: {} commands)", 
                 current_packet.len(), current_response.len());
        pairs.push(PacketResponsePair {
            packet: current_packet,
            response: current_response,
        });
    }

    if let Some(block) = current_output.take() {
        output_blocks.push(block);
    }

    println!("[PARSER] Script parsing complete: {} packet/response pair(s)", pairs.len());
    Ok(PacketScript {
        pairs,
        output_blocks,
    })
}

fn parse_packet_command(line: &str, line_num: usize) -> Result<PacketCommand> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("Empty command at line {}", line_num);
    }

    match parts[0] {
        "WRITE_BYTE" => {
            let value = parse_byte_value(parts.get(1).copied())?;
            Ok(PacketCommand::WriteByte(value))
        }
        "WRITE_SHORT" => {
            let value = parse_short_value(parts.get(1).copied())?;
            Ok(PacketCommand::WriteShort(value, false))
        }
        "WRITE_SHORT_BE" => {
            let value = parse_short_value(parts.get(1).copied())?;
            Ok(PacketCommand::WriteShort(value, true))
        }
        "WRITE_INT" => {
            let token = parts.get(1)
                .ok_or_else(|| anyhow::anyhow!("WRITE_INT requires value at line {}", line_num))?;
            if token.eq_ignore_ascii_case("PACKET_LEN") {
                Ok(PacketCommand::WriteIntLen(false)) // little-endian by default
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
                    // No quotes, use old parsing
                    let text = parse_string_value(parts.get(1).copied())?;
                    Ok(PacketCommand::WriteString(text, None))
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
        _ => anyhow::bail!("Unknown response command: {} at line {}", parts[0], line_num),
    }
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
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
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

pub fn build_packets(script: &PacketScript) -> Result<Vec<Vec<u8>>> {
    println!("[BUILD] Starting packet construction with {} pair(s)", script.pairs.len());
    let mut built_packets = Vec::new();

    for (pair_idx, pair) in script.pairs.iter().enumerate() {
        let packet_commands = &pair.packet;
        println!("[BUILD] Building packet {} (pair {}) with {} commands", pair_idx + 1, pair_idx + 1, packet_commands.len());
        let packet_idx = pair_idx + 1; // For logging compatibility
        let mut packet = Vec::new();
        let mut varint_placeholders = Vec::new();
        let mut int_placeholders = Vec::new(); // (position, big_endian)

        for (idx, cmd) in packet_commands.iter().enumerate() {
            match cmd {
                PacketCommand::WriteByte(v) => {
                    println!("[BUILD] Packet {} Command {}: WRITE_BYTE 0x{:02X} ({})", packet_idx + 1, idx + 1, v, v);
                    packet.push(*v);
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
) -> Result<(HashMap<String, serde_json::Value>, usize)> {
    println!("[PARSE] Starting response parsing: {} bytes received (hex: {})", 
             response.len(), hex::encode(response));
    let mut vars = HashMap::new();
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
        }
        println!("[PARSE] Cursor position after command {}: {} / {}", idx + 1, cursor, response.len());
    }

    println!("[PARSE] Response parsing complete: {} variables extracted, {} bytes consumed", 
             vars.len(), cursor);
    Ok((vars, cursor))
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

