# Minecraft RCON Protocol - Implementation Guide

## Overview

RCON (Remote Console) is a protocol used to remotely execute commands on Minecraft servers. It uses TCP connections and a simple packet-based protocol. This document explains how to connect to and interact with Minecraft RCON using the pseudo-code format defined in `gameserver_check.md`.

## Protocol Details

- **Protocol**: TCP (not UDP)
- **Default Port**: 25575 (configurable in `server.properties`)
- **Endianness**: All integers are little-endian
- **Security**: RCON is **not encrypted** - use SSH tunneling or VPN for secure access

## RCON Packet Structure

Each RCON packet has the following structure:

```
[Length: 4 bytes (LE)] [Request ID: 4 bytes (LE)] [Type: 4 bytes (LE)] [Payload: null-terminated string] [0x00] [0x00]
```

### Packet Fields

1. **Length** (4 bytes, little-endian): Total length of the packet (excluding the length field itself)
   - Calculated as: `4 (Request ID) + 4 (Type) + len(payload) + 2 (null terminators)`

2. **Request ID** (4 bytes, little-endian): Unique identifier for the request/response pair
   - Use a unique ID for each request
   - Server echoes this ID in the response

3. **Type** (4 bytes, little-endian): Packet type identifier
   - `3` = `SERVERDATA_AUTH` (authentication request)
   - `2` = `SERVERDATA_EXECCOMMAND` (command execution request)
   - `0` = `SERVERDATA_RESPONSE_VALUE` (response from server)

4. **Payload** (null-terminated string): The actual data
   - For authentication: the RCON password
   - For commands: the command string to execute
   - For responses: the command output or error message

5. **Two null bytes** (`0x00 0x00`): Terminates the packet

## Server Configuration

Before connecting, ensure the Minecraft server has RCON enabled in `server.properties`:

```properties
enable-rcon=true
rcon.password=your_secure_password_here
rcon.port=25575
```

## Pseudo-Code Implementation

### Step 1: Authentication Packet

To authenticate with the RCON server, send an authentication packet. **Important**: RCON requires a length field at the beginning of each packet. Since the current pseudo-code system doesn't automatically calculate fixed-length fields, you'll need to manually calculate and write the length.

**Length Calculation**: 
- Length = 4 (Request ID) + 4 (Type) + len(password) + 2 (null terminators)
- Example: For password "mypass" (6 bytes), length = 4 + 4 + 6 + 2 = 16

```pseudo
# RCON Authentication Packet
# Protocol: TCP
# Packet Type: SERVERDATA_AUTH (3)
# Password: "mypass" (6 bytes)
# Length = 4 + 4 + 6 + 2 = 16

PACKET_START
# Length field (4 bytes, little-endian)
# Must be calculated manually: 4 (Request ID) + 4 (Type) + len(password) + 2 (nulls)
WRITE_INT 16

# Request ID (unique identifier, e.g., 1)
WRITE_INT 1

# Packet Type: SERVERDATA_AUTH = 3
WRITE_INT 3

# Payload: RCON password (null-terminated string)
WRITE_STRING "mypass"

# Two null bytes to terminate packet
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
# Read response packet
# Length field (4 bytes) - we need to read this first to know packet size
READ_INT response_length

# Request ID should match what we sent (1)
READ_INT response_id

# Packet Type: Should be 0 (SERVERDATA_RESPONSE_VALUE) for auth response
READ_INT response_type

# If response_type is 0 and response_id matches, authentication succeeded
# If response_id is -1, authentication failed

# Read payload (null-terminated string)
READ_STRING_NULL auth_result

# Skip the two null bytes
SKIP_BYTES 2

# Validate authentication
# If response_id == 1 and response_type == 0, auth succeeded
# If response_id == -1, auth failed
RESPONSE_END
```

**Note**: The actual implementation needs to calculate the length field correctly. The length is the size of the packet minus the 4-byte length field itself.

### Step 2: Command Execution Packet

After successful authentication, send commands using the execution packet. Again, the length field must be calculated manually.

**Length Calculation**:
- Length = 4 (Request ID) + 4 (Type) + len(command) + 2 (null terminators)
- Example: For command "list" (4 bytes), length = 4 + 4 + 4 + 2 = 14

```pseudo
# RCON Command Execution Packet
# Protocol: TCP
# Packet Type: SERVERDATA_EXECCOMMAND (2)
# Command: "list" (4 bytes)
# Length = 4 + 4 + 4 + 2 = 14

PACKET_START
# Length field (4 bytes, little-endian)
WRITE_INT 14

# Request ID (increment for each command, e.g., 2)
WRITE_INT 2

# Packet Type: SERVERDATA_EXECCOMMAND = 2
WRITE_INT 2

# Payload: Command to execute (null-terminated string)
WRITE_STRING "list"

# Two null bytes to terminate packet
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
# Read response packet
READ_INT response_length
READ_INT response_id
READ_INT response_type

# Response type should be 0 (SERVERDATA_RESPONSE_VALUE)
# Read the command output
READ_STRING_NULL command_output

# Skip the two null bytes
SKIP_BYTES 2
RESPONSE_END
```

## Complete Example: Authentication + Command

Here's a complete example that authenticates and executes a command:

```pseudo
# Complete RCON Example: Authenticate and run "list" command
# Protocol: TCP
# Port: 25575 (default)

# ============================================
# PACKET 1: Authentication
# Password: "my_rcon_password" (17 bytes)
# Length = 4 + 4 + 17 + 2 = 27
# ============================================
PACKET_START
# Length field (27 bytes)
WRITE_INT 27

# Request ID: 1
WRITE_INT 1

# Type: SERVERDATA_AUTH (3)
WRITE_INT 3

# Password
WRITE_STRING "my_rcon_password"

# Terminators
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
READ_INT auth_response_length
READ_INT auth_response_id
READ_INT auth_response_type
READ_STRING_NULL auth_result
SKIP_BYTES 2

# Validate: auth_response_id should be 1, auth_response_type should be 0
# If auth_response_id is -1, authentication failed
RESPONSE_END

# ============================================
# PACKET 2: Execute Command "list"
# Command: "list" (4 bytes)
# Length = 4 + 4 + 4 + 2 = 14
# ============================================
PACKET_START
# Length field (14 bytes)
WRITE_INT 14

# Request ID: 2
WRITE_INT 2

# Type: SERVERDATA_EXECCOMMAND (2)
WRITE_INT 2

# Command to execute
WRITE_STRING "list"

# Terminators
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END

RESPONSE_START
READ_INT cmd_response_length
READ_INT cmd_response_id
READ_INT cmd_response_type
READ_STRING_NULL command_output
SKIP_BYTES 2

# command_output contains the result of "list" command
RESPONSE_END
```

## Quick Reference: Common Length Calculations

To make it easier, here's a quick reference for common password and command lengths:

| Password/Command | String Length | Calculated Length |
|----------------|---------------|-------------------|
| "test" | 4 | 14 (4+4+4+2) |
| "password" | 8 | 18 (4+4+8+2) |
| "mypassword123" | 13 | 23 (4+4+13+2) |
| "list" | 4 | 14 (4+4+4+2) |
| "say Hello" | 9 | 19 (4+4+9+2) |
| "whitelist add Player" | 19 | 29 (4+4+19+2) |

**Formula**: `Length = 4 + 4 + string_length + 2`

## Important Implementation Notes

### Length Field Calculation

The length field in RCON packets is critical. It represents the size of the packet **excluding the 4-byte length field itself**:

```
Length = 4 (Request ID) + 4 (Type) + len(payload) + 2 (null terminators)
```

**Important**: The current pseudo-code system does **not** automatically calculate fixed-length fields. You **must manually calculate** the length and write it as the first field in each RCON packet.

**Formula for calculating length**:
1. Count the bytes in your password/command string
2. Add 4 (Request ID) + 4 (Type) + string_length + 2 (null terminators)
3. Write this value as a 4-byte little-endian integer at the start of the packet

**Example calculations**:
- Password "test" (4 bytes): Length = 4 + 4 + 4 + 2 = 14
- Password "mypassword123" (13 bytes): Length = 4 + 4 + 13 + 2 = 23
- Command "list" (4 bytes): Length = 4 + 4 + 4 + 2 = 14
- Command "say Hello" (9 bytes): Length = 4 + 4 + 9 + 2 = 19

### Request ID Management

- Use unique, incrementing request IDs for each packet
- The server echoes the request ID in responses
- If authentication fails, the server returns request ID `-1`
- Always verify the response ID matches your request ID

### Multi-Packet Responses

Some commands may generate responses that span multiple packets. The server may send:
1. An empty response packet (type 0, empty payload)
2. The actual response packet (type 0, with data)

You may need to read multiple packets until you receive the complete response.

### Error Handling

- **Authentication Failure**: Response ID will be `-1`
- **Connection Refused**: Server not running or RCON not enabled
- **Timeout**: Server not responding
- **Invalid Packet**: Server may close connection

## How to Pass Commands

To pass a command to the RCON server, you need to:

1. **First authenticate** (see Step 1 above)
2. **Then send a command packet** with the command string in the payload

The command is simply the text you want to execute, placed in the `WRITE_STRING` field of the command execution packet.

**Important**: RCON uses **fixed 4-byte little-endian integers** (`WRITE_INT`), **NOT VarInts** (`WRITE_VARINT`). Also use `WRITE_STRING` (null-terminated), not `WRITE_STRING_LEN`.

### Basic Command Example

Here's how to send the `list` command:

```pseudo
# Command: "list"
# Length = 4 + 4 + 4 + 2 = 14

PACKET_START
WRITE_INT 14      # Length
WRITE_INT 2       # Request ID
WRITE_INT 2       # Type: SERVERDATA_EXECCOMMAND
WRITE_STRING "list"  # The command to execute
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END
```

### Commands with Parameters

For commands that take parameters, include the full command string. The parser now handles quoted strings with spaces correctly:

```pseudo
# Command: "say Hello everyone!"
# Length = 4 + 4 + 18 + 2 = 28

PACKET_START
WRITE_INT 28
WRITE_INT 3       # Request ID (increment for each command)
WRITE_INT 2       # Type: SERVERDATA_EXECCOMMAND
WRITE_STRING "say Hello everyone!"  # Full command with parameters (quotes with spaces work)
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END
```

```pseudo
# Command: "spark tps"
# Length = 4 + 4 + 9 + 2 = 19

PACKET_START
WRITE_INT 19
WRITE_INT 2       # Request ID
WRITE_INT 2       # Type: SERVERDATA_EXECCOMMAND
WRITE_STRING "spark tps"  # Command with spaces - quotes are handled correctly
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END
```

```pseudo
# Command: "whitelist add PlayerName"
# Length = 4 + 4 + 24 + 2 = 34

PACKET_START
WRITE_INT 34
WRITE_INT 4       # Request ID
WRITE_INT 2       # Type: SERVERDATA_EXECCOMMAND
WRITE_STRING "whitelist add PlayerName"
WRITE_BYTE 0x00
WRITE_BYTE 0x00
PACKET_END
```

### Important Notes

1. **Calculate length for each command**: The length changes based on the command string length
2. **Increment Request ID**: Use a unique, incrementing request ID for each command (1 for auth, 2 for first command, 3 for second, etc.)
3. **Command format**: Write the exact command as you would type it in the Minecraft server console
4. **Multiple commands**: Send separate packets for each command, incrementing the Request ID each time

### Workflow Summary

```
1. Connect via TCP to server:port (e.g., localhost:25575)
2. Send AUTH packet (Request ID: 1, Type: 3, Password in payload)
3. Read AUTH response (verify Request ID matches and Type is 0)
4. Send COMMAND packet (Request ID: 2, Type: 2, Command in payload)
5. Read COMMAND response (Request ID matches, Type is 0, output in payload)
6. Send more commands as needed (increment Request ID each time)
7. Close connection when done
```

## Example Commands

Common RCON commands you might execute:

- `list` - List online players (4 bytes, length = 14)
- `say <message>` - Broadcast message to all players
- `stop` - Stop the server (4 bytes, length = 14)
- `whitelist add <player>` - Add player to whitelist
- `ban <player>` - Ban a player
- `op <player>` - Grant operator status
- `time set day` - Set time to day (9 bytes, length = 19)
- `weather clear` - Set weather to clear (13 bytes, length = 23)

## Security Considerations

1. **Never expose RCON port to public internet** - Use firewall rules or VPN
2. **Use strong passwords** - RCON has no rate limiting
3. **Consider SSH tunneling** - `ssh -L 25575:localhost:25575 user@server`
4. **Monitor RCON access** - Log all commands executed
5. **Disable when not needed** - Set `enable-rcon=false` if not using

## Testing

To test your RCON connection:

1. Ensure Minecraft server is running with RCON enabled
2. Use the authentication pseudo-code first
3. Verify successful authentication (response_id matches, response_type is 0)
4. Send a simple command like `list`
5. Parse and display the response

## References

- [Minecraft Wiki - RCON](https://minecraft.wiki/w/RCON)
- Source RCON Protocol (Minecraft uses a similar protocol)
- RCON implementations in various languages can provide additional insights

## Integration with Net Sentinel

To use RCON with Net Sentinel's game server checker:

1. Set protocol to `TCP`
2. Set port to your RCON port (default 25575)
3. Use the authentication pseudo-code as the first packet
4. Optionally add command execution as additional packets
5. Parse responses to extract server status information

**Note**: You may need to handle the length field calculation manually or extend the packet builder to support RCON's specific packet format requirements.

