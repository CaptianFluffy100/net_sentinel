# Pseudo-Code Syntax Reference

Complete reference for all pseudo-code commands. Use this as a quick reference when writing scripts.

## Table of Contents

1. [Packet Construction](#packet-construction)
2. [HTTP Request Construction](#http-request-construction)
3. [Response Parsing](#response-parsing)
4. [HTTP Response Parsing](#http-response-parsing)
5. [Validation Commands](#validation-commands)
6. [Output Formatting](#output-formatting)
7. [Code Blocks](#code-blocks)
8. [Special Features](#special-features)

## Packet Construction

### `PACKET_START` / `PACKET_END`

Marks the beginning and end of a packet definition.

```pseudo
PACKET_START
  ... commands ...
PACKET_END
```

### `WRITE_BYTE <value>`

Writes a single byte (0-255).

**Formats:**
- Decimal: `WRITE_BYTE 255`
- Hexadecimal: `WRITE_BYTE 0xFF`

**Example:**
```pseudo
WRITE_BYTE 0x01
WRITE_BYTE 255
```

### `WRITE_SHORT <value>` / `WRITE_SHORT_BE <value>`

Writes a 16-bit integer (0-65,535).

- `WRITE_SHORT` - Little-endian (default)
- `WRITE_SHORT_BE` - Big-endian (network byte order)

**Example:**
```pseudo
WRITE_SHORT 1234
WRITE_SHORT_BE 1234
```

### `WRITE_INT <value>` / `WRITE_INT_BE <value>`

Writes a 32-bit integer (0-4,294,967,295).

- `WRITE_INT` - Little-endian (default)
- `WRITE_INT_BE` - Big-endian (network byte order)
- Special: Use `PACKET_LEN` to auto-calculate packet length

**Example:**
```pseudo
WRITE_INT 50000
WRITE_INT_BE PACKET_LEN
```

### `WRITE_STRING <text>`

Writes a null-terminated string. Automatically adds `0x00` at the end.

**Example:**
```pseudo
WRITE_STRING "Hello Server"
WRITE_STRING "status"
```

**Note:** Strings with spaces must be quoted.

### `WRITE_STRING_LEN <text> <length>`

Writes a fixed-length string. Pads with null bytes if shorter, truncates if longer.

**Example:**
```pseudo
WRITE_STRING_LEN "Test" 10    # "Test" + 6 null bytes
WRITE_STRING_LEN "Hello" 3    # "Hel" (truncated)
```

### `WRITE_VARINT <value>`

Writes a variable-length integer (used in Minecraft-style protocols).

- Special: Use `PACKET_LEN` to auto-calculate as VarInt

**Example:**
```pseudo
WRITE_VARINT 300
WRITE_VARINT PACKET_LEN
```

**How VarInt works:**
- Values 0-127 use 1 byte
- Larger values use multiple bytes
- Each byte's most significant bit indicates more bytes follow

### `WRITE_BYTES <hex_string>`

Writes raw hexadecimal bytes.

**Example:**
```pseudo
WRITE_BYTES "FF00AA55"
WRITE_BYTES "FF 00 AA 55"
WRITE_BYTES "0xFF00AA55"
```

## Response Parsing

### `RESPONSE_START` / `RESPONSE_END`

Marks the beginning and end of response parsing rules.

```pseudo
RESPONSE_START
  ... commands ...
RESPONSE_END
```

### `READ_BYTE <var_name>`

Reads a single byte and stores it in a variable.

**Example:**
```pseudo
READ_BYTE packet_id
READ_BYTE status_code
```

### `READ_SHORT <var_name>` / `READ_SHORT_BE <var_name>`

Reads a 16-bit integer and stores it in a variable.

- `READ_SHORT` - Little-endian
- `READ_SHORT_BE` - Big-endian

**Example:**
```pseudo
READ_SHORT player_count
READ_SHORT_BE port_number
```

### `READ_INT <var_name>` / `READ_INT_BE <var_name>`

Reads a 32-bit integer and stores it in a variable.

- `READ_INT` - Little-endian
- `READ_INT_BE` - Big-endian

**Example:**
```pseudo
READ_INT server_version
READ_INT_BE response_length
```

### `READ_VARINT <var_name>`

Reads a variable-length integer (VarInt).

**Example:**
```pseudo
READ_VARINT packet_length
READ_VARINT packet_id
```

### `READ_STRING <var_name> <length>`

Reads a fixed-length string.

**Example:**
```pseudo
READ_STRING server_name 32
READ_STRING map_name 64
```

**Note:** Trailing null bytes are automatically trimmed.

### `READ_STRING_NULL <var_name>`

Reads a null-terminated string. Stops when `0x00` is encountered.

**Example:**
```pseudo
READ_STRING_NULL server_name
READ_STRING_NULL command_output
```

### `SKIP_BYTES <count>`

Skips (advances past) the specified number of bytes without reading them.

**Example:**
```pseudo
SKIP_BYTES 4
SKIP_BYTES 2
```

**Use case:** Skip padding, reserved fields, or data you don't need.

## HTTP Request Construction

HTTP/HTTPS requests are constructed using blocks similar to `PACKET_START`/`PACKET_END`, but with HTTP-specific commands. These commands are used when the protocol is set to `HTTP` or `HTTPS`.

### `HTTP_START REQUEST <METHOD> <PATH>`

Marks the beginning of an HTTP request and specifies the HTTP method and path.

**HTTP Methods Supported:**
- `GET` - Retrieve data
- `POST` - Submit data
- `PUT` - Update/replace data
- `DELETE` - Delete data
- `Custom` - Custom HTTP method (specify the method name)

**Path:**
- Must start with `/` (e.g., `/api/users`, `/status`, `/health`)
- Query parameters are specified separately using `PARAM` commands

**Example:**
```pseudo
HTTP_START REQUEST GET /api/status
HTTP_START REQUEST POST /api/users
HTTP_START REQUEST PUT /api/users/123
HTTP_START REQUEST DELETE /api/users/123
HTTP_START REQUEST Custom PATCH /api/users/123
```

### `PARAM <Key> <Value>`

Adds a query parameter to the request URL. Multiple `PARAM` commands can be used to add multiple query parameters.

**Behavior:**
- Query parameters are URL-encoded automatically
- Parameters are appended to the URL path in the order they appear
- First parameter uses `?`, subsequent parameters use `&`

**Example:**
```pseudo
HTTP_START REQUEST GET /api/search
PARAM q hello
PARAM limit 10
PARAM page 1
```
This generates: `/api/search?q=hello&limit=10&page=1`

### `HEADER <Key> <Value>`

Adds an HTTP header to the request. Multiple `HEADER` commands can be used to add multiple headers.

**Common Headers:**
- `Content-Type` - Specifies the format of the request body
- `Authorization` - Authentication credentials
- `User-Agent` - Client identifier
- `Accept` - Expected response format
- Custom headers as needed

**Example:**
```pseudo
HEADER Content-Type application/json
HEADER Authorization Bearer token123
HEADER User-Agent NetSentinel/1.0
HEADER Accept application/json
```

**Note:** `Content-Type` is automatically set based on `BODY_START TYPE`, but can be overridden explicitly.

### `BODY_START TYPE <TYPE>`

Marks the beginning of the request body section and specifies the body format type.

**Body Types:**
- `FORM` - URL-encoded form data (application/x-www-form-urlencoded)
- `RAW` - Raw body content (typically used with JSON, XML, or plain text)

**Example:**
```pseudo
BODY_START TYPE FORM
BODY_START TYPE RAW
```

### `DATA <Content>`

Specifies the body content for the request. The content is processed based on the body type specified in `BODY_START TYPE`.

**For `FORM` type:**
- Multiple `DATA` commands or a single `DATA` command with `key=value` pairs
- Content is automatically URL-encoded
- Format: `DATA key1=value1&key2=value2` or separate `DATA` commands per field

**For `RAW` type:**
- Raw content as-is (typically JSON, XML, or plain text)
- JSON objects can be written directly and will be automatically stringified
- Users don't need to manually convert JSON to strings

**Examples:**

Form data:
```pseudo
BODY_START TYPE FORM
DATA username=admin
DATA password=secret123
DATA remember=true
BODY_END
```

JSON data (RAW):
```pseudo
BODY_START TYPE RAW
DATA {"name": "John", "age": 30, "active": true}
BODY_END
```

JSON with nested objects (automatically stringified):
```pseudo
BODY_START TYPE RAW
DATA {
  "user": {
    "name": "John",
    "email": "john@example.com"
  },
  "preferences": {
    "theme": "dark"
  }
}
BODY_END
```

### `BODY_END`

Marks the end of the request body section. Must appear after all `DATA` commands.

### `HTTP_END`

Marks the end of the HTTP request block. Must appear after all request configuration commands.

## HTTP Response Parsing

HTTP responses use the existing `RESPONSE_START`/`RESPONSE_END` blocks, but with HTTP-specific parsing commands.

### `EXPECT_STATUS <Code>`

Validates that the HTTP response status code matches the expected value.

**Status Codes:**
- Common success codes: `200`, `201`, `204`
- Common error codes: `400`, `401`, `403`, `404`, `500`, `503`
- Any valid HTTP status code (100-599)

**Example:**
```pseudo
RESPONSE_START
EXPECT_STATUS 200
RESPONSE_END
```

**Behavior:**
- If the status code doesn't match, the test fails
- Status code is available as a variable `STATUS_CODE` for use in output blocks

### `EXPECT_HEADER <Key> <Value>`

Validates that the HTTP response contains a header with the specified key and value.

**Case Sensitivity:**
- Header names are case-insensitive (as per HTTP specification)
- Header values are compared as-is (may be case-sensitive depending on header)

**Example:**
```pseudo
RESPONSE_START
EXPECT_STATUS 200
EXPECT_HEADER Content-Type application/json
EXPECT_HEADER X-API-Version 2.0
RESPONSE_END
```

**Behavior:**
- If the header is missing or the value doesn't match, the test fails
- Header values are available as variables named `HEADER_<Key>` (e.g., `HEADER_Content-Type`)

### `READ_BODY_JSON <VariableName>`

Reads and parses the entire HTTP response body as JSON, storing the result in a variable.

**Behavior:**
- Parses the response body as JSON
- Stores the parsed JSON object/array in the specified variable
- The variable can then be accessed in output blocks using dot notation (e.g., `status.data.users[0].name`)
- If parsing fails, the test fails with an error

**Example:**
```pseudo
RESPONSE_START
EXPECT_STATUS 200
EXPECT_HEADER Content-Type application/json
READ_BODY_JSON response
RESPONSE_END

OUTPUT_SUCCESS
RETURN "users=response.data.users.length, status=response.status"
OUTPUT_END
```

**JSON Path Access:**
After reading with `READ_BODY_JSON`, you can access nested fields using dot notation:
- `response.status` - Top-level field
- `response.data.users` - Nested object field
- `response.data.users[0].name` - Array element access

### `READ_BODY <VariableName>`

Reads the entire HTTP response body as raw text and stores it in a variable. This is useful for non-JSON responses such as plain text, HTML, XML, or other text-based formats.

**Behavior:**
- Reads the response body as UTF-8 text
- Stores the text string in the specified variable
- The variable can then be accessed in output blocks as a regular string
- If the body is not valid UTF-8, the test fails with an error

**Example:**
```pseudo
RESPONSE_START
EXPECT_STATUS 200
EXPECT_HEADER Content-Type text/plain
READ_BODY responseText
RESPONSE_END

OUTPUT_SUCCESS
RETURN "response=responseText"
OUTPUT_END
```

**When to use `READ_BODY` vs `READ_BODY_JSON`:**
- Use `READ_BODY_JSON` when the response is JSON and you need to access nested fields
- Use `READ_BODY` when the response is plain text, HTML, XML, or any other non-JSON format

## Validation Commands

### `EXPECT_BYTE <value>`

Validates that the next byte matches the expected value. Raises an error if it doesn't match.

**Example:**
```pseudo
EXPECT_BYTE 0xFE
EXPECT_BYTE 0xFF
EXPECT_BYTE 255
```

**Use case:** Validate magic bytes, protocol headers, or status codes.

### `EXPECT_MAGIC <hex_string>`

Validates that the next bytes match the expected magic bytes sequence.

**Example:**
```pseudo
EXPECT_MAGIC "FEEDFACE"
EXPECT_MAGIC "FF FF FF FF"
```

**Use case:** Validate protocol headers or magic byte sequences.

## Output Formatting

### `OUTPUT_SUCCESS` / `OUTPUT_ERROR` / `OUTPUT_END`

Marks output blocks that execute on success or error.

```pseudo
OUTPUT_SUCCESS
  ... commands ...
OUTPUT_END

OUTPUT_ERROR
  ... commands ...
OUTPUT_END
```

### `JSON_OUTPUT <var_name>`

Parses a string variable as JSON, making nested fields accessible.

**Example:**
```pseudo
RESPONSE_START
READ_STRING_NULL JSON_PAYLOAD
RESPONSE_END

OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "protocol=JSON_PAYLOAD.version.protocol"
OUTPUT_END
```

**Use case:** Many game servers return JSON. This makes nested fields like `JSON_PAYLOAD.version.protocol` accessible.

### `RETURN "<expression>"`

Formats the expression into Prometheus metric labels.

**Available in expressions:**
- Variable references: `JSON_PAYLOAD.version.protocol`
- Placeholders: `HOST`, `PORT`, `IP`
- Error placeholder: `<ERROR REASON>` (only in `OUTPUT_ERROR`)

**Example:**
```pseudo
OUTPUT_SUCCESS
RETURN "server='HOST', protocol=JSON_PAYLOAD.version.protocol, players=player_count"
OUTPUT_END

OUTPUT_ERROR
RETURN "server='HOST', error=<ERROR REASON>"
OUTPUT_END
```

**Note:** The format is `key=value, key=value` (comma-separated key-value pairs).

## Code Blocks

Code blocks allow variable declarations, control flow, and data manipulation.

### `CODE_START` / `CODE_END`

Marks the beginning and end of a code block.

```pseudo
CODE_START
  ... commands ...
CODE_END
```

### Variable Declarations

Declare variables with explicit types:

```pseudo
STRING <name> = <value>
INT <name> = <value>
BYTE <name> = <value>
FLOAT <name> = <value>
ARRAY <name> = <value>
```

**Example:**
```pseudo
CODE_START
STRING password = "my_password"
INT request_id = 1
BYTE status = 0xFF
FLOAT version = 1.19
ARRAY parts = ["a", "b", "c"]
CODE_END
```

### Variable Assignment

Reassign variables:

```pseudo
<name> = <value>
```

**Example:**
```pseudo
count = 20
message = "Updated"
```

### FOR Loops

Iterate over arrays:

```pseudo
FOR <var_name> IN <array_name>:
  ... commands ...
```

**Example:**
```pseudo
CODE_START
ARRAY items = ["a", "b", "c"]
FOR item IN items:
  WRITE_STRING item
CODE_END
```

### IF Statements

Conditional execution:

```pseudo
IF <condition>:
  ... commands ...
ELSE IF <condition>:
  ... commands ...
ELSE:
  ... commands ...
```

**Comparison Operators:**
- `==` - Equals
- `!=` - Not equals
- `>` - Greater than
- `<` - Less than
- `>=` - Greater than or equal
- `<=` - Less than or equal
- `CONTAINS` - String contains substring

**Example:**
```pseudo
CODE_START
IF response_id == 1:
  STRING status = "Success"
ELSE:
  STRING status = "Error"
CODE_END
```

### String Functions

#### `SPLIT(<var_name>, '<delimiter>')`

Splits a string by delimiter and stores as array:

```pseudo
STRING data = "a,b,c"
SPLIT(data, ',')
# data is now ["a", "b", "c"]
```

#### `REPLACE(<var_name>, '<search>', '<replace>')`

Replaces all occurrences in a string:

```pseudo
STRING message = "Hello World"
REPLACE(message, 'World', 'Server')
# message is now "Hello Server"
```

## Special Features

### Automatic Length Calculation (`PACKET_LEN`)

Use `PACKET_LEN` as a placeholder to auto-calculate packet length:

```pseudo
PACKET_START
WRITE_INT PACKET_LEN      # Auto-calculated
WRITE_INT 1
WRITE_STRING "command"
PACKET_END
```

**Supported commands:**
- `WRITE_INT PACKET_LEN` - 4-byte little-endian
- `WRITE_INT_BE PACKET_LEN` - 4-byte big-endian
- `WRITE_VARINT PACKET_LEN` - VarInt format

**Note:** Length excludes the length field itself.

### Placeholder Variables

Automatically resolved by the server:

- `HOST` - Server hostname/address
- `PORT` - Server port number
- `IP` - Server IP address
- `IP_LEN` - Length of IP address string
- `IP_LEN_HEX` - Length of IP address in hexadecimal

**Example:**
```pseudo
PACKET_START
WRITE_BYTE IP_LEN
WRITE_STRING IP
WRITE_SHORT_BE PORT
PACKET_END
```

### Multiple Packet/Response Pairs

Execute multiple pairs sequentially:

```pseudo
# Pair 1: Authentication
PACKET_START
WRITE_STRING "auth"
PACKET_END

RESPONSE_START
READ_BYTE auth_result
RESPONSE_END

# Pair 2: Command
PACKET_START
WRITE_STRING "status"
PACKET_END

RESPONSE_START
READ_STRING_NULL status_info
RESPONSE_END
```

**Execution order:**
1. Send packet 1 → Receive response 1 → Parse response 1
2. Send packet 2 → Receive response 2 → Parse response 2
3. Continue for all pairs...

### Connection Management

Use `CONNECTION_CLOSE` to close the connection before the next packet:

```pseudo
# First request
PACKET_START
WRITE_STRING "hello"
PACKET_END

RESPONSE_START
READ_STRING_NULL response
RESPONSE_END

CONNECTION_CLOSE

# Second request (new connection)
PACKET_START
WRITE_STRING "goodbye"
PACKET_END
```

## Comments

Lines starting with `#` are comments:

```pseudo
# This is a comment
PACKET_START
WRITE_BYTE 0xFF  # Inline comment
PACKET_END
```

## Value Formats

### Decimal Numbers
```pseudo
WRITE_BYTE 255
WRITE_SHORT 1234
WRITE_INT 4294967295
```

### Hexadecimal Numbers
```pseudo
WRITE_BYTE 0xFF
WRITE_SHORT 0x1234
WRITE_INT 0xFFFFFFFF
```

### Strings
```pseudo
WRITE_STRING "Hello World"        # Quoted (required for spaces)
WRITE_STRING test                 # Unquoted (simple strings)
WRITE_STRING_LEN "Test" 10       # Fixed length
```

### Hex Bytes
```pseudo
WRITE_BYTES "FF00AA55"           # Without spaces
WRITE_BYTES "FF 00 AA 55"        # With spaces
WRITE_BYTES "0xFF00AA55"         # With 0x prefix
```

## Quick Reference Table

### TCP/UDP Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `WRITE_BYTE` | Write 1 byte | `WRITE_BYTE 0xFF` |
| `WRITE_SHORT` | Write 2 bytes (LE) | `WRITE_SHORT 1234` |
| `WRITE_SHORT_BE` | Write 2 bytes (BE) | `WRITE_SHORT_BE 1234` |
| `WRITE_INT` | Write 4 bytes (LE) | `WRITE_INT 50000` |
| `WRITE_INT_BE` | Write 4 bytes (BE) | `WRITE_INT_BE 50000` |
| `WRITE_VARINT` | Write VarInt | `WRITE_VARINT 300` |
| `WRITE_STRING` | Write text (null-term) | `WRITE_STRING "hello"` |
| `WRITE_STRING_LEN` | Write fixed-length text | `WRITE_STRING_LEN "test" 10` |
| `WRITE_BYTES` | Write hex bytes | `WRITE_BYTES "FF00"` |
| `READ_BYTE` | Read 1 byte | `READ_BYTE status` |
| `READ_SHORT` | Read 2 bytes (LE) | `READ_SHORT count` |
| `READ_SHORT_BE` | Read 2 bytes (BE) | `READ_SHORT_BE port` |
| `READ_INT` | Read 4 bytes (LE) | `READ_INT version` |
| `READ_INT_BE` | Read 4 bytes (BE) | `READ_INT_BE length` |
| `READ_VARINT` | Read VarInt | `READ_VARINT length` |
| `READ_STRING` | Read fixed-length string | `READ_STRING name 32` |
| `READ_STRING_NULL` | Read null-term string | `READ_STRING_NULL name` |
| `SKIP_BYTES` | Skip bytes | `SKIP_BYTES 4` |
| `EXPECT_BYTE` | Validate byte | `EXPECT_BYTE 0xFE` |
| `EXPECT_MAGIC` | Validate magic bytes | `EXPECT_MAGIC "FEED"` |

### HTTP/HTTPS Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `HTTP_START REQUEST` | Begin HTTP request | `HTTP_START REQUEST GET /api/status` |
| `PARAM` | Add query parameter | `PARAM page 1` |
| `HEADER` | Add HTTP header | `HEADER Authorization Bearer token123` |
| `BODY_START TYPE` | Begin request body | `BODY_START TYPE RAW` |
| `DATA` | Add body content | `DATA {"key": "value"}` |
| `BODY_END` | End request body | `BODY_END` |
| `HTTP_END` | End HTTP request | `HTTP_END` |
| `EXPECT_STATUS` | Validate status code | `EXPECT_STATUS 200` |
| `EXPECT_HEADER` | Validate response header | `EXPECT_HEADER Content-Type application/json` |
| `READ_BODY_JSON` | Parse JSON response | `READ_BODY_JSON response` |
| `READ_BODY` | Read raw text response | `READ_BODY responseText` |

## Next Steps

- [Examples and Tutorials](03-examples.md) - See real-world usage
- [How It Works Internally](04-how-it-works.md) - Understand the implementation

