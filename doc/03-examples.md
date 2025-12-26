# Pseudo-Code Examples and Tutorials

Real-world examples of pseudo-code scripts for different game server protocols.

## Table of Contents

1. [Simple UDP Query](#simple-udp-query)
2. [RCON Protocol](#rcon-protocol)
3. [Minecraft Protocol](#minecraft-protocol)
4. [Source Engine Query](#source-engine-query)
5. [HTTP/HTTPS REST API Examples](#httphttps-rest-api-examples)
6. [Custom Protocol Examples](#custom-protocol-examples)

## Simple UDP Query

A basic UDP server query that sends a magic byte sequence and reads the response.

### Protocol Description
- **Request**: Send magic bytes `0xFE 0xFD 0x09` followed by session ID `0x00000000`
- **Response**: Receive magic bytes `0xFE 0xFD`, then packet type, session ID, and challenge token

### Script

```pseudo
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

OUTPUT_SUCCESS
RETURN "type=packet_type, session=session_id"
OUTPUT_END
```

### Explanation

1. **Packet Construction:**
   - `WRITE_BYTE 0xFE` - First magic byte
   - `WRITE_BYTE 0xFD` - Second magic byte
   - `WRITE_BYTE 0x09` - Packet type
   - `WRITE_INT 0x00000000` - Session ID (4 bytes)

2. **Response Parsing:**
   - `EXPECT_BYTE 0xFE` - Validate first magic byte
   - `EXPECT_BYTE 0xFD` - Validate second magic byte
   - `READ_BYTE packet_type` - Read packet type
   - `READ_STRING_NULL session_id` - Read null-terminated session ID
   - `READ_STRING_NULL challenge_token` - Read null-terminated challenge token

## RCON Protocol

RCON (Remote Console) is used by many game servers for remote administration. It requires authentication followed by command execution.

### Protocol Description
- **Authentication**: Send packet with length, request ID, type (3 = auth), password, and two null bytes
- **Response**: Receive length, response ID, type, and result string
- **Command**: Send packet with length, request ID, type (2 = command), command string, and two null bytes
- **Response**: Receive length, response ID, type, and output string

### Complete Script

```pseudo
# Authentication Packet
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

# Command Execution Packet
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

OUTPUT_SUCCESS
RETURN "output=command_output"
OUTPUT_END

OUTPUT_ERROR
RETURN "error=<ERROR REASON>"
OUTPUT_END
```

### Explanation

1. **Authentication:**
   - `WRITE_INT PACKET_LEN` - Auto-calculates packet length
   - `WRITE_INT 1` - Request ID
   - `WRITE_INT 3` - Type: SERVERDATA_AUTH (3)
   - `WRITE_STRING "my_rcon_password"` - Password (null-terminated)
   - `WRITE_BYTE 0x00` - First null byte
   - `WRITE_BYTE 0x00` - Second null byte

2. **Response Parsing:**
   - `READ_INT response_length` - Packet length
   - `READ_INT response_id` - Response ID (should match request ID)
   - `READ_INT response_type` - Response type
   - `READ_STRING_NULL auth_result` - Authentication result
   - `SKIP_BYTES 2` - Skip trailing null bytes

3. **Command Execution:**
   - Similar structure, but type is `2` (SERVERDATA_EXECCOMMAND)
   - Command string replaces password

### Using Variables (Advanced)

You can use code blocks to make the script more maintainable:

```pseudo
CODE_START
STRING password = "my_rcon_password"
INT auth_id = 1
INT command_id = 2
STRING command = "list"
CODE_END

# Authentication
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT auth_id
WRITE_INT 3
WRITE_STRING password
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

# Command
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT command_id
WRITE_INT 2
WRITE_STRING command
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

## Minecraft Protocol

Minecraft uses VarInt encoding for packet lengths and IDs. This example queries server status.

### Protocol Description
- **Handshake**: Send VarInt packet length, VarInt packet ID (0), VarInt protocol version, VarInt hostname length, hostname string, short port, VarInt next state (1)
- **Status Request**: Send VarInt packet length (1), VarInt packet ID (0)
- **Response**: Receive VarInt length, VarInt packet ID (0), VarInt JSON length, JSON string

### Complete Script

```pseudo
# Handshake Packet
PACKET_START
WRITE_VARINT PACKET_LEN
WRITE_VARINT 0x00
WRITE_VARINT 0x47
WRITE_VARINT IP_LEN
WRITE_STRING IP
WRITE_SHORT_BE PORT
WRITE_VARINT 0x01
PACKET_END

# Status Request Packet
PACKET_START
WRITE_VARINT 0x01
WRITE_VARINT 0x00
PACKET_END

RESPONSE_START
READ_VARINT length_varint
READ_VARINT packet_id
READ_VARINT json_length_varint
READ_STRING JSON_PAYLOAD json_length_varint
RESPONSE_END

OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "protocol=JSON_PAYLOAD.version.protocol, players=JSON_PAYLOAD.players.online, max=JSON_PAYLOAD.players.max"
OUTPUT_END

OUTPUT_ERROR
RETURN "error=<ERROR REASON>"
OUTPUT_END
```

### Explanation

1. **Handshake Packet:**
   - `WRITE_VARINT PACKET_LEN` - Auto-calculates packet length as VarInt
   - `WRITE_VARINT 0x00` - Packet ID: Handshake (0)
   - `WRITE_VARINT 0x47` - Protocol version (71 in decimal)
   - `WRITE_VARINT IP_LEN` - Hostname length (auto-resolved)
   - `WRITE_STRING IP` - Hostname (auto-resolved)
   - `WRITE_SHORT_BE PORT` - Port (big-endian, auto-resolved)
   - `WRITE_VARINT 0x01` - Next state: Status (1)

2. **Status Request:**
   - `WRITE_VARINT 0x01` - Packet length (1 byte for packet ID)
   - `WRITE_VARINT 0x00` - Packet ID: Status Request (0)

3. **Response Parsing:**
   - `READ_VARINT length_varint` - Packet length
   - `READ_VARINT packet_id` - Packet ID (should be 0)
   - `READ_VARINT json_length_varint` - JSON string length
   - `READ_STRING JSON_PAYLOAD json_length_varint` - JSON string

4. **Output Formatting:**
   - `JSON_OUTPUT JSON_PAYLOAD` - Parse JSON string
   - Access nested fields: `JSON_PAYLOAD.version.protocol`

## Source Engine Query

Source Engine (used by games like Counter-Strike, Team Fortress 2) uses big-endian integers and specific magic bytes.

### Protocol Description
- **Request**: Send magic bytes `0xFF 0xFF 0xFF 0xFF` followed by string `"TSource Engine Query"` and null byte
- **Response**: Receive magic bytes, header byte, protocol version, server name, map name, game directory, game description, and various server stats

### Script

```pseudo
PACKET_START
WRITE_BYTE 0xFF
WRITE_BYTE 0xFF
WRITE_BYTE 0xFF
WRITE_BYTE 0xFF
WRITE_STRING "TSource Engine Query"
PACKET_END

RESPONSE_START
EXPECT_MAGIC "FFFFFFFF"
READ_BYTE header
READ_STRING_NULL protocol_version
READ_STRING_NULL server_name
READ_STRING_NULL map_name
READ_STRING_NULL game_directory
READ_STRING_NULL game_description
READ_SHORT_BE app_id
READ_BYTE player_count
READ_BYTE max_players
READ_BYTE bot_count
READ_BYTE server_type
READ_BYTE environment
READ_BYTE visibility
READ_BYTE vac
RESPONSE_END

OUTPUT_SUCCESS
RETURN "name=server_name, map=map_name, players=player_count, max=max_players"
OUTPUT_END
```

### Explanation

1. **Packet Construction:**
   - `WRITE_BYTE 0xFF` (x4) - Magic bytes
   - `WRITE_STRING "TSource Engine Query"` - Query string

2. **Response Parsing:**
   - `EXPECT_MAGIC "FFFFFFFF"` - Validate magic bytes
   - Read various server information fields
   - Use `READ_SHORT_BE` for big-endian integers

## HTTP/HTTPS REST API Examples

HTTP/HTTPS support allows monitoring REST APIs, web services, and HTTP-based endpoints. Unlike TCP/UDP which sends raw binary packets, HTTP/HTTPS constructs proper HTTP requests with methods, headers, query parameters, and request bodies.

### Protocol Selection

When adding a game server or endpoint, set the protocol to either:
- `HTTP` - For unencrypted HTTP connections (default port 80)
- `HTTPS` - For encrypted HTTPS connections (default port 443)

**Port Handling:**
- If no port is specified, the default port is used (80 for HTTP, 443 for HTTPS)
- If a port is specified, it will be used (e.g., `8080` for HTTP on non-standard port)

### Example 1: Simple GET Request

A basic GET request to retrieve server status.

```pseudo
HTTP_START REQUEST GET /api/status
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
EXPECT_HEADER Content-Type application/json
READ_BODY_JSON response
RESPONSE_END

OUTPUT_SUCCESS
RETURN "status=response.status, uptime=response.uptime"
OUTPUT_END
```

**Explanation:**
1. **Request Construction:**
   - `HTTP_START REQUEST GET /api/status` - Creates a GET request to `/api/status`
   - `HTTP_END` - Closes the request block

2. **Response Parsing:**
   - `EXPECT_STATUS 200` - Validates successful response
   - `EXPECT_HEADER Content-Type application/json` - Validates JSON response
   - `READ_BODY_JSON response` - Parses JSON response into `response` variable

3. **Output:**
   - Accesses nested JSON fields: `response.status`, `response.uptime`

### Example 2: GET Request with Query Parameters

A GET request with query parameters for filtering/searching.

```pseudo
HTTP_START REQUEST GET /api/search
PARAM q hello
PARAM limit 10
PARAM page 1
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
READ_BODY_JSON results
RESPONSE_END

OUTPUT_SUCCESS
RETURN "count=results.data.length, total=results.total"
OUTPUT_END
```

**Explanation:**
- `PARAM` commands add query parameters: `/api/search?q=hello&limit=10&page=1`
- Parameters are automatically URL-encoded

### Example 3: POST Request with JSON Body

A POST request to create a resource with JSON data.

```pseudo
HTTP_START REQUEST POST /api/users
HEADER Authorization Bearer abc123token
HEADER User-Agent NetSentinel/1.0
BODY_START TYPE RAW
DATA {
  "name": "John Doe",
  "email": "john@example.com",
  "active": true
}
BODY_END
HTTP_END

RESPONSE_START
EXPECT_STATUS 201
EXPECT_HEADER Content-Type application/json
READ_BODY_JSON user
RESPONSE_END

OUTPUT_SUCCESS
RETURN "user_id=user.id, created_at=user.created_at"
OUTPUT_END
```

**Explanation:**
1. **Request Construction:**
   - `HEADER Authorization Bearer abc123token` - Adds authentication header
   - `BODY_START TYPE RAW` - Specifies raw body (JSON)
   - `DATA {...}` - JSON body (automatically stringified)
   - `BODY_END` - Closes body section

2. **Response:**
   - Expects status `201` (Created)
   - Parses JSON response into `user` variable

### Example 4: POST Request with Form Data

A POST request with URL-encoded form data.

```pseudo
HTTP_START REQUEST POST /api/login
BODY_START TYPE FORM
DATA username=admin
DATA password=secret123
DATA remember=true
BODY_END
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
READ_BODY_JSON login_response
RESPONSE_END

OUTPUT_SUCCESS
RETURN "token=login_response.token, expires=login_response.expires_at"
OUTPUT_END
```

**Explanation:**
- `BODY_START TYPE FORM` - Specifies form-encoded body
- Form data is automatically URL-encoded
- Content-Type is automatically set to `application/x-www-form-urlencoded`

### Example 5: Multiple Requests (Authentication Flow)

A complete example showing authentication followed by an authenticated request.

```pseudo
# First request: Login
HTTP_START REQUEST POST /api/login
BODY_START TYPE RAW
DATA {"username": "admin", "password": "secret"}
BODY_END
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
READ_BODY_JSON login_response
RESPONSE_END

# Second request: Get user info (using token from first response)
HTTP_START REQUEST GET /api/users/me
HEADER Authorization Bearer login_response.token
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
READ_BODY_JSON user_info
RESPONSE_END

OUTPUT_SUCCESS
RETURN "username=user_info.username, email=user_info.email"
OUTPUT_END
```

**Explanation:**
- First request authenticates and receives a token
- Second request uses the token from `login_response.token` in the Authorization header
- Variables from previous responses can be used in subsequent requests

### Example 6: Reading Plain Text Response

A request that reads a plain text response instead of JSON.

```pseudo
HTTP_START REQUEST GET /api/health
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
EXPECT_HEADER Content-Type text/plain
READ_BODY responseText
RESPONSE_END

OUTPUT_SUCCESS
RETURN "health=responseText"
OUTPUT_END
```

**Explanation:**
- `READ_BODY responseText` - Reads response as raw UTF-8 text
- Useful for plain text, HTML, XML, or other non-JSON formats

### Example 7: Using Variables in Requests

Using variables for dynamic request construction.

```pseudo
CODE_START
STRING api_key = "abc123"
STRING endpoint = "/api/data"
INT page = 1
INT limit = 10
CODE_END

HTTP_START REQUEST GET endpoint
HEADER X-API-Key api_key
PARAM page page
PARAM limit limit
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
READ_BODY_JSON data
RESPONSE_END

OUTPUT_SUCCESS
RETURN "items=data.items.length, total=data.total"
OUTPUT_END
```

**Explanation:**
- Variables can be used in paths, headers, and query parameters
- Makes scripts more maintainable and reusable

### Example 8: Error Handling

Proper error handling for HTTP requests.

```pseudo
HTTP_START REQUEST GET /api/status
HTTP_END

RESPONSE_START
EXPECT_STATUS 200
READ_BODY_JSON response
RESPONSE_END

OUTPUT_SUCCESS
RETURN "status=response.status, uptime=response.uptime"
OUTPUT_END

OUTPUT_ERROR
RETURN "error=<ERROR REASON>"
OUTPUT_END
```

**Common Error Scenarios:**
- Connection errors: Host unreachable, DNS resolution failure
- Timeout: Server doesn't respond within timeout period
- SSL/TLS errors: Invalid certificates (for HTTPS)
- Status code mismatches: When `EXPECT_STATUS` doesn't match
- Header mismatches: When `EXPECT_HEADER` doesn't match
- JSON parsing errors: When `READ_BODY_JSON` fails to parse
- Text parsing errors: When `READ_BODY` fails to decode as UTF-8

### HTTPS vs HTTP

The protocol selection (`HTTP` vs `HTTPS`) determines:
- **HTTP**: Unencrypted connection on port 80 (default)
- **HTTPS**: Encrypted TLS/SSL connection on port 443 (default)

All syntax and commands are identical between HTTP and HTTPS. The only difference is:
- The underlying connection uses TLS encryption for HTTPS
- Certificate validation is performed automatically
- Self-signed certificates may cause connection failures (this is expected behavior)

## Custom Protocol Examples

### Example 1: Simple Text Protocol

A protocol that sends a command and receives a text response.

```pseudo
PACKET_START
WRITE_STRING "status"
PACKET_END

RESPONSE_START
READ_STRING_NULL status_info
RESPONSE_END

OUTPUT_SUCCESS
RETURN "status=status_info"
OUTPUT_END
```

### Example 2: Binary Protocol with Header

A protocol with a header byte, command byte, and data.

```pseudo
PACKET_START
WRITE_BYTE 0xAA        # Header
WRITE_BYTE 0x01        # Command: Get Status
WRITE_SHORT_BE 1234    # Some parameter
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xAA       # Validate header
READ_BYTE response_cmd
READ_SHORT_BE status_code
READ_STRING_NULL message
RESPONSE_END

OUTPUT_SUCCESS
RETURN "code=status_code, message=message"
OUTPUT_END
```

### Example 3: Protocol with Length Field

A protocol that prefixes packets with their length.

```pseudo
PACKET_START
WRITE_INT PACKET_LEN   # Auto-calculated length
WRITE_BYTE 0x01        # Command
WRITE_STRING "query"   # Data
PACKET_END

RESPONSE_START
READ_INT response_length
READ_BYTE response_type
READ_STRING response_data response_length - 5  # Length minus header bytes
RESPONSE_END
```

### Example 4: Protocol with Multiple Responses

Some protocols send multiple packets in response. Handle this by reading until you get all data.

```pseudo
PACKET_START
WRITE_BYTE 0x01
PACKET_END

RESPONSE_START
READ_BYTE packet_count
# Note: You may need to read multiple packets
# This depends on the specific protocol
FOR i IN 0..packet_count:
  READ_STRING_NULL packet_data
RESPONSE_END
```

**Note:** The FOR loop example above is conceptual. Actual implementation depends on how the protocol structures multiple packets.

## Tips for Writing Your Own Scripts

### 1. Start Simple
Begin with the most basic packet possible - just one byte. Verify you can send and receive.

### 2. Use Packet Capture Tools
Tools like Wireshark can help you see exactly what bytes are being sent and received.

### 3. Test Incrementally
- First: Send packet, see raw response
- Second: Parse one field at a time
- Third: Add validation
- Fourth: Format output

### 4. Handle Errors Gracefully
Always include `OUTPUT_ERROR` blocks:

```pseudo
OUTPUT_ERROR
RETURN "error=<ERROR REASON>"
OUTPUT_END
```

### 5. Use Comments
Document your protocol understanding:

```pseudo
# Protocol: Custom Game Server Query
# Request: Send command byte 0x01
# Response: Receive status byte, player count (2 bytes), server name (null-term)

PACKET_START
WRITE_BYTE 0x01  # Command: Get Status
PACKET_END

RESPONSE_START
READ_BYTE status
READ_SHORT_BE player_count
READ_STRING_NULL server_name
RESPONSE_END
```

### 6. Leverage Auto-Features
Use `PACKET_LEN` for automatic length calculation instead of manual calculation.

### 7. Test with Real Servers
Always test with actual game servers to verify your script works correctly.

## Common Patterns

### Pattern: Authentication + Command

```pseudo
# Auth
PACKET_START
WRITE_STRING "auth"
WRITE_STRING password
PACKET_END

RESPONSE_START
READ_BYTE auth_result
RESPONSE_END

# Command (only if auth succeeded)
PACKET_START
WRITE_STRING "status"
PACKET_END

RESPONSE_START
READ_STRING_NULL status
RESPONSE_END
```

### Pattern: Challenge-Response

```pseudo
# Get Challenge
PACKET_START
WRITE_BYTE 0x01
PACKET_END

RESPONSE_START
READ_STRING_NULL challenge
RESPONSE_END

# Use Challenge
PACKET_START
WRITE_BYTE 0x02
WRITE_STRING challenge
PACKET_END

RESPONSE_START
READ_STRING_NULL response
RESPONSE_END
```

## Next Steps

- Review the [Syntax Reference](02-pseudo-code-syntax.md) for complete command details
- Learn [How It Works Internally](04-how-it-works.md) to understand the implementation
- Check the [Beginner's Guide](01-beginners-guide.md) if you need more basics

