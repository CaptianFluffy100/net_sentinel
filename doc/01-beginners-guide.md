# Beginner's Guide to Writing Pseudo-Code

Welcome! This guide will teach you how to write pseudo-code scripts for monitoring game servers. No programming experience required!

## What You'll Learn

By the end of this guide, you'll be able to:
- Write simple pseudo-code scripts
- Understand packet construction
- Parse server responses
- Format output for monitoring

## Understanding the Basics

### What is a Protocol?

A **protocol** is like a language that computers use to communicate. When you connect to a game server, you need to:
1. **Send a message** (a packet) in the format the server expects
2. **Receive a response** from the server
3. **Interpret the response** to extract useful information

### What is a Packet?

A **packet** is a sequence of bytes (numbers from 0-255) that represents a message. Think of it like a letter:
- The **envelope** (packet structure) tells the server what kind of message it is
- The **content** (data) contains the actual information

### Example: Saying "Hello" to a Server

Imagine you want to ask a server "Are you online?":

1. **You send**: A packet with bytes `[0x01, 0x48, 0x65, 0x6C, 0x6C, 0x6F]`
   - `0x01` = "This is a status request"
   - `0x48, 0x65, 0x6C, 0x6C, 0x6F` = "Hello" in ASCII

2. **Server responds**: `[0x01, 0x4F, 0x4B]`
   - `0x01` = "This is a status response"
   - `0x4F, 0x4B` = "OK" in ASCII

3. **You interpret**: Server is online!

## Your First Pseudo-Code Script

Let's write a simple script that sends a single byte and reads the response:

```
PACKET_START
WRITE_BYTE 0x01
PACKET_END

RESPONSE_START
READ_BYTE status
RESPONSE_END
```

### Breaking It Down

**`PACKET_START`** - "I'm about to describe a packet to send"
- This marks the beginning of packet construction

**`WRITE_BYTE 0x01`** - "Write the byte value 1 (0x01 in hexadecimal)"
- This adds one byte to the packet

**`PACKET_END`** - "I'm done describing the packet"
- This marks the end of packet construction

**`RESPONSE_START`** - "I'm about to describe how to parse the response"
- This marks the beginning of response parsing

**`READ_BYTE status`** - "Read one byte and store it in a variable called 'status'"
- This extracts one byte from the response and saves it

**`RESPONSE_END`** - "I'm done describing the response parsing"
- This marks the end of response parsing

## Understanding Data Types

### Bytes (0-255)

A **byte** is a number from 0 to 255. You can write it in two ways:
- **Decimal**: `255` (normal number)
- **Hexadecimal**: `0xFF` (starts with `0x`, uses letters A-F)

Examples:
```
WRITE_BYTE 255      # Decimal
WRITE_BYTE 0xFF     # Hexadecimal (same value)
WRITE_BYTE 0x01     # Hexadecimal for 1
```

### Strings (Text)

A **string** is text. You write it in quotes:
```
WRITE_STRING "Hello Server"
WRITE_STRING "status"
```

### Numbers (Integers)

Larger numbers are written as **integers**:
- **Short** (16-bit): 0 to 65,535
- **Int** (32-bit): 0 to 4,294,967,295

```
WRITE_SHORT 1234
WRITE_INT 50000
```

## Common Patterns

### Pattern 1: Simple Request/Response

Send a simple command, get a simple response:

```
PACKET_START
WRITE_STRING "ping"
PACKET_END

RESPONSE_START
READ_STRING_NULL response
RESPONSE_END
```

### Pattern 2: Magic Bytes

Many protocols start with "magic bytes" - special bytes that identify the protocol:

```
PACKET_START
WRITE_BYTE 0xFE
WRITE_BYTE 0xFD
WRITE_BYTE 0x09
PACKET_END

RESPONSE_START
EXPECT_BYTE 0xFE    # Must be 0xFE or error
EXPECT_BYTE 0xFD    # Must be 0xFD or error
READ_BYTE packet_type
RESPONSE_END
```

### Pattern 3: Length-Prefixed Packets

Some protocols put the packet length at the beginning:

```
PACKET_START
WRITE_INT PACKET_LEN    # Auto-calculates length
WRITE_STRING "command"
PACKET_END
```

The `PACKET_LEN` placeholder automatically calculates how long your packet is!

## Step-by-Step: Writing Your First Real Script

Let's create a script for a hypothetical "Simple Game Server" protocol:

### Step 1: Understand the Protocol

The protocol documentation says:
- Send: `[0x01]` (status request)
- Receive: `[0x01, player_count (2 bytes), max_players (2 bytes)]`

### Step 2: Write the Packet

```
PACKET_START
WRITE_BYTE 0x01
PACKET_END
```

### Step 3: Write the Response Parsing

```
RESPONSE_START
READ_BYTE response_type
READ_SHORT player_count
READ_SHORT max_players
RESPONSE_END
```

### Step 4: Format the Output

```
OUTPUT_SUCCESS
RETURN "players=player_count, max=max_players"
OUTPUT_END
```

### Complete Script

```
PACKET_START
WRITE_BYTE 0x01
PACKET_END

RESPONSE_START
READ_BYTE response_type
READ_SHORT player_count
READ_SHORT max_players
RESPONSE_END

OUTPUT_SUCCESS
RETURN "players=player_count, max=max_players"
OUTPUT_END
```

## Understanding Endianness

**Endianness** determines the order of bytes in multi-byte numbers.

### Little-Endian (Default)
Bytes are ordered from least significant to most significant:
- Number `1234` (0x04D2) = `[0xD2, 0x04]`

### Big-Endian (Network Order)
Bytes are ordered from most significant to least significant:
- Number `1234` (0x04D2) = `[0x04, 0xD2]`

**When to use which?**
- Most protocols use **big-endian** (network byte order)
- Use `WRITE_SHORT_BE` and `READ_SHORT_BE` for big-endian
- Use `WRITE_SHORT` and `READ_SHORT` for little-endian (default)

Example:
```
PACKET_START
WRITE_SHORT_BE 1234    # Big-endian
PACKET_END

RESPONSE_START
READ_SHORT_BE port     # Big-endian
RESPONSE_END
```

## Common Mistakes and How to Avoid Them

### Mistake 1: Wrong Byte Order
**Problem**: Server expects big-endian, you send little-endian
**Solution**: Use `_BE` suffix for big-endian commands

### Mistake 2: Missing Null Terminator
**Problem**: String doesn't end properly
**Solution**: Use `WRITE_STRING` (auto-adds null) or `READ_STRING_NULL` (reads until null)

### Mistake 3: Wrong Packet Length
**Problem**: Length field doesn't match actual packet size
**Solution**: Use `PACKET_LEN` placeholder - it calculates automatically!

### Mistake 4: Reading in Wrong Order
**Problem**: Reading response fields in wrong sequence
**Solution**: Read fields in the exact order they appear in the response

## Testing Your Script

1. **Start with a simple packet** - just send one byte
2. **Check the response** - see what bytes you actually receive
3. **Add parsing gradually** - parse one field at a time
4. **Test with real server** - use the test endpoint to verify

## Next Steps

Now that you understand the basics:
1. Read the [Complete Syntax Reference](02-pseudo-code-syntax.md)
2. Check out [Real-World Examples](03-examples.md)
3. Learn [How It Works Internally](04-how-it-works.md)

## Quick Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `WRITE_BYTE` | Write one byte | `WRITE_BYTE 0xFF` |
| `WRITE_SHORT` | Write 2 bytes (little-endian) | `WRITE_SHORT 1234` |
| `WRITE_SHORT_BE` | Write 2 bytes (big-endian) | `WRITE_SHORT_BE 1234` |
| `WRITE_STRING` | Write text (null-terminated) | `WRITE_STRING "hello"` |
| `READ_BYTE` | Read one byte | `READ_BYTE status` |
| `READ_SHORT` | Read 2 bytes (little-endian) | `READ_SHORT count` |
| `READ_STRING_NULL` | Read text until null | `READ_STRING_NULL name` |
| `EXPECT_BYTE` | Verify byte matches | `EXPECT_BYTE 0xFE` |

## Practice Exercise

Try writing a script for this protocol:
- **Request**: Send byte `0x02` followed by string `"info"`
- **Response**: Receive byte `0x02`, then string with server name

<details>
<summary>Click to see solution</summary>

```
PACKET_START
WRITE_BYTE 0x02
WRITE_STRING "info"
PACKET_END

RESPONSE_START
EXPECT_BYTE 0x02
READ_STRING_NULL server_name
RESPONSE_END
```

</details>

