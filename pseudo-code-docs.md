# Pseudo-Code Language Documentation

Complete reference for all pseudo-code commands available in Net Sentinel's game server checker.

## Table of Contents

1. [Overview](#overview)
2. [Basic Structure](#basic-structure)
3. [Packet Construction Commands](#packet-construction-commands)
4. [Response Parsing Commands](#response-parsing-commands)
5. [Validation Commands](#validation-commands)
6. [Output Formatting Directives](#output-formatting-directives)
7. [Comments](#comments)
8. [Special Features](#special-features)
9. [Complete Examples](#complete-examples)

## Overview

The pseudo-code language is a simple, line-based scripting language used to define:
- **Packet Construction**: How to build request packets to send to game servers
- **Response Parsing**: How to interpret and extract data from server responses
- **Output Formatting**: How to format results for Prometheus metrics and UI display

The language supports both **UDP** and **TCP** protocols, and can handle multiple packet/response pairs in sequence (useful for protocols like RCON that require authentication followed by commands).

## Basic Structure

A pseudo-code script consists of one or more **packet/response pairs**:

```
PACKET_START
  ... packet construction commands ...
PACKET_END

RESPONSE_START
  ... response parsing commands ...
RESPONSE_END

# Optional: Additional pairs
PACKET_START
  ... more packet commands ...
PACKET_END

RESPONSE_START
  ... more response commands ...
RESPONSE_END
```

**Important**: Pairs are executed **sequentially**. The first packet is sent, its response is received and parsed, then the second packet is sent, and so on. The connection/socket is kept alive for all pairs.

## Packet Construction Commands

### `PACKET_START`
Marks the beginning of a packet definition. Must be followed by packet construction commands and closed with `PACKET_END`.

**Example**:
```
PACKET_START
WRITE_BYTE 0xFF
PACKET_END
```

---

### `WRITE_BYTE <value>`
Writes a single byte (0-255) to the packet.

**Parameters**:
- `<value>`: Byte value in decimal (0-255) or hexadecimal (0x00-0xFF)

**Examples**:
```
WRITE_BYTE 255
WRITE_BYTE 0xFF
WRITE_BYTE 0xFE
```

---

### `WRITE_SHORT <value>`
Writes a 16-bit integer (little-endian by default).

**Parameters**:
- `<value>`: 16-bit integer value (0-65535) in decimal or hexadecimal

**Examples**:
```
WRITE_SHORT 1234
WRITE_SHORT 0x1234
```

**Note**: Use `WRITE_SHORT_BE` for big-endian format.

---

### `WRITE_SHORT_BE <value>`
Writes a 16-bit integer in big-endian (network byte order) format.

**Parameters**:
- `<value>`: 16-bit integer value (0-65535)

**Examples**:
```
WRITE_SHORT_BE 1234
WRITE_SHORT_BE 0x1234
```

---

### `WRITE_INT <value>`
Writes a 32-bit integer (little-endian by default).

**Parameters**:
- `<value>`: 32-bit integer value (0-4294967295) in decimal or hexadecimal
- Special: Use `PACKET_LEN` as the value to automatically calculate and write the packet length

**Examples**:
```
WRITE_INT 4294967295
WRITE_INT 0xFFFFFFFF
WRITE_INT PACKET_LEN  # Auto-calculates length (little-endian)
```

**Note**: Use `WRITE_INT_BE` for big-endian format.

---

### `WRITE_INT_BE <value>`
Writes a 32-bit integer in big-endian (network byte order) format.

**Parameters**:
- `<value>`: 32-bit integer value (0-4294967295)
- Special: Use `PACKET_LEN` as the value to automatically calculate and write the packet length

**Examples**:
```
WRITE_INT_BE 4294967295
WRITE_INT_BE PACKET_LEN  # Auto-calculates length (big-endian)
```

---

### `WRITE_STRING <text>`
Writes a null-terminated string to the packet. The string is automatically null-terminated (a `0x00` byte is appended).

**Parameters**:
- `<text>`: String value, optionally quoted. If the string contains spaces, it must be quoted.

**Examples**:
```
WRITE_STRING "Hello Server"
WRITE_STRING "spark tps"  # Strings with spaces must be quoted
WRITE_STRING test  # Simple strings without spaces don't need quotes
```

**Note**: The null terminator (`0x00`) is automatically added after the string.

---

### `WRITE_STRING_LEN <text> <length>`
Writes a fixed-length string to the packet. The string is padded with null bytes if shorter than the specified length, or truncated if longer.

**Parameters**:
- `<text>`: String value, optionally quoted (required if contains spaces)
- `<length>`: Fixed length in bytes

**Examples**:
```
WRITE_STRING_LEN "Test" 10  # Writes "Test" + 6 null bytes
WRITE_STRING_LEN "spark tps" 9  # Writes "spark tps" (9 bytes, no padding)
WRITE_STRING_LEN "Hello" 3  # Writes "Hel" (truncated to 3 bytes)
```

---

### `WRITE_VARINT <value>`
Writes a VarInt (variable-length integer) to the packet. VarInts are used in Minecraft and similar protocols where integers are encoded using a variable number of bytes.

**Parameters**:
- `<value>`: Integer value (0-18446744073709551615)
- Special: Use `PACKET_LEN` as the value to automatically calculate and write the packet length as a VarInt

**Examples**:
```
WRITE_VARINT 300
WRITE_VARINT 0x12
WRITE_VARINT PACKET_LEN  # Auto-calculates length as VarInt
```

**How VarInt works**: 
- Values 0-127 use 1 byte
- Larger values use multiple bytes (each byte has a continuation bit)
- Most significant bit (0x80) indicates more bytes follow

---

### `WRITE_BYTES <hex_string>`
Writes raw hexadecimal bytes to the packet.

**Parameters**:
- `<hex_string>`: Hexadecimal string (without spaces or with spaces, with or without `0x` prefix)

**Examples**:
```
WRITE_BYTES "FF00AA55"
WRITE_BYTES "FF 00 AA 55"
WRITE_BYTES "0xFF00AA55"
```

**Note**: The hex string is decoded and written as raw bytes. Odd-length strings will cause an error.

---

### `PACKET_END`
Marks the end of a packet definition. Must be paired with a preceding `PACKET_START`.

**Example**:
```
PACKET_START
WRITE_BYTE 0xFF
PACKET_END
```

---

## Response Parsing Commands

### `RESPONSE_START`
Marks the beginning of response parsing rules. Must be followed by response parsing commands and closed with `RESPONSE_END`.

**Example**:
```
RESPONSE_START
READ_BYTE packet_id
RESPONSE_END
```

---

### `READ_BYTE <var_name>`
Reads a single byte from the response and stores it in a variable.

**Parameters**:
- `<var_name>`: Variable name to store the byte value (0-255)

**Example**:
```
READ_BYTE packet_id
READ_BYTE status_code
```

**Note**: The variable will be available in the parsed values output and can be referenced in output formatting directives.

---

### `READ_SHORT <var_name>`
Reads a 16-bit integer (little-endian) from the response and stores it in a variable.

**Parameters**:
- `<var_name>`: Variable name to store the integer value

**Example**:
```
READ_SHORT player_count
READ_SHORT port_number
```

**Note**: Use `READ_SHORT_BE` for big-endian format.

---

### `READ_SHORT_BE <var_name>`
Reads a 16-bit integer (big-endian) from the response and stores it in a variable.

**Parameters**:
- `<var_name>`: Variable name to store the integer value

**Example**:
```
READ_SHORT_BE player_count
READ_SHORT_BE port_number
```

---

### `READ_INT <var_name>`
Reads a 32-bit integer (little-endian) from the response and stores it in a variable.

**Parameters**:
- `<var_name>`: Variable name to store the integer value

**Example**:
```
READ_INT server_version
READ_INT response_length
```

**Note**: Use `READ_INT_BE` for big-endian format.

---

### `READ_INT_BE <var_name>`
Reads a 32-bit integer (big-endian) from the response and stores it in a variable.

**Parameters**:
- `<var_name>`: Variable name to store the integer value

**Example**:
```
READ_INT_BE server_version
READ_INT_BE response_length
```

---

### `READ_VARINT <var_name>`
Reads a VarInt (variable-length integer) from the response and stores it in a variable.

**Parameters**:
- `<var_name>`: Variable name to store the integer value

**Example**:
```
READ_VARINT packet_length
READ_VARINT packet_id
```

**Note**: VarInts are commonly used in Minecraft-style protocols. The parser will read bytes until it finds one without the continuation bit set.

---

### `READ_STRING <var_name> <length>`
Reads a fixed-length string from the response.

**Parameters**:
- `<var_name>`: Variable name to store the string value
- `<length>`: Number of bytes to read

**Example**:
```
READ_STRING server_name 32
READ_STRING map_name 64
```

**Note**: The string may contain null bytes. Trailing null bytes are automatically trimmed.

---

### `READ_STRING_NULL <var_name>`
Reads a null-terminated string from the response. Reading stops when a null byte (`0x00`) is encountered.

**Parameters**:
- `<var_name>`: Variable name to store the string value

**Example**:
```
READ_STRING_NULL server_name
READ_STRING_NULL command_output
```

**Note**: The null terminator is consumed but not included in the stored string value.

---

### `SKIP_BYTES <count>`
Skips (advances past) the specified number of bytes in the response without reading them.

**Parameters**:
- `<count>`: Number of bytes to skip

**Example**:
```
SKIP_BYTES 4
SKIP_BYTES 2
```

**Use case**: Useful when you need to skip padding, reserved fields, or other data you don't need to parse.

---

### `RESPONSE_END`
Marks the end of response parsing rules. Must be paired with a preceding `RESPONSE_START`.

**Example**:
```
RESPONSE_START
READ_BYTE packet_id
RESPONSE_END
```

---

## Validation Commands

### `EXPECT_BYTE <value>`
Validates that the next byte in the response matches the expected value. Raises an error if the byte doesn't match.

**Parameters**:
- `<value>`: Expected byte value (0-255) in decimal or hexadecimal

**Example**:
```
EXPECT_BYTE 0xFE
EXPECT_BYTE 0xFF
EXPECT_BYTE 255
```

**Use case**: Validates magic bytes, protocol headers, or status codes. If validation fails, parsing stops and an error is returned.

---

### `EXPECT_MAGIC <hex_string>`
Validates that the next bytes in the response match the expected magic bytes sequence.

**Parameters**:
- `<hex_string>`: Hexadecimal string representing the expected bytes

**Example**:
```
EXPECT_MAGIC "FEEDFACE"
EXPECT_MAGIC "FF FF FF FF"
EXPECT_MAGIC "0xFE0xFD"
```

**Use case**: Validates protocol headers or magic byte sequences. If validation fails, parsing stops and an error is returned.

---

## Output Formatting Directives

Output formatting directives control how results are formatted for Prometheus metrics and UI display. They are placed **outside** of `PACKET_START`/`PACKET_END` and `RESPONSE_START`/`RESPONSE_END` blocks.

### `OUTPUT_SUCCESS`
Marks the beginning of a success output block. All commands between `OUTPUT_SUCCESS` and `OUTPUT_END` are executed when the script completes successfully.

**Example**:
```
OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "net_sentinel_gameserver_up{server='HOST', protocol=JSON_PAYLOAD.version.protocol} 1"
OUTPUT_END
```

---

### `OUTPUT_ERROR`
Marks the beginning of an error output block. All commands between `OUTPUT_ERROR` and `OUTPUT_END` are executed when the script encounters an error.

**Example**:
```
OUTPUT_ERROR
RETURN "net_sentinel_gameserver_up{server='HOST', error=<ERROR REASON>} 0"
OUTPUT_END
```

---

### `OUTPUT_END`
Marks the end of an output block (success or error). Must be paired with either `OUTPUT_SUCCESS` or `OUTPUT_ERROR`.

---

### `JSON_OUTPUT <var_name>`
Parses the named string variable as JSON, making nested fields accessible in output expressions.

**Parameters**:
- `<var_name>`: Name of the variable containing a JSON string

**Example**:
```
JSON_OUTPUT JSON_PAYLOAD
# Now you can reference JSON_PAYLOAD.version.protocol in RETURN statements
```

**Use case**: Many game servers return JSON responses. This command parses the JSON string so you can access nested fields like `JSON_PAYLOAD.version.protocol` or `JSON_PAYLOAD.players.online`.

---

### `RETURN "<expression>"`
Formats the quoted expression into the return label fragment for Prometheus metrics.

**Parameters**:
- `<expression>`: Quoted string expression that may contain:
  - Variable references: `JSON_PAYLOAD.version.protocol`
  - Placeholders: `HOST`, `PORT`, `IP`
  - Error placeholder: `<ERROR REASON>` (only in `OUTPUT_ERROR` blocks)

**Examples**:
```
RETURN "net_sentinel_gameserver_up{server='HOST', protocol=JSON_PAYLOAD.version.protocol} 1"
RETURN "net_sentinel_gameserver_up{server='HOST', error=<ERROR REASON>} 0"
```

**Note**: 
- The server automatically prefixes `net_sentinel_gameserver_up` to your return value
- Placeholders like `HOST`, `PORT`, `IP`, `IP_LEN`, `IP_LEN_HEX` are resolved by the server before execution
- `<ERROR REASON>` is replaced with the actual error message in error blocks

---

## Comments

Lines starting with `#` are treated as comments and ignored by the parser.

**Example**:
```
# This is a comment
PACKET_START
WRITE_BYTE 0xFF  # Inline comment
PACKET_END
```

**Note**: Comments can appear on their own line or at the end of a command line.

---

## Special Features

### Automatic Length Calculation (`PACKET_LEN`)

For protocols that require a length field at the beginning of packets, you can use `PACKET_LEN` as a placeholder. The system will automatically calculate the packet length and write it in the correct format.

**Supported Commands**:
- `WRITE_INT PACKET_LEN` - Calculates length as 4-byte little-endian integer
- `WRITE_INT_BE PACKET_LEN` - Calculates length as 4-byte big-endian integer
- `WRITE_VARINT PACKET_LEN` - Calculates length as VarInt

**How it works**:
1. The placeholder reserves space in the packet
2. After all packet commands are processed, the length is calculated
3. The length field is replaced with the calculated value

**Example (RCON protocol)**:
```
PACKET_START
WRITE_INT PACKET_LEN    # Auto-calculated: 4 (Request ID) + 4 (Type) + len(payload) + 2 (nulls)
WRITE_INT 1             # Request ID
WRITE_INT 3             # Type: SERVERDATA_AUTH
WRITE_STRING "password" # Payload
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END
```

**Note**: The length calculation excludes the length field itself. For example, if the packet (excluding length field) is 15 bytes, the length field will be set to 15.

---

### Sequential Packet/Response Pairs

The system supports multiple packet/response pairs that are executed sequentially. The connection/socket is kept alive for all pairs.

**Example**:
```
# Pair 1: Authentication
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1
WRITE_INT 3
WRITE_STRING "password"
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
READ_INT response_length
READ_INT response_id
READ_INT response_type
READ_STRING_NULL auth_result
RESPONSE_END

# Pair 2: Command execution
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 2
WRITE_INT 2
WRITE_STRING "list"
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
READ_INT response_length
READ_INT response_id
READ_INT response_type
READ_STRING_NULL command_output
RESPONSE_END
```

**Execution order**:
1. Send packet 1 → Receive response 1 → Parse response 1
2. Send packet 2 → Receive response 2 → Parse response 2
3. Continue for all pairs...

---

### Placeholder Variables

The following placeholders are automatically resolved by the server before script execution:

- `HOST` - Server hostname or address
- `PORT` - Server port number
- `IP` - Server IP address
- `IP_LEN` - Length of IP address string
- `IP_LEN_HEX` - Length of IP address in hexadecimal

These placeholders are available in `RETURN` statements but are never shown in response logs.

---

## Complete Examples

### Example 1: Simple UDP Query

```
# Simple UDP server query
PACKET_START
WRITE_BYTE 0xFE
WRITE_BYTE 0xFD
WRITE_BYTE 0x09
WRITE_INT 0x00000000
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xFE
EXPECT_BYTE 0xFD
READ_BYTE packet_type
READ_STRING_NULL session_id
READ_STRING_NULL challenge_token
RESPONSE_END
```

---

### Example 2: RCON Authentication and Command

```
# RCON Authentication
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1
WRITE_INT 3
WRITE_STRING "my_rcon_password"
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
READ_INT response_length
READ_INT response_id
READ_INT response_type
READ_STRING_NULL auth_result
SKIP_BYTES 2
RESPONSE_END

# RCON Command Execution
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 2
WRITE_INT 2
WRITE_STRING "list"
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
READ_INT response_length
READ_INT response_id
READ_INT response_type
READ_STRING_NULL command_output
SKIP_BYTES 2
RESPONSE_END
```

---

### Example 3: Minecraft VarInt Protocol

```
# Minecraft-style packet with VarInt length
PACKET_START
WRITE_VARINT PACKET_LEN
WRITE_VARINT 0
WRITE_VARINT 0
WRITE_STRING "ping"
PACKET_END

RESPONSE_START
READ_VARINT packet_length
READ_VARINT packet_id
READ_STRING_NULL response_data
RESPONSE_END
```

---

### Example 4: With Output Formatting

```
PACKET_START
WRITE_STRING "status"
PACKET_END

RESPONSE_START
READ_STRING_NULL JSON_PAYLOAD
RESPONSE_END

OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "net_sentinel_gameserver_up{server='HOST', protocol=JSON_PAYLOAD.version.protocol, players=JSON_PAYLOAD.players.online} 1"
OUTPUT_END

OUTPUT_ERROR
RETURN "net_sentinel_gameserver_up{server='HOST', error=<ERROR REASON>} 0"
OUTPUT_END
```

---

## Value Formats

### Decimal Numbers
```
WRITE_BYTE 255
WRITE_SHORT 1234
WRITE_INT 4294967295
```

### Hexadecimal Numbers
```
WRITE_BYTE 0xFF
WRITE_SHORT 0x1234
WRITE_INT 0xFFFFFFFF
```

### Strings
```
WRITE_STRING "Hello World"        # Quoted (required for strings with spaces)
WRITE_STRING test                 # Unquoted (only for simple strings)
WRITE_STRING_LEN "Test" 10       # Fixed length
```

### Hex Bytes
```
WRITE_BYTES "FF00AA55"           # Without spaces
WRITE_BYTES "FF 00 AA 55"        # With spaces
WRITE_BYTES "0xFF00AA55"         # With 0x prefix
```

---

## Error Handling

If any command fails:
- **Packet construction errors**: Script execution stops, error is returned
- **Network errors**: Connection is closed, error is returned
- **Response validation errors**: Parsing stops, error is returned (e.g., `EXPECT_BYTE` mismatch)
- **Response parsing errors**: Parsing stops, error is returned (e.g., insufficient data)

All errors include:
- Error type (SyntaxError, NetworkError, ValidationError, ParseError)
- Error message
- Line number (for syntax errors)

---

## Best Practices

1. **Always validate responses**: Use `EXPECT_BYTE` or `EXPECT_MAGIC` to verify protocol headers
2. **Use appropriate endianness**: Match the protocol's byte order (little-endian vs big-endian)
3. **Handle null terminators**: Use `READ_STRING_NULL` for null-terminated strings, `SKIP_BYTES` for padding
4. **Test incrementally**: Start with simple packets, then add complexity
5. **Use comments**: Document your packet structure for future reference
6. **Leverage PACKET_LEN**: Use automatic length calculation when the protocol requires it
7. **Parse sequentially**: Read fields in the exact order they appear in the response

---

## Protocol-Specific Notes

### RCON Protocol
- Uses fixed 4-byte little-endian integers (not VarInts)
- Requires `WRITE_INT PACKET_LEN` for length field
- Uses null-terminated strings (`WRITE_STRING`, `READ_STRING_NULL`)
- Requires two null bytes (`0x00 0x00`) at the end of packets

### Minecraft Protocol
- Uses VarInts for packet lengths and IDs
- Use `WRITE_VARINT PACKET_LEN` for length field
- Strings are length-prefixed with VarInt

### Source Engine
- Uses big-endian integers
- Use `WRITE_INT_BE` and `READ_INT_BE`
- Magic bytes: `0xFF 0xFF 0xFF 0xFF`

---

## Troubleshooting

### "Invalid length at line X"
- Check that `WRITE_STRING_LEN` has both text and length parameters
- Ensure quoted strings with spaces are properly quoted

### "Insufficient data" errors
- Response may be shorter than expected
- Check if you're reading past the end of the response
- Verify the protocol matches your expectations

### "Expected byte X, got Y"
- Protocol may have changed
- Check if you're using the correct magic bytes
- Verify endianness (little-endian vs big-endian)

### Connection issues
- Ensure the server is running and accessible
- Check firewall settings
- Verify port number is correct
- For TCP, ensure the connection stays alive (it does automatically)

---

## Summary

This pseudo-code language provides a flexible way to define custom packet protocols for game server monitoring. By combining packet construction, response parsing, and output formatting, you can create monitoring scripts for virtually any game server protocol.

For more examples and protocol-specific guides, see:
- `rcon.md` - RCON protocol guide
- `gameserver_check.md` - Implementation guide
- `minecraft_pseudo-code.md` - Minecraft-specific examples

