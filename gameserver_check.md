# Game Server Checker - Implementation Guide

## Overview

This document explains how to implement a dynamic game server checker that can ping game servers using custom packet protocols. The system allows users to define packet structures using a simple pseudo-code format, which is then interpreted to construct and send packets via UDP or TCP.

## Core Requirements

1. **Protocol Selection**: Each game server can use either UDP or TCP (not both)
2. **Dynamic Packet Construction**: Packets are built dynamically based on user-defined rules
3. **Pseudo-code Interpreter**: A simple scripting language to define packet structure and response parsing
4. **Web Interface**: Editor for writing pseudo-code and viewing responses/errors
5. **Response Parsing**: Extract relevant information from server responses

## Architecture

### Data Model

Each game server entry should contain:
- **Name**: Display name for the server
- **Address**: IP address or hostname
- **Port**: Server port number
- **Protocol**: Either "UDP" or "TCP" (enum/string)
- **Pseudo-code**: Script defining packet construction and response parsing
- **Timeout**: Maximum time to wait for response (in milliseconds)

### Packet Construction Flow

```
User Input (Pseudo-code) 
  → Parser 
    → Packet Builder 
      → Network Layer (UDP/TCP)
        → Send to Server
          → Receive Response
            → Response Parser (based on pseudo-code)
              → Display Result
```

## Pseudo-code Language Specification

### Basic Syntax

The pseudo-code uses a simple, line-based syntax with commands that define:
1. **Packet Construction**: How to build the request packet
2. **Response Parsing**: How to interpret the server's response
3. **Variables**: Store values for reuse

### Packet Construction Commands

#### `PACKET_START`
Marks the beginning of packet definition.

#### `WRITE_BYTE <value>`
Writes a single byte (0-255) to the packet.

**Example**: `WRITE_BYTE 0xFF`

#### `WRITE_SHORT <value>`
Writes a 16-bit integer (little-endian by default).

**Example**: `WRITE_SHORT 1234`

#### `WRITE_SHORT_BE <value>`
Writes a 16-bit integer in big-endian format.

**Example**: `WRITE_SHORT_BE 1234`

#### `WRITE_INT <value>`
Writes a 32-bit integer (little-endian by default).

**Example**: `WRITE_INT 4294967295`

#### `WRITE_INT_BE <value>`
Writes a 32-bit integer in big-endian format.

**Example**: `WRITE_INT_BE 4294967295`

#### `WRITE_STRING <text>`
Writes a null-terminated string.

**Example**: `WRITE_STRING "Hello Server"`

#### `WRITE_STRING_LEN <text> <length>`
Writes a fixed-length string (padded or truncated).

**Example**: `WRITE_STRING_LEN "Test" 10`

#### `WRITE_BYTES <hex_string>`
Writes raw hex bytes.

**Example**: `WRITE_BYTES "FF00AA55"`

#### `PACKET_END`
Marks the end of packet definition.

### Response Parsing Commands

#### `RESPONSE_START`
Marks the beginning of response parsing rules.

#### `READ_BYTE <var_name>`
Reads a single byte from response and stores it in a variable.

**Example**: `READ_BYTE packet_id`

#### `READ_SHORT <var_name>`
Reads a 16-bit integer (little-endian) from response.

**Example**: `READ_SHORT player_count`

#### `READ_SHORT_BE <var_name>`
Reads a 16-bit integer (big-endian) from response.

**Example**: `READ_SHORT_BE player_count`

#### `READ_INT <var_name>`
Reads a 32-bit integer (little-endian) from response.

**Example**: `READ_INT server_version`

#### `READ_INT_BE <var_name>`
Reads a 32-bit integer (big-endian) from response.

**Example**: `READ_INT_BE server_version`

#### `READ_STRING <var_name> <length>`
Reads a string of specified length.

**Example**: `READ_STRING server_name 32`

#### `READ_STRING_NULL <var_name>`
Reads a null-terminated string.

**Example**: `READ_STRING_NULL server_name`

#### `READ_VARINT <var_name>`
Reads a VarInt (variable-length integer) from the response, storing it in the named variable. Useful for lengths and packet IDs in Minecraft-style protocols.

**Example**: `READ_VARINT LENGTH_VARINT`

#### `SKIP_BYTES <count>`
Skips the specified number of bytes in the response.

**Example**: `SKIP_BYTES 4`

#### `RESPONSE_END`
Marks the end of response parsing.

### Validation Commands

#### `EXPECT_BYTE <value>`
Validates that the next byte matches the expected value. Raises error if not.

**Example**: `EXPECT_BYTE 0xFE`

#### `EXPECT_MAGIC <hex_string>`
Validates that the next bytes match the expected magic bytes.

**Example**: `EXPECT_MAGIC "FEEDFACE"`

### Comments

Lines starting with `#` are treated as comments and ignored.

**Example**: `# This is a comment`

### Output formatting directives

Net Sentinel emits structured output blocks after each script run so Prometheus/UI can consume parsed values. These commands wrap the parsed response and control whether the run is considered successful or failed.

#### `OUTPUT_SUCCESS`
Signals the script completed without errors. Follow it with metrics or structured data (e.g., raw response hex, response time, player count) that the collector will include in the unpublished meter block.

#### `OUTPUT_ERROR`
Mark failures (syntax, network, validation) with this directive. Include fields such as `error_type`, `message`, and the script line number so the UI and exporter know what went wrong and can emit an error counter.

#### `OUTPUT_END`
Denotes the end of the output block (success or error). The downstream handler flushes its metrics/buffer only after seeing `OUTPUT_END`, ensuring the block is complete before the next test starts.

#### Example: Prometheus output
In your test script append the output block below to emit the metric line:

```
OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "net_sentinel_gameserver_up{server='HOST', protocol=JSON_PAYLOAD.version.protocol} 1"
OUTPUT_END

OUTPUT_ERROR
RETURN "net_sentinel_gameserver_up{server='HOST', error=<ERROR REASON>} 0"
OUTPUT_END
```

`JSON_OUTPUT` converts the parsed `JSON_PAYLOAD` string into usable JSON, and `RETURN` formats the label set. `<ERROR REASON>` is replaced with the current error message in the failure block, so Prometheus sees either a success (1) line with the protocol label or an error (0) line with the reason.

#### `JSON_OUTPUT <var_name>`
Parses the named string variable (e.g., `JSON_PAYLOAD`) into JSON so the output block can reference nested fields like `JSON_PAYLOAD.version.protocol`.

#### `RETURN "<expression>"`
Formats the quoted expression into the return label fragment; the server automatically prefixes `net_sentinel_gameserver_up`. You may reference placeholders (e.g., `HOST`, `PORT`, `JSON_PAYLOAD.version.protocol`), and `<ERROR REASON>` is replaced with the actual error message when the block runs in error mode. Placeholders are expanded before the return is evaluated, and they are never sent back to the editor panel.

HOST, HOST_LEN, IP, IP_LEN, IP_LEN_HEX, and PORT are resolved by the server before the script executes, so you never see them in the response log—they exist only during evaluation.

### JSON conversion helpers

Many responses contain strings (e.g., `JSON_PAYLOAD`) that should be interpreted as JSON objects. The parser can run `serde_json::from_str` on those strings; when successful, the resulting object is serialized back into escaped JSON and fed to `OUTPUT_SUCCESS` (so dashboards can show structured data such as `players`, `version`, `motd`, etc.). If parsing fails, emit `OUTPUT_ERROR` with the raw string so the handler still captures the payload for debugging.

## Example Pseudo-code Scripts

### Example 1: Simple Minecraft-style Query (UDP)

```
# Minecraft-style server query
PACKET_START
WRITE_BYTE 0xFE  # Magic byte
WRITE_BYTE 0xFD  # Query packet ID
WRITE_BYTE 0x09  # Challenge token
WRITE_INT 0x00000000  # Session ID
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xFE  # Response magic
EXPECT_BYTE 0xFD  # Response type
READ_BYTE packet_type
READ_STRING_NULL session_id
READ_STRING_NULL challenge_token
RESPONSE_END
```

### Example 2: Source Engine Query (UDP)

```
# Source Engine A2S_INFO query
PACKET_START
WRITE_STRING "Source Engine Query"
WRITE_BYTE 0x00
WRITE_BYTE 0x54  # Challenge response
WRITE_INT 0xFFFFFFFF  # -1 for challenge request
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xFF
EXPECT_BYTE 0xFF
EXPECT_BYTE 0xFF
EXPECT_BYTE 0xFF
READ_BYTE header
READ_STRING_NULL server_name
READ_STRING_NULL map_name
READ_STRING_NULL game_folder
READ_STRING_NULL game_name
READ_SHORT game_id
READ_BYTE player_count
READ_BYTE max_players
RESPONSE_END
```

### Example 3: Custom Protocol (TCP)

```
# Custom game server protocol
PACKET_START
WRITE_BYTE 0x01  # Command: STATUS
WRITE_SHORT 1234  # Request ID
WRITE_STRING "STATUS"
PACKET_END

RESPONSE_START
READ_BYTE response_code
IF response_code == 0x00
    READ_SHORT request_id
    READ_BYTE status
    READ_STRING_NULL message
ELSE
    ERROR "Invalid response code"
END
RESPONSE_END
```

## Implementation Details

### Packet Builder

The packet builder should:
1. Parse the pseudo-code line by line
2. Maintain a buffer to construct the packet
3. Handle endianness correctly for multi-byte values
4. Validate input values (e.g., bytes 0-255, strings within length limits)
5. Return a byte vector ready to send

**Error Handling**:
- Invalid command syntax → Error message with line number
- Value out of range → Error message with expected range
- Missing PACKET_START/END → Error message

### Network Layer

#### UDP Implementation
- Create UDP socket
- Send packet to server address:port
- Wait for response with timeout
- Handle connection refused errors
- Handle timeout errors

#### TCP Implementation
- Create TCP socket
- Connect to server address:port
- Send packet
- Read response until timeout or connection close
- Handle connection errors
- Handle timeout errors

### Response Parser

The response parser should:
1. Parse the RESPONSE_START to RESPONSE_END section
2. Maintain a cursor position in the response buffer
3. Extract values based on commands
4. Store parsed values in variables
5. Validate expected values
6. Return parsed data structure

**Error Handling**:
- Unexpected byte values → Error message showing expected vs received
- Insufficient data → Error message showing required vs available bytes
- Invalid command syntax → Error message with line number

### Variable Storage

During response parsing, maintain a dictionary/map of variable names to values:
- Variables can be strings, integers, or bytes
- Variables are available for display in the UI
- Variables can be used in conditional logic (future enhancement)

## Web Interface Requirements

### Editor Section

1. **Text Editor**:
   - Syntax highlighting for pseudo-code commands
   - Line numbers
   - Auto-indentation
   - Error highlighting (red squiggles for invalid lines)
   - Real-time validation feedback

2. **Server Configuration**:
   - Name input field
   - Address input field
   - Port input field
   - Protocol selector (UDP/TCP radio buttons - mutually exclusive)
   - Timeout input field (default: 5000ms)

3. **Action Buttons**:
   - "Test Connection" button: Sends packet and shows response
   - "Save Configuration" button: Saves the game server entry
   - "Clear" button: Clears the editor

### Response Viewer Section

1. **Response Display**:
   - **Raw Response**: Hexadecimal dump of received bytes
   - **Parsed Values**: Table showing variable names and their parsed values
   - **Status Indicator**: Green (success) / Red (error) indicator
   - **Response Time**: Time taken to receive response

2. **Error Display**:
   - Error messages in red text
   - Error location (line number in pseudo-code)
   - Error type (parsing error, network error, validation error)
   - Suggestions for fixing common errors

3. **Raw Data View**:
   - Hexadecimal view with offsets
   - ASCII view alongside hex
   - Ability to copy raw response

### Response Display Format

```
Status: ✅ Success (Response received in 45ms)

Raw Response (24 bytes):
00000000: FF FF FF FF 49 02 54 65  73 74 20 53 65 72 76 65  ....I.Test Serve
00000010: 72 20 4E 61 6D 65 00 00                          r Name..

Parsed Values:
┌─────────────────────┬─────────────────────┐
│ Variable            │ Value               │
├─────────────────────┼─────────────────────┤
│ header              │ 0xFF                │
│ packet_type         │ 0x49 (73)           │
│ server_name         │ "Test Server Name"  │
│ player_count        │ 54                  │
└─────────────────────┴─────────────────────┘
```

### Error Display Format

```
Status: ❌ Error

Error Type: Response Validation Error
Location: Line 8 in pseudo-code
Message: Expected byte 0xFE, but received 0xFF

Raw Response (12 bytes):
00000000: FF FF FF FF 49 02 54 65  73 74 20 53 65           .........Test Se

Suggestion: Check if the server protocol matches your pseudo-code definition.
```

## Database Schema

Add to the JSON database structure:

```json
{
  "game_servers": [
    {
      "id": 1,
      "name": "My Minecraft Server",
      "address": "example.com",
      "port": 25565,
      "protocol": "UDP",
      "timeout_ms": 5000,
      "pseudo_code": "PACKET_START\nWRITE_BYTE 0xFE\n...",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## API Endpoints

### GET /api/gameservers
Returns list of all game servers.

### GET /api/gameservers/:id
Returns a specific game server configuration.

### POST /api/gameservers
Creates a new game server entry.
Request body:
```json
{
  "name": "Server Name",
  "address": "example.com",
  "port": 25565,
  "protocol": "UDP",
  "timeout_ms": 5000,
  "pseudo_code": "PACKET_START\n..."
}
```

### PUT /api/gameservers/:id
Updates an existing game server entry.

### DELETE /api/gameservers/:id
Deletes a game server entry.

### POST /api/gameservers/:id/test
Tests the game server connection using the configured pseudo-code.
Returns:
```json
{
  "success": true,
  "response_time_ms": 45,
  "raw_response": "FF00AA55...",
  "parsed_values": {
    "server_name": "Test Server",
    "player_count": 42
  },
  "error": null
}
```

Or on error:
```json
{
  "success": false,
  "response_time_ms": 0,
  "raw_response": null,
  "parsed_values": null,
  "error": {
    "type": "NetworkError",
    "message": "Connection timed out",
    "line": null
  }
}
```

## Metrics Output

Add to Prometheus metrics:

```
# HELP net_sentinel_gameserver_up Game server connectivity status (1 = up, 0 = down)
# TYPE net_sentinel_gameserver_up gauge
net_sentinel_gameserver_up{name="My Minecraft Server",address="example.com",port="25565"} 1
```

## Error Types

1. **SyntaxError**: Invalid pseudo-code syntax
   - Missing PACKET_START/END
   - Invalid command
   - Invalid parameter format

2. **NetworkError**: Network-related errors
   - Connection refused
   - Connection timed out
   - DNS resolution failed
   - Socket creation failed

3. **ValidationError**: Response validation errors
   - Unexpected byte value
   - Magic bytes mismatch
   - Response too short

4. **ParseError**: Response parsing errors
   - Invalid data type
   - Out of bounds read
   - String parsing error

## Future Enhancements

1. **Conditional Logic**: Add IF/ELSE statements to pseudo-code
2. **Loops**: Add FOR/WHILE loops for repeated structures
3. **Functions**: Define reusable packet patterns
4. **Packet Templates**: Pre-built templates for common game servers
5. **Response Comparison**: Compare responses over time for changes
6. **Multi-packet Queries**: Support for query-response-challenge patterns
7. **Hex Editor**: Visual hex editor for constructing packets
8. **Response Replay**: Save and replay responses for testing

## Testing Considerations

1. **Unit Tests**: Test packet builder with various pseudo-code scripts
2. **Network Tests**: Test with actual game servers (Minecraft, Source Engine, etc.)
3. **Error Cases**: Test all error paths (timeouts, invalid responses, etc.)
4. **Protocol Tests**: Verify UDP vs TCP behavior differences
5. **Edge Cases**: Test with maximum packet sizes, empty responses, etc.

## Security Considerations

1. **Input Validation**: Sanitize all user inputs
2. **Packet Size Limits**: Prevent DoS by limiting packet size
3. **Timeout Limits**: Prevent resource exhaustion with reasonable timeouts
4. **Rate Limiting**: Limit number of test requests per user
5. **DDoS Protection**: Consider rate limiting at network level

