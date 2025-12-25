# Hands-On Demo: RCON and Minecraft Ping

This guide walks you through two complete, real-world examples:
1. **RCON with TabTPS** - Querying server performance metrics via RCON
2. **Minecraft Status Ping** - Getting server status using Minecraft's protocol

We'll build each script step-by-step, explaining every command and how it works.

## Table of Contents

1. [RCON with TabTPS Demo](#rcon-with-tabtps-demo)
2. [Minecraft Ping Demo](#minecraft-ping-demo)
3. [Testing Your Scripts](#testing-your-scripts)
4. [Troubleshooting](#troubleshooting)

---

## RCON with TabTPS Demo

This example shows how to:
- Authenticate with an RCON server
- Execute a command (`mspt` from TabTPS mod)
- Parse complex text output
- Extract multiple metrics (TPS, MSPT, CPU, Memory)

### Understanding the Protocol

**RCON Protocol:**
- Uses TCP connection
- Requires authentication before commands
- Packets have: Length (4 bytes), Request ID (4 bytes), Type (4 bytes), Payload (string + 2 null bytes)
- Type `3` = Authentication, Type `2` = Command execution

**TabTPS Mod:**
- Fabric mod that provides server performance metrics
- `mspt` command returns TPS, MSPT, CPU, and Memory information
- Output is formatted text that needs parsing

### Step 1: Authentication Packet

First, we need to authenticate with the RCON server.

```pseudo
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1              // REQUEST_ID (pick any positive int)
WRITE_INT 3              // AUTH
WRITE_STRING_LEN "<CODE>" 32
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END
```

**Breaking it down:**
- `WRITE_INT PACKET_LEN` - Auto-calculates the total packet length (excluding this field)
- `WRITE_INT 1` - Request ID (can be any number, used to match responses)
- `WRITE_INT 3` - Type: `3` means "SERVERDATA_AUTH" (authentication request)
- `WRITE_STRING_LEN "<CODE>" 32` - Your RCON password, fixed to 32 bytes (padded with nulls)
- `WRITE_BYTE 0` - First null terminator (RCON requires two null bytes)
- `WRITE_BYTE 0` - Second null terminator

**Note:** Replace `<CODE>` with your actual RCON password. The `WRITE_STRING_LEN` ensures it's exactly 32 bytes.

### Step 2: Parse Authentication Response

```pseudo
RESPONSE_START
READ_INT RESPONSE_LEN_AUTH
READ_INT RESPONSE_ID_AUTH
READ_INT RESPONSE_TYPE_AUTH
READ_STRING_NULL COMMAND_OUT_AUTH
RESPONSE_END
```

**Breaking it down:**
- `READ_INT RESPONSE_LEN_AUTH` - Packet length (we'll store it but may not use it)
- `READ_INT RESPONSE_ID_AUTH` - Should match our request ID (1)
- `READ_INT RESPONSE_TYPE_AUTH` - Response type
- `READ_STRING_NULL COMMAND_OUT_AUTH` - Authentication result message

**What to expect:**
- If authentication succeeds, `RESPONSE_ID_AUTH` will match `1` (our request ID)
- If it fails, you'll get an error response

### Step 3: Command Execution Packet

Now that we're authenticated, we can execute commands.

```pseudo
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 2
WRITE_INT 2
WRITE_STRING_LEN "mspt" 4
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END
```

**Breaking it down:**
- `WRITE_INT PACKET_LEN` - Auto-calculates packet length
- `WRITE_INT 2` - Request ID (different from auth, so we can track this separately)
- `WRITE_INT 2` - Type: `2` means "SERVERDATA_EXECCOMMAND" (command execution)
- `WRITE_STRING_LEN "mspt" 4` - The command to execute (TabTPS's `mspt` command)
- `WRITE_BYTE 0` - First null terminator
- `WRITE_BYTE 0` - Second null terminator

**The `mspt` command:**
- This is specific to the TabTPS mod for Fabric
- Returns server performance metrics in a formatted text output

### Step 4: Parse Command Response

```pseudo
RESPONSE_START
READ_INT RESPONSE_LEN
READ_INT RESPONSE_ID
READ_INT RESPONSE_TYPE
READ_STRING_NULL COMMAND_OUT
RESPONSE_END
```

**Breaking it down:**
- `READ_INT RESPONSE_LEN` - Packet length
- `READ_INT RESPONSE_ID` - Should match our request ID (2)
- `READ_INT RESPONSE_TYPE` - Response type
- `READ_STRING_NULL COMMAND_OUT` - **This contains the command output** - the TabTPS metrics!

**What you'll get:**
The `COMMAND_OUT` variable will contain text like:
```
TPS: 20.0 (19.8, 19.9, 20.0)
├─ 5s - 50.0, 45.0, 55.0
CPU: 15.2%, 12.5% (sys. 2.7%)
RAM: 928M / 1120M (max. 10240M)
```

### Step 5: Parse TabTPS Output

Now we need to extract specific values from the text output. This is where code blocks shine!

```pseudo
CODE_START
STRING output = COMMAND_OUT

# Extract TPS: Split by "TPS: " then by " ("
ARRAY tps_split1 = SPLIT(output, "TPS: ")
STRING tps_section = tps_split1[1]
ARRAY tps_split2 = SPLIT(tps_section, " (")
STRING tps_str = tps_split2[0]
FLOAT tps_fs_out = tps_str
ARRAY tps_split3 = SPLIT(tps_split2[1], ",")
FLOAT tps_om_out = REPLACE(tps_split3[1], " ", "")
ARRAY tps_split4 = SPLIT(tps_split2[2], ",")
FLOAT tps_fm_out = REPLACE(tps_split4[1], " ", "")
ARRAY tps_split5 = SPLIT(tps_split2[3], ",")
FLOAT tps_ftm_out = REPLACE(tps_split5[1], " ", "")
```

**Breaking it down:**

1. **Store the output:**
   ```pseudo
   STRING output = COMMAND_OUT
   ```
   Copy the command output to a variable we can manipulate.

2. **Extract TPS values:**
   - Split by `"TPS: "` to get everything after "TPS: "
   - Split by `" ("` to separate the main TPS from the averages
   - The format is: `20.0 (19.8, 19.9, 20.0)`
   - `tps_fs_out` = First/second value (current TPS: 20.0)
   - `tps_om_out` = One-minute average
   - `tps_fm_out` = Five-minute average
   - `tps_ftm_out` = Fifteen-minute average

**Example parsing:**
```
Input: "TPS: 20.0 (19.8, 19.9, 20.0)"
After SPLIT(output, "TPS: "): ["", "20.0 (19.8, 19.9, 20.0)"]
tps_section = "20.0 (19.8, 19.9, 20.0)"
After SPLIT(tps_section, " ("): ["20.0", "19.8, 19.9, 20.0)"]
tps_str = "20.0"
tps_fs_out = 20.0
```

### Step 6: Extract MSPT (Milliseconds Per Tick)

```pseudo
# Extract MSPT: Find "├─ 5s - " line
ARRAY mspt_lines = SPLIT(output, "├─")
STRING mspt_line = ""
FOR line IN mspt_lines:
  IF line CONTAINS "5s - ":
    mspt_line = line
    BREAK

# Extract MSPT values from the line
ARRAY mspt_split1 = SPLIT(mspt_line, "5s - ")
STRING mspt_values_str = mspt_split1[1]
ARRAY mspt_array = SPLIT(mspt_values_str, ",")
FLOAT mspt_ave_out = mspt_array[0]
FLOAT mspt_min_out = mspt_array[1]
FLOAT mspt_max_out = mspt_array[2]
```

**Breaking it down:**

1. **Find the MSPT line:**
   - Split output by `"├─"` (the tree character in TabTPS output)
   - Loop through lines to find one containing `"5s - "`
   - This line contains: `├─ 5s - 50.0, 45.0, 55.0`

2. **Extract MSPT values:**
   - Split by `"5s - "` to get the values part
   - Split by `","` to get individual values
   - `mspt_ave_out` = Average MSPT
   - `mspt_min_out` = Minimum MSPT
   - `mspt_max_out` = Maximum MSPT

**Example parsing:**
```
Input line: "├─ 5s - 50.0, 45.0, 55.0"
After SPLIT(mspt_line, "5s - "): ["├─ ", "50.0, 45.0, 55.0"]
mspt_values_str = "50.0, 45.0, 55.0"
After SPLIT(mspt_values_str, ","): ["50.0", " 45.0", " 55.0"]
mspt_ave_out = 50.0
mspt_min_out = 45.0
mspt_max_out = 55.0
```

### Step 7: Extract CPU Usage

```pseudo
# Extract CPU: Split by "CPU: " then by ","
ARRAY cpu_split1 = SPLIT(output, "CPU: ")
STRING cpu_section = cpu_split1[1]
ARRAY cpu_split2 = SPLIT(cpu_section, ',')
STRING cpu_sys_str = REPLACE(cpu_split2[0], "%", "")
STRING cpu_proc_str = REPLACE(cpu_split2[1], "% (sys.", "")
FLOAT cpu_sys_out = cpu_sys_str
FLOAT cpu_proc_out = cpu_proc_str
```

**Breaking it down:**

1. **Extract CPU section:**
   - Split by `"CPU: "` to get the CPU line
   - Format: `CPU: 15.2%, 12.5% (sys. 2.7%)`

2. **Parse CPU values:**
   - Split by `","` to separate values
   - Remove `"%"` and `"% (sys."` to get clean numbers
   - `cpu_sys_out` = System CPU usage
   - `cpu_proc_out` = Process CPU usage

### Step 8: Extract Memory Usage

```pseudo
# Extract Memory: Split by "RAM: " then by "/" then by " "
ARRAY mem_split1 = SPLIT(output, "RAM: ")
STRING mem_section = mem_split1[1]
ARRAY mem_split2 = SPLIT(mem_section, "/")
STRING mem_used_str = REPLACE(mem_split2[0], "M", "")
STRING mem_stack_section = mem_split2[1]
INT mem_used_out = mem_used_str

ARRAY mem_stack_split = SPLIT(mem_stack_section, ' ')
STRING mem_stack_str = REPLACE(mem_stack_split[0], 'M', '')
STRING mem_max_str = REPLACE(mem_stack_split[2], 'M)[...]', '')
INT mem_stack_out = mem_stack_str
INT mem_max_out = mem_max_str
```

**Breaking it down:**

1. **Extract RAM section:**
   - Format: `RAM: 928M / 1120M (max. 10240M)`

2. **Parse memory values:**
   - Split by `"/"` to separate used from total
   - `mem_used_out` = Used memory (928M)
   - Split the second part by spaces to get stack and max
   - `mem_stack_out` = Stack memory (1120M)
   - `mem_max_out` = Maximum memory (10240M)

### Step 9: Format Output

```pseudo
OUTPUT_SUCCESS
RETURN "mem_max=mem_max_out, mem_stack=mem_stack_out, mem_used=mem_used_out, cpu_sys=cpu_sys_out, cpu_proc=cpu_proc_out, mspt_ave=mspt_ave_out, mspt_min=mspt_min_out, mspt_max=mspt_max_out, tps=tps_fs_out, tps_one_min=tps_om_out, tps_five_min=tps_fm_out, tps_fifteen_min=tps_ftm_out"
OUTPUT_END

OUTPUT_ERROR
RETURN "server='HOST', error=ERROR"
OUTPUT_END
```

**Breaking it down:**

- **Success output:** Returns all extracted metrics as key-value pairs
- **Error output:** Returns error information if something goes wrong

**Final output format:**
```
mem_max=10240, mem_stack=1120, mem_used=928, cpu_sys=15.2, cpu_proc=12.5, mspt_ave=50.0, mspt_min=45.0, mspt_max=55.0, tps=20.0, tps_one_min=19.8, tps_five_min=19.9, tps_fifteen_min=20.0
```

### Complete RCON Script

Here's the complete script (replace `<CODE>` with your RCON password):

```pseudo
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1
WRITE_INT 3
WRITE_STRING_LEN "<CODE>" 32
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END

RESPONSE_START
READ_INT RESPONSE_LEN_AUTH
READ_INT RESPONSE_ID_AUTH
READ_INT RESPONSE_TYPE_AUTH
READ_STRING_NULL COMMAND_OUT_AUTH
RESPONSE_END

PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 2
WRITE_INT 2
WRITE_STRING_LEN "mspt" 4
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END

RESPONSE_START
READ_INT RESPONSE_LEN
READ_INT RESPONSE_ID
READ_INT RESPONSE_TYPE
READ_STRING_NULL COMMAND_OUT
RESPONSE_END

CODE_START
STRING output = COMMAND_OUT
ARRAY tps_split1 = SPLIT(output, "TPS: ")
STRING tps_section = tps_split1[1]
ARRAY tps_split2 = SPLIT(tps_section, " (")
STRING tps_str = tps_split2[0]
FLOAT tps_fs_out = tps_str
ARRAY tps_split3 = SPLIT(tps_split2[1], ",")
FLOAT tps_om_out = REPLACE(tps_split3[1], " ", "")
ARRAY tps_split4 = SPLIT(tps_split2[2], ",")
FLOAT tps_fm_out = REPLACE(tps_split4[1], " ", "")
ARRAY tps_split5 = SPLIT(tps_split2[3], ",")
FLOAT tps_ftm_out = REPLACE(tps_split5[1], " ", "")
ARRAY mspt_lines = SPLIT(output, "├─")
STRING mspt_line = ""
FOR line IN mspt_lines:
  IF line CONTAINS "5s - ":
    mspt_line = line
    BREAK
ARRAY mspt_split1 = SPLIT(mspt_line, "5s - ")
STRING mspt_values_str = mspt_split1[1]
ARRAY mspt_array = SPLIT(mspt_values_str, ",")
FLOAT mspt_ave_out = mspt_array[0]
FLOAT mspt_min_out = mspt_array[1]
FLOAT mspt_max_out = mspt_array[2]
ARRAY cpu_split1 = SPLIT(output, "CPU: ")
STRING cpu_section = cpu_split1[1]
ARRAY cpu_split2 = SPLIT(cpu_section, ',')
STRING cpu_sys_str = REPLACE(cpu_split2[0], "%", "")
STRING cpu_proc_str = REPLACE(cpu_split2[1], "% (sys.", "")
FLOAT cpu_sys_out = cpu_sys_str
FLOAT cpu_proc_out = cpu_proc_str
ARRAY mem_split1 = SPLIT(output, "RAM: ")
STRING mem_section = mem_split1[1]
ARRAY mem_split2 = SPLIT(mem_section, "/")
STRING mem_used_str = REPLACE(mem_split2[0], "M", "")
STRING mem_stack_section = mem_split2[1]
INT mem_used_out = mem_used_str
ARRAY mem_stack_split = SPLIT(mem_stack_section, ' ')
STRING mem_stack_str = REPLACE(mem_stack_split[0], 'M', '')
STRING mem_max_str = REPLACE(mem_stack_split[2], 'M)[...]', '')
INT mem_stack_out = mem_stack_str
INT mem_max_out = mem_max_str
CODE_END

OUTPUT_SUCCESS
RETURN "mem_max=mem_max_out, mem_stack=mem_stack_out, mem_used=mem_used_out, cpu_sys=cpu_sys_out, cpu_proc=cpu_proc_out, mspt_ave=mspt_ave_out, mspt_min=mspt_min_out, mspt_max=mspt_max_out, tps=tps_fs_out, tps_one_min=tps_om_out, tps_five_min=tps_fm_out, tps_fifteen_min=tps_ftm_out"
OUTPUT_END

OUTPUT_ERROR
RETURN "server='HOST', error=ERROR"
OUTPUT_END
```

---

## Minecraft Ping Demo

This example shows how to:
- Send a Minecraft handshake packet
- Request server status
- Parse JSON response
- Extract server information

### Understanding the Protocol

**Minecraft Protocol:**
- Uses TCP connection
- Uses VarInt encoding (variable-length integers)
- Two-step process: Handshake → Status Request
- Response is JSON string

**Protocol Flow:**
1. **Handshake:** Establish connection and request status state
2. **Status Request:** Request server status
3. **Response:** Receive JSON with server information

### Step 1: Handshake Packet

The handshake tells the server we want status information.

```pseudo
PACKET_START
WRITE_VARINT PACKET_LEN
WRITE_VARINT 0x00
WRITE_VARINT 0x47
WRITE_VARINT IP_LEN
WRITE_STRING_LEN "HOST" IP_LEN
WRITE_SHORT_BE PORT
WRITE_VARINT 0x01
PACKET_END
```

**Breaking it down:**

1. **`WRITE_VARINT PACKET_LEN`**
   - Auto-calculates packet length as a VarInt
   - VarInts use 1-5 bytes depending on value

2. **`WRITE_VARINT 0x00`**
   - Packet ID: `0` = Handshake packet

3. **`WRITE_VARINT 0x47`**
   - Protocol version: `0x47` = 71 (decimal)
   - This is the protocol version number

4. **`WRITE_VARINT IP_LEN`**
   - Length of server address as VarInt
   - `IP_LEN` is automatically replaced with actual hostname length

5. **`WRITE_STRING_LEN "HOST" IP_LEN`**
   - Server hostname/address
   - `HOST` is automatically replaced with actual server address
   - Length must match `IP_LEN`

6. **`WRITE_SHORT_BE PORT`**
   - Server port (big-endian, 2 bytes)
   - `PORT` is automatically replaced with actual port

7. **`WRITE_VARINT 0x01`**
   - Next state: `1` = Status (we want status information)

**What happens:**
- Server receives handshake and prepares for status request
- Connection stays open for the next packet

### Step 2: Status Request Packet

Now we request the actual status.

```pseudo
PACKET_START
WRITE_VARINT 0x01
WRITE_VARINT 0x00
PACKET_END
```

**Breaking it down:**

1. **`WRITE_VARINT 0x01`**
   - Packet length: `1` byte (just the packet ID)

2. **`WRITE_VARINT 0x00`**
   - Packet ID: `0` = Status Request

**Why so simple?**
- Status request is just a packet ID, no other data needed
- Server already knows we want status from the handshake

### Step 3: Parse Response

The server responds with a JSON string containing server information.

```pseudo
RESPONSE_START
READ_VARINT LENGTH_VARINT
READ_VARINT PACKET_ID
READ_VARINT JSON_LENGTH_VARINT
READ_STRING_NULL JSON_PAYLOAD
RESPONSE_END
```

**Breaking it down:**

1. **`READ_VARINT LENGTH_VARINT`**
   - Total packet length (VarInt)
   - We read it but may not need it

2. **`READ_VARINT PACKET_ID`**
   - Should be `0` (Status Response packet ID)

3. **`READ_VARINT JSON_LENGTH_VARINT`**
   - Length of JSON string (VarInt)
   - Minecraft prefixes the JSON with its length

4. **`READ_STRING_NULL JSON_PAYLOAD`**
   - The actual JSON string
   - Contains server information like version, players, description

**What you'll get:**
The `JSON_PAYLOAD` variable will contain JSON like:
```json
{
  "version": {
    "name": "1.20.1",
    "protocol": 763
  },
  "players": {
    "max": 100,
    "online": 42
  },
  "description": {
    "text": "Welcome to our server!"
  }
}
```

### Step 4: Format Output

```pseudo
OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "protocol=JSON_PAYLOAD.version.protocol"
OUTPUT_END

OUTPUT_ERROR
RETURN "server=HOST, error=ERROR"
OUTPUT_END
```

**Breaking it down:**

1. **`JSON_OUTPUT JSON_PAYLOAD`**
   - Parses the JSON string into a structured object
   - Makes nested fields accessible like `JSON_PAYLOAD.version.protocol`

2. **`RETURN "protocol=..."`**
   - Formats output with extracted values
   - `JSON_PAYLOAD.version.protocol` accesses the nested protocol field

**Available JSON fields:**
- `JSON_PAYLOAD.version.name` - Version name (e.g., "1.20.1")
- `JSON_PAYLOAD.version.protocol` - Protocol version number
- `JSON_PAYLOAD.players.online` - Current player count
- `JSON_PAYLOAD.players.max` - Maximum players
- `JSON_PAYLOAD.description.text` - Server description

### Complete Minecraft Script

Here's the complete script:

```pseudo
PACKET_START
WRITE_VARINT PACKET_LEN
WRITE_VARINT 0x00
WRITE_VARINT 0x47
WRITE_VARINT IP_LEN
WRITE_STRING_LEN "HOST" IP_LEN
WRITE_SHORT_BE PORT
WRITE_VARINT 0x01
PACKET_END

PACKET_START
WRITE_VARINT 0x01
WRITE_VARINT 0x00
PACKET_END

RESPONSE_START
READ_VARINT LENGTH_VARINT
READ_VARINT PACKET_ID
READ_VARINT JSON_LENGTH_VARINT
READ_STRING_NULL JSON_PAYLOAD
RESPONSE_END

OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "protocol=JSON_PAYLOAD.version.protocol, players=JSON_PAYLOAD.players.online, max=JSON_PAYLOAD.players.max"
OUTPUT_END

OUTPUT_ERROR
RETURN "server=HOST, error=ERROR"
OUTPUT_END
```

**Enhanced version with more fields:**

```pseudo
PACKET_START
WRITE_VARINT PACKET_LEN
WRITE_VARINT 0x00
WRITE_VARINT 0x47
WRITE_VARINT IP_LEN
WRITE_STRING_LEN "HOST" IP_LEN
WRITE_SHORT_BE PORT
WRITE_VARINT 0x01
PACKET_END

PACKET_START
WRITE_VARINT 0x01
WRITE_VARINT 0x00
PACKET_END

RESPONSE_START
READ_VARINT LENGTH_VARINT
READ_VARINT PACKET_ID
READ_VARINT JSON_LENGTH_VARINT
READ_STRING_NULL JSON_PAYLOAD
RESPONSE_END

OUTPUT_SUCCESS
JSON_OUTPUT JSON_PAYLOAD
RETURN "version=JSON_PAYLOAD.version.name, protocol=JSON_PAYLOAD.version.protocol, players=JSON_PAYLOAD.players.online, max=JSON_PAYLOAD.players.max, description=JSON_PAYLOAD.description.text"
OUTPUT_END

OUTPUT_ERROR
RETURN "server=HOST, error=ERROR"
OUTPUT_END
```

---

## Testing Your Scripts

### Testing RCON Script

1. **Set up your game server:**
   - Ensure RCON is enabled
   - Note your RCON password
   - Install TabTPS mod (for Fabric)

2. **Configure in Net Sentinel:**
   - Protocol: `TCP`
   - Address: Your server IP
   - Port: RCON port (usually 25575)
   - Replace `<CODE>` with your RCON password

3. **Test the script:**
   - Use the test endpoint to verify it works
   - Check that authentication succeeds
   - Verify command execution returns TabTPS output
   - Confirm metrics are extracted correctly

### Testing Minecraft Script

1. **Set up your Minecraft server:**
   - Ensure server is running
   - Note server address and port

2. **Configure in Net Sentinel:**
   - Protocol: `TCP`
   - Address: Server IP or hostname
   - Port: Server port (default 25565)

3. **Test the script:**
   - Use the test endpoint
   - Verify handshake succeeds
   - Check status request works
   - Confirm JSON is parsed correctly

### Common Issues

**RCON Authentication Fails:**
- Check RCON password is correct
- Verify RCON is enabled on server
- Ensure password length matches (32 bytes in example)

**Minecraft Connection Fails:**
- Check server is online
- Verify port is correct
- Ensure firewall allows connections
- Check protocol version (0x47 = 71, may need updating for newer versions)

**Parsing Errors:**
- Verify output format matches expected format
- Check for typos in SPLIT delimiters
- Ensure array indices are correct (0-based)

---

## Troubleshooting

### RCON Issues

**Problem: Authentication always fails**
- **Solution:** Check password is exactly 32 bytes. Use `WRITE_STRING_LEN` with length 32.

**Problem: Command output is empty**
- **Solution:** Verify TabTPS mod is installed and `mspt` command works in-game.

**Problem: Parsing fails on specific field**
- **Solution:** Check the actual output format. TabTPS output may vary by version.

### Minecraft Issues

**Problem: Connection timeout**
- **Solution:** 
  - Check server is online
  - Verify port (default 25565)
  - Check firewall settings

**Problem: Wrong protocol version**
- **Solution:** Update `WRITE_VARINT 0x47` to match your server version:
  - 1.20.1 = 0x47 (71)
  - 1.20.2 = 0x48 (72)
  - Check [Minecraft Protocol Version](https://wiki.vg/Protocol_version_numbers)

**Problem: JSON parsing fails**
- **Solution:**
  - Verify `READ_STRING_NULL` reads the complete JSON
  - Check JSON is valid (no truncation)
  - Ensure `JSON_OUTPUT` is called before accessing nested fields

### General Debugging Tips

1. **Start simple:**
   - Test with minimal script first
   - Add complexity gradually

2. **Check raw responses:**
   - Look at `raw_response` in test results
   - Verify bytes match expectations

3. **Test parsing step-by-step:**
   - Parse one field at a time
   - Verify each step works before adding next

4. **Use comments:**
   - Document what each part does
   - Makes debugging easier

---

## Next Steps

Now that you've seen complete examples:

1. **Adapt for your needs:**
   - Modify RCON password handling
   - Add more Minecraft fields
   - Create custom parsing logic

2. **Explore other protocols:**
   - Try Source Engine query
   - Experiment with custom protocols
   - Check [Examples](03-examples.md) for more patterns

3. **Optimize your scripts:**
   - Remove unnecessary parsing
   - Combine similar operations
   - Add error handling

4. **Learn more:**
   - Read [How It Works](04-how-it-works.md) for implementation details
   - Check [Syntax Reference](02-pseudo-code-syntax.md) for all commands
   - Review [Beginner's Guide](01-beginners-guide.md) for basics

---

**Congratulations!** You've completed two real-world examples. You now have the knowledge to create monitoring scripts for any game server protocol! 🎮

