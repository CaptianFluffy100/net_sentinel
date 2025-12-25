use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PacketCommand {
    PacketStart,
    PacketEnd,
    WriteByte(u8),
    WriteShort(u16, bool), // value, big_endian
    WriteInt(u32, bool),   // value, big_endian
    WriteString(String),
    WriteStringLen(String, usize),
    WriteBytes(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum ResponseCommand {
    ResponseStart,
    ResponseEnd,
    ReadByte(String),
    ReadShort(String, bool), // var_name, big_endian
    ReadInt(String, bool),   // var_name, big_endian
    ReadString(String, usize),
    ReadStringNull(String),
    SkipBytes(usize),
    ExpectByte(u8),
    ExpectMagic(Vec<u8>),
}

pub fn parse_packet_commands(code: &str) -> Result<Vec<PacketCommand>> {
    let mut commands = Vec::new();
    let mut in_packet = false;

    for (line_num, line) in code.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle PACKET_START
        if line == "PACKET_START" {
            in_packet = true;
            commands.push(PacketCommand::PacketStart);
            continue;
        }

        // Handle PACKET_END
        if line == "PACKET_END" {
            in_packet = false;
            commands.push(PacketCommand::PacketEnd);
            continue;
        }

        // Only parse packet commands if we're in a packet block
        if !in_packet {
            continue;
        }

        // Parse WRITE commands
        if let Some(arg) = line.strip_prefix("WRITE_BYTE ") {
            let value: u8 = arg
                .trim()
                .strip_prefix("0x")
                .map(|s| u8::from_str_radix(s, 16))
                .unwrap_or_else(|| arg.trim().parse())
                .context(format!("Invalid byte value at line {}", line_num + 1))?;
            commands.push(PacketCommand::WriteByte(value));
        } else if let Some(arg) = line.strip_prefix("WRITE_SHORT_BE ") {
            let value: u16 = arg
                .trim()
                .strip_prefix("0x")
                .map(|s| u16::from_str_radix(s, 16))
                .unwrap_or_else(|| arg.trim().parse())
                .context(format!("Invalid short value at line {}", line_num + 1))?;
            commands.push(PacketCommand::WriteShort(value, true));
        } else if let Some(arg) = line.strip_prefix("WRITE_SHORT ") {
            let value: u16 = arg
                .trim()
                .strip_prefix("0x")
                .map(|s| u16::from_str_radix(s, 16))
                .unwrap_or_else(|| arg.trim().parse())
                .context(format!("Invalid short value at line {}", line_num + 1))?;
            commands.push(PacketCommand::WriteShort(value, false));
        } else if let Some(arg) = line.strip_prefix("WRITE_INT_BE ") {
            let value: u32 = arg
                .trim()
                .strip_prefix("0x")
                .map(|s| u32::from_str_radix(s, 16))
                .unwrap_or_else(|| arg.trim().parse())
                .context(format!("Invalid int value at line {}", line_num + 1))?;
            commands.push(PacketCommand::WriteInt(value, true));
        } else if let Some(arg) = line.strip_prefix("WRITE_INT ") {
            let value: u32 = arg
                .trim()
                .strip_prefix("0x")
                .map(|s| u32::from_str_radix(s, 16))
                .unwrap_or_else(|| arg.trim().parse())
                .context(format!("Invalid int value at line {}", line_num + 1))?;
            commands.push(PacketCommand::WriteInt(value, false));
        } else if let Some(arg) = line.strip_prefix("WRITE_STRING \"") {
            if let Some(end) = arg.rfind('"') {
                let value = arg[..end].to_string();
                commands.push(PacketCommand::WriteString(value));
            } else {
                return Err(anyhow::anyhow!("Unclosed string at line {}", line_num + 1));
            }
        } else if let Some(arg) = line.strip_prefix("WRITE_STRING_LEN \"") {
            if let Some(end) = arg.rfind('"') {
                let parts: Vec<&str> = arg[end + 1..].trim().split_whitespace().collect();
                if parts.len() != 1 {
                    return Err(anyhow::anyhow!("WRITE_STRING_LEN requires length argument at line {}", line_num + 1));
                }
                let value = arg[..end].to_string();
                let len: usize = parts[0].parse()
                    .context(format!("Invalid length at line {}", line_num + 1))?;
                commands.push(PacketCommand::WriteStringLen(value, len));
            } else {
                return Err(anyhow::anyhow!("Unclosed string at line {}", line_num + 1));
            }
        } else if let Some(arg) = line.strip_prefix("WRITE_BYTES \"") {
            if let Some(end) = arg.rfind('"') {
                let hex_str = arg[..end].replace(" ", "");
                let bytes = (0..hex_str.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex_str[i..i.min(hex_str.len())], 16))
                    .collect::<Result<Vec<u8>, _>>()
                    .context(format!("Invalid hex string at line {}", line_num + 1))?;
                commands.push(PacketCommand::WriteBytes(bytes));
            } else {
                return Err(anyhow::anyhow!("Unclosed string at line {}", line_num + 1));
            }
        } else {
            return Err(anyhow::anyhow!("Unknown command at line {}: {}", line_num + 1, line));
        }
    }

    Ok(commands)
}

pub fn parse_response_commands(code: &str) -> Result<Vec<ResponseCommand>> {
    let mut commands = Vec::new();
    let mut in_response = false;

    for (line_num, line) in code.lines().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle RESPONSE_START
        if line == "RESPONSE_START" {
            in_response = true;
            commands.push(ResponseCommand::ResponseStart);
            continue;
        }

        // Handle RESPONSE_END
        if line == "RESPONSE_END" {
            in_response = false;
            commands.push(ResponseCommand::ResponseEnd);
            continue;
        }

        // Only parse response commands if we're in a response block
        if !in_response {
            continue;
        }

        // Parse READ and EXPECT commands
        if let Some(arg) = line.strip_prefix("READ_BYTE ") {
            commands.push(ResponseCommand::ReadByte(arg.trim().to_string()));
        } else if let Some(arg) = line.strip_prefix("READ_SHORT_BE ") {
            commands.push(ResponseCommand::ReadShort(arg.trim().to_string(), true));
        } else if let Some(arg) = line.strip_prefix("READ_SHORT ") {
            commands.push(ResponseCommand::ReadShort(arg.trim().to_string(), false));
        } else if let Some(arg) = line.strip_prefix("READ_INT_BE ") {
            commands.push(ResponseCommand::ReadInt(arg.trim().to_string(), true));
        } else if let Some(arg) = line.strip_prefix("READ_INT ") {
            commands.push(ResponseCommand::ReadInt(arg.trim().to_string(), false));
        } else if let Some(arg) = line.strip_prefix("READ_STRING ") {
            let parts: Vec<&str> = arg.trim().split_whitespace().collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("READ_STRING requires var_name and length at line {}", line_num + 1));
            }
            let len: usize = parts[1].parse()
                .context(format!("Invalid length at line {}", line_num + 1))?;
            commands.push(ResponseCommand::ReadString(parts[0].to_string(), len));
        } else if let Some(arg) = line.strip_prefix("READ_STRING_NULL ") {
            commands.push(ResponseCommand::ReadStringNull(arg.trim().to_string()));
        } else if let Some(arg) = line.strip_prefix("SKIP_BYTES ") {
            let count: usize = arg.trim().parse()
                .context(format!("Invalid byte count at line {}", line_num + 1))?;
            commands.push(ResponseCommand::SkipBytes(count));
        } else if let Some(arg) = line.strip_prefix("EXPECT_BYTE ") {
            let value: u8 = arg
                .trim()
                .strip_prefix("0x")
                .map(|s| u8::from_str_radix(s, 16))
                .unwrap_or_else(|| arg.trim().parse())
                .context(format!("Invalid byte value at line {}", line_num + 1))?;
            commands.push(ResponseCommand::ExpectByte(value));
        } else if let Some(arg) = line.strip_prefix("EXPECT_MAGIC \"") {
            if let Some(end) = arg.rfind('"') {
                let hex_str = arg[..end].replace(" ", "");
                let bytes = (0..hex_str.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex_str[i..i.min(hex_str.len())], 16))
                    .collect::<Result<Vec<u8>, _>>()
                    .context(format!("Invalid hex string at line {}", line_num + 1))?;
                commands.push(ResponseCommand::ExpectMagic(bytes));
            } else {
                return Err(anyhow::anyhow!("Unclosed string at line {}", line_num + 1));
            }
        } else {
            return Err(anyhow::anyhow!("Unknown command at line {}: {}", line_num + 1, line));
        }
    }

    Ok(commands)
}

pub fn build_packet(commands: &[PacketCommand]) -> Result<Vec<u8>> {
    let mut packet = Vec::new();

    for cmd in commands {
        match cmd {
            PacketCommand::PacketStart => {
                packet.clear(); // Reset packet
            }
            PacketCommand::PacketEnd => {
                // Done building packet
            }
            PacketCommand::WriteByte(value) => {
                packet.push(*value);
            }
            PacketCommand::WriteShort(value, big_endian) => {
                if *big_endian {
                    packet.extend_from_slice(&value.to_be_bytes());
                } else {
                    packet.extend_from_slice(&value.to_le_bytes());
                }
            }
            PacketCommand::WriteInt(value, big_endian) => {
                if *big_endian {
                    packet.extend_from_slice(&value.to_be_bytes());
                } else {
                    packet.extend_from_slice(&value.to_le_bytes());
                }
            }
            PacketCommand::WriteString(value) => {
                packet.extend_from_slice(value.as_bytes());
                packet.push(0); // Null terminator
            }
            PacketCommand::WriteStringLen(value, len) => {
                let bytes = value.as_bytes();
                let len = *len;
                if bytes.len() >= len {
                    packet.extend_from_slice(&bytes[..len]);
                } else {
                    packet.extend_from_slice(bytes);
                    packet.extend(vec![0; len - bytes.len()]);
                }
            }
            PacketCommand::WriteBytes(bytes) => {
                packet.extend_from_slice(bytes);
            }
        }
    }

    Ok(packet)
}

pub fn parse_response(
    commands: &[ResponseCommand],
    data: &[u8],
) -> Result<(HashMap<String, serde_json::Value>, Option<(usize, String)>)> {
    let mut vars = HashMap::new();
    let mut pos = 0;

    for (cmd_idx, cmd) in commands.iter().enumerate() {
        match cmd {
            ResponseCommand::ResponseStart => {
                pos = 0; // Reset position
            }
            ResponseCommand::ResponseEnd => {
                // Done parsing
            }
            ResponseCommand::ReadByte(var_name) => {
                if pos >= data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data reading byte for {} at position {}",
                        var_name,
                        pos
                    ));
                }
                vars.insert(var_name.clone(), serde_json::json!(data[pos]));
                pos += 1;
            }
            ResponseCommand::ReadShort(var_name, big_endian) => {
                if pos + 1 >= data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data reading short for {} at position {}",
                        var_name,
                        pos
                    ));
                }
                let value = if *big_endian {
                    u16::from_be_bytes([data[pos], data[pos + 1]])
                } else {
                    u16::from_le_bytes([data[pos], data[pos + 1]])
                };
                vars.insert(var_name.clone(), serde_json::json!(value));
                pos += 2;
            }
            ResponseCommand::ReadInt(var_name, big_endian) => {
                if pos + 3 >= data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data reading int for {} at position {}",
                        var_name,
                        pos
                    ));
                }
                let value = if *big_endian {
                    u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                } else {
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                };
                vars.insert(var_name.clone(), serde_json::json!(value));
                pos += 4;
            }
            ResponseCommand::ReadString(var_name, len) => {
                if pos + *len > data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data reading string for {} at position {}",
                        var_name,
                        pos
                    ));
                }
                let string_bytes = &data[pos..pos + len];
                let string = String::from_utf8_lossy(string_bytes).trim_end_matches('\0').to_string();
                vars.insert(var_name.clone(), serde_json::json!(string));
                pos += len;
            }
            ResponseCommand::ReadStringNull(var_name) => {
                let start_pos = pos;
                while pos < data.len() && data[pos] != 0 {
                    pos += 1;
                }
                if pos > start_pos {
                    let string = String::from_utf8_lossy(&data[start_pos..pos]).to_string();
                    vars.insert(var_name.clone(), serde_json::json!(string));
                } else {
                    vars.insert(var_name.clone(), serde_json::json!(""));
                }
                if pos < data.len() {
                    pos += 1; // Skip null terminator
                }
            }
            ResponseCommand::SkipBytes(count) => {
                if pos + *count > data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data skipping {} bytes at position {}",
                        count,
                        pos
                    ));
                }
                pos += count;
            }
            ResponseCommand::ExpectByte(expected) => {
                if pos >= data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data expecting byte 0x{:02X} at position {}",
                        expected,
                        pos
                    ));
                }
                if data[pos] != *expected {
                    return Ok((
                        vars,
                        Some((
                            cmd_idx + 1,
                            format!(
                                "Expected byte 0x{:02X}, but received 0x{:02X} at position {}",
                                expected,
                                data[pos],
                                pos
                            ),
                        )),
                    ));
                }
                pos += 1;
            }
            ResponseCommand::ExpectMagic(expected_bytes) => {
                if pos + expected_bytes.len() > data.len() {
                    return Err(anyhow::anyhow!(
                        "Insufficient data expecting magic bytes at position {}",
                        pos
                    ));
                }
                if &data[pos..pos + expected_bytes.len()] != expected_bytes.as_slice() {
                    return Ok((
                        vars,
                        Some((
                            cmd_idx + 1,
                            format!(
                                "Magic bytes mismatch at position {}",
                                pos
                            ),
                        )),
                    ));
                }
                pos += expected_bytes.len();
            }
        }
    }

    Ok((vars, None))
}

pub fn hex_dump(data: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        result.push_str(&format!("{:08X}: ", offset));
        
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push_str(" ");
            }
            result.push_str(&format!("{:02X} ", byte));
        }
        
        // Pad to align ASCII
        for _ in chunk.len()..16 {
            result.push_str("   ");
            if chunk.len() < 8 && _ == 7 {
                result.push_str(" ");
            }
        }
        
        result.push_str(" ");
        for byte in chunk {
            let ch = if *byte >= 32 && *byte < 127 {
                *byte as char
            } else {
                '.'
            };
            result.push(ch);
        }
        result.push('\n');
    }
    result
}

