# How Pseudo-Code Works Internally

This document explains how Net Sentinel processes pseudo-code scripts from text to execution. Understanding this helps you write better scripts and debug issues.

## Overview

The pseudo-code system has several stages:

```
Text Script → Parser → Abstract Syntax Tree → Executor → Results
```

Let's trace through each stage.

## Stage 1: Text Parsing

**File:** `src/packet_parser.rs` - `parse_script()`

The parser reads your text script line by line and converts it into structured data.

### How It Works

1. **Line-by-line processing**: Each line is examined to determine its type
2. **State tracking**: The parser tracks whether it's in a `PACKET_START` block, `RESPONSE_START` block, `CODE_START` block, or output block
3. **Command recognition**: Each command is matched against known patterns
4. **Structure building**: Commands are organized into packet/response pairs, code blocks, and output blocks

### Example

**Input:**
```pseudo
PACKET_START
WRITE_BYTE 0xFF
PACKET_END

RESPONSE_START
READ_BYTE status
RESPONSE_END
```

**Internal Structure Created:**
```rust
PacketScript {
    pairs: [
        PacketResponsePair {
            packets: [
                [WriteByte(0xFF)]
            ],
            response: [
                ReadByte("status")
            ]
        }
    ],
    code_blocks: [],
    output_blocks: []
}
```

### Key Functions

- `parse_script()` - Main entry point, orchestrates parsing
- `parse_packet_command()` - Parses `WRITE_*` commands
- `parse_response_command()` - Parses `READ_*` commands
- `parse_code_command()` - Parses code block commands
- `parse_control_flow()` - Parses IF/FOR statements

## Stage 2: Packet Building

**File:** `src/packet_parser.rs` - `build_packets()`

The packet builder converts command structures into actual byte arrays.

### How It Works

1. **Command execution**: Each `WRITE_*` command is executed in order
2. **Byte accumulation**: Bytes are appended to a vector
3. **Placeholder handling**: `PACKET_LEN` placeholders are noted for later
4. **Length calculation**: After all commands, length placeholders are filled in

### Example

**Commands:**
```pseudo
WRITE_INT PACKET_LEN
WRITE_INT 1
WRITE_STRING "test"
```

**Process:**
1. `WRITE_INT PACKET_LEN` → Reserve 4 bytes, note position
2. `WRITE_INT 1` → Write `[0x01, 0x00, 0x00, 0x00]` (little-endian)
3. `WRITE_STRING "test"` → Write `[0x74, 0x65, 0x73, 0x74, 0x00]` (with null)
4. Calculate length: 4 (int) + 5 (string) = 9 bytes
5. Replace placeholder: Write `[0x09, 0x00, 0x00, 0x00]` at position 0

**Final Packet:**
```
[0x09, 0x00, 0x00, 0x00,  # Length (9)
 0x01, 0x00, 0x00, 0x00,  # Value (1)
 0x74, 0x65, 0x73, 0x74, 0x00]  # "test\0"
```

### Special Handling

**VarInt Encoding:**
VarInts use a variable number of bytes. The encoder:
- Takes 7 bits from the value
- Sets the 8th bit if more bytes follow
- Continues until all bits are encoded

Example: `300` → `[0xAC, 0x02]`
- `0xAC` = `10101100` (bits 0-6 = 44, bit 7 = 1 means more)
- `0x02` = `00000010` (bits 7-13 = 2)
- Value = 44 + (2 << 7) = 44 + 256 = 300

**Length Calculation:**
When `PACKET_LEN` is used:
1. Placeholder position is recorded
2. Packet is built completely
3. Length is calculated (everything after the length field)
4. Length is encoded and inserted at placeholder position

## Stage 3: Network Communication

**File:** `src/gameserver_check.rs` - `check_game_server()`

The network layer sends packets and receives responses.

### How It Works

1. **Connection establishment**: TCP connection or UDP socket is created
2. **Packet sending**: Built packets are sent over the network
3. **Response waiting**: System waits for server response (with timeout)
4. **Response receiving**: Raw bytes are received from the server

### TCP vs UDP

**TCP (Transmission Control Protocol):**
- Connection-oriented (connection stays open)
- Reliable (guaranteed delivery)
- Used for: RCON, custom protocols requiring connection state

**UDP (User Datagram Protocol):**
- Connectionless (each packet is independent)
- Unreliable (no guarantee of delivery)
- Used for: Game server queries, status checks

### Connection Management

For multiple packet/response pairs:
- **TCP**: Connection is kept alive across all pairs
- **UDP**: Socket is reused for all pairs
- **CONNECTION_CLOSE**: Forces connection close before next pair

### Timeout Handling

Each operation has a timeout:
- Connection timeout
- Send timeout
- Receive timeout

If timeout occurs, an error is returned.

## Stage 4: Response Parsing

**File:** `src/packet_parser.rs` - `parse_response()`

The response parser extracts data from raw bytes using your parsing commands.

### How It Works

1. **Cursor tracking**: A cursor tracks position in the response buffer
2. **Command execution**: Each `READ_*` command reads bytes at the cursor
3. **Variable storage**: Read values are stored in a variable map
4. **Validation**: `EXPECT_*` commands validate bytes match expected values

### Example

**Response Bytes:**
```
[0xFE, 0xFD, 0x09, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x74, 0x6F, 0x6B, 0x65, 0x6E, 0x00]
```

**Commands:**
```pseudo
EXPECT_BYTE 0xFE
EXPECT_BYTE 0xFD
READ_BYTE packet_type
READ_STRING_NULL session_id
READ_STRING_NULL challenge_token
```

**Process:**
1. Cursor = 0, `EXPECT_BYTE 0xFE` → Check `[0] == 0xFE` ✓, cursor = 1
2. Cursor = 1, `EXPECT_BYTE 0xFD` → Check `[1] == 0xFD` ✓, cursor = 2
3. Cursor = 2, `READ_BYTE packet_type` → Read `[2] = 0x09`, store `packet_type = 9`, cursor = 3
4. Cursor = 3, `READ_STRING_NULL session_id` → Read until `0x00` at position 10, store `session_id = "session"`, cursor = 11
5. Cursor = 11, `READ_STRING_NULL challenge_token` → Read until `0x00` at position 16, store `challenge_token = "token"`, cursor = 17

**Variables Created:**
```rust
{
    "packet_type": 9,
    "session_id": "session",
    "challenge_token": "token"
}
```

### Error Handling

If parsing fails:
- **Insufficient data**: Not enough bytes in response
- **Validation failure**: `EXPECT_*` command failed
- **Parse error**: Invalid data format

Errors stop parsing and return an error result.

## Stage 5: Code Block Execution

**File:** `src/packet_parser.rs` - `execute_code_blocks()`

Code blocks allow variable manipulation and control flow.

### How It Works

1. **Variable evaluation**: Expressions are evaluated to get values
2. **Command execution**: Each command in the code block is executed
3. **Variable storage**: New/modified variables are stored
4. **Control flow**: IF/FOR statements control execution flow

### Variable Scoping

Variables exist in two scopes:
1. **Parsed variables**: From `READ_*` commands
2. **Code variables**: From code blocks

Code variables can override parsed variables if they have the same name.

### Expression Evaluation

Expressions can be:
- **Literals**: `"string"`, `123`, `0xFF`
- **Variables**: `var_name`
- **Array indices**: `array_name[0]`
- **Function calls**: `SPLIT(var, ',')`

### Example

**Code Block:**
```pseudo
CODE_START
STRING data = "a,b,c"
SPLIT(data, ',')
IF data[0] == "a":
  STRING result = "first is a"
CODE_END
```

**Execution:**
1. `STRING data = "a,b,c"` → Store `data = "a,b,c"`
2. `SPLIT(data, ',')` → Store `data = ["a", "b", "c"]`
3. `IF data[0] == "a"` → Evaluate `"a" == "a"` → true
4. `STRING result = "first is a"` → Store `result = "first is a"`

## Stage 6: Output Formatting

**File:** `src/gameserver_check.rs` - `evaluate_output_labels()`

Output blocks format results for Prometheus metrics.

### How It Works

1. **Block selection**: Success or error block is selected based on result
2. **Command execution**: `JSON_OUTPUT` and `RETURN` commands are executed
3. **Variable substitution**: Variables in `RETURN` expressions are replaced
4. **Label generation**: Final label strings are created

### Variable Resolution

Variables can be referenced as:
- **Simple**: `var_name`
- **Nested (JSON)**: `JSON_PAYLOAD.version.protocol`
- **Placeholders**: `HOST`, `PORT`, `IP`

### Example

**Output Block:**
```pseudo
OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "protocol=JSON_PAYLOAD.version.protocol, players=player_count"
OUTPUT_END
```

**Variables:**
```json
{
  "JSON_PAYLOAD": "{\"version\":{\"protocol\":773},\"players\":{\"online\":10}}",
  "player_count": 10
}
```

**Process:**
1. `JSON_OUTPUT JSON_PAYLOAD` → Parse JSON string into object
2. `RETURN "protocol=..."` → Resolve variables:
   - `JSON_PAYLOAD.version.protocol` → `773`
   - `player_count` → `10`
3. Result: `"protocol=773, players=10"`

### Placeholder Resolution

Special placeholders are resolved:
- `HOST` → Server address
- `PORT` → Server port
- `IP` → Server IP
- `<ERROR REASON>` → Error message (in error blocks)

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│ 1. Text Script                                          │
│    "PACKET_START\nWRITE_BYTE 0xFF\n..."                │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 2. Parser (parse_script)                                │
│    - Tokenizes lines                                    │
│    - Builds PacketScript structure                      │
│    - Creates PacketResponsePair objects                 │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Packet Builder (build_packets)                      │
│    - Executes WRITE_* commands                          │
│    - Builds byte arrays                                 │
│    - Calculates and fills PACKET_LEN                    │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Network Layer (check_game_server)                    │
│    - Establishes TCP/UDP connection                     │
│    - Sends packets                                      │
│    - Receives responses                                 │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 5. Response Parser (parse_response)                      │
│    - Executes READ_* commands                           │
│    - Extracts variables                                 │
│    - Validates with EXPECT_*                            │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 6. Code Execution (execute_code_blocks)                 │
│    - Executes code block commands                       │
│    - Evaluates expressions                              │
│    - Modifies variables                                 │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 7. Output Formatting (evaluate_output_labels)          │
│    - Executes OUTPUT_* blocks                           │
│    - Formats RETURN expressions                         │
│    - Generates Prometheus labels                        │
└──────────────────┬────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ 8. Final Result                                         │
│    - Success/error status                               │
│    - Parsed values                                      │
│    - Formatted output labels                            │
└─────────────────────────────────────────────────────────┘
```

## Key Data Structures

### PacketScript
```rust
struct PacketScript {
    pairs: Vec<PacketResponsePair>,      // Packet/response pairs
    output_blocks: Vec<OutputBlock>,      // Output formatting
    code_blocks: Vec<CodeBlock>,          // Code execution
}
```

### PacketResponsePair
```rust
struct PacketResponsePair {
    packets: Vec<Vec<PacketCommand>>,    // Multiple packets
    response: Vec<ResponseCommand>,       // Response parsing
    close_connection_before: bool,         // Close connection flag
}
```

### Variables
Variables are stored in `IndexMap<String, JsonValue>`:
- Keys: Variable names
- Values: JSON values (string, number, object, array, etc.)

## Error Handling

Errors can occur at any stage:

1. **Parse errors**: Invalid syntax → `SyntaxError`
2. **Build errors**: Invalid packet construction → `SyntaxError`
3. **Network errors**: Connection/timeout issues → `NetworkError`
4. **Parse errors**: Invalid response format → `ParseError`
5. **Validation errors**: EXPECT_* failures → `ValidationError`

All errors include:
- Error type
- Error message
- Line number (for syntax errors)

## Performance Considerations

1. **Packet building**: Happens once per script execution
2. **Network I/O**: Usually the slowest part (network latency)
3. **Parsing**: Fast (just reading bytes)
4. **Code execution**: Fast (simple operations)

## Debugging Tips

1. **Enable logging**: The code has extensive `println!` statements for debugging
2. **Check raw response**: Look at `raw_response` hex dump
3. **Verify packet bytes**: Check what bytes are actually sent
4. **Test incrementally**: Add one command at a time
5. **Use packet capture**: Tools like Wireshark show actual network traffic

## Next Steps

- Review [Examples](03-examples.md) to see these concepts in action
- Check [Syntax Reference](02-pseudo-code-syntax.md) for command details
- Read [Beginner's Guide](01-beginners-guide.md) for basics

