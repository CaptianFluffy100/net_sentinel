# Hands-On Demo: RCON, Minecraft, and Source Engine

This guide walks you through three complete, real-world examples:
1. **RCON with TabTPS** - Querying server performance metrics via RCON
2. **Minecraft Status Ping** - Getting server status using Minecraft's protocol
3. **Source Engine Query (Valheim)** - Querying Valheim and other Source Engine game servers

We'll build each script step-by-step, explaining every command and how it works.

## Table of Contents

1. [RCON with TabTPS Demo](#rcon-with-tabtps-demo)
2. [Minecraft Ping Demo](#minecraft-ping-demo)
3. [Source Engine Query Demo](#source-engine-query-demo)
4. [Testing Your Scripts](#testing-your-scripts)
5. [Troubleshooting](#troubleshooting)

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

**TabTPS Output Format:**
The `mspt` command returns output in this format:
```
[TabTPS] Server Tick Information
TPS: 20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)
MSPT - Average, Minimum, Maximum
 ├─ 5s - 0.08, 0.04, 0.18
 ├─ 10s - 0.07, 0.04, 0.18
 └─ 60s - 0.16, 0.04, 20.22
CPU: 0.14%, 0.07% (sys., proc.)
RAM: 457M/616M (max. 10240M)
[|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||]
```

Where:
- **TPS line**: Shows TPS values for different time periods (5s, 1m, 5m, 15m)
- **MSPT section**: Shows milliseconds per tick for different time windows (5s, 10s, 60s)
- **CPU line**: Shows system and process CPU usage percentages
- **RAM line**: Shows used memory, stack memory, and maximum memory

### Step 1: Authentication Packet

First, we need to authenticate with the RCON server.

```pseudo
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1              // REQUEST_ID (pick any positive int)
WRITE_INT 3              // AUTH
WRITE_STRING_LEN "<CODE>" "<CODE_LEN>"
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END
```

**Breaking it down:**
- `WRITE_INT PACKET_LEN` - Auto-calculates the total packet length (excluding this field)
- `WRITE_INT 1` - Request ID (can be any number, used to match responses)
- `WRITE_INT 3` - Type: `3` means "SERVERDATA_AUTH" (authentication request)
- `WRITE_STRING_LEN "<CODE>" "<CODE_LEN>"` - Your RCON password and its length
- `WRITE_BYTE 0` - First null terminator (RCON requires two null bytes)
- `WRITE_BYTE 0` - Second null terminator

**Note:** Replace `<CODE>` with your actual RCON password and `<CODE_LEN>` with the length of your password in bytes. The `WRITE_STRING_LEN` ensures the password field is exactly the specified length (padded with nulls if shorter, truncated if longer).

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
[TabTPS] Server Tick Information
TPS: 20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)
MSPT - Average, Minimum, Maximum
 ├─ 5s - 0.08, 0.04, 0.18
 ├─ 10s - 0.07, 0.04, 0.18
 └─ 60s - 0.16, 0.04, 20.22
CPU: 0.14%, 0.07% (sys., proc.)
RAM: 457M/616M (max. 10240M)
[|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||]
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
   - Split by `" ("` to separate the first TPS value from the rest
   - The format is: `20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)`
   - `tps_split2[0]` = First value: `"20.00"` → `tps_fs_out` = 5-second average
   - `tps_split2[1]` = `"5s), 20.00 (1m)"` → Split by comma, get second element → `tps_om_out` = 1-minute average
   - `tps_split2[2]` = `"5m), 20.00 (15m)"` → Split by comma, get second element → `tps_fm_out` = 5-minute average
   - `tps_split2[3]` = `"15m)"` → Split by comma, get second element → `tps_ftm_out` = 15-minute average

**How it works with the actual output:**
The parsing logic is flexible and works with the TabTPS output format. When you split by `" ("`, you get:
- `tps_split2[0]` = `"20.00"` (the first TPS value)
- `tps_split2[1]` = `"5s), 20.00 (1m)"` (contains the 1m value)
- `tps_split2[2]` = `"5m), 20.00 (15m)"` (contains the 5m value)
- `tps_split2[3]` = `"15m)"` (the last value)

By splitting these by comma and taking the appropriate elements, we extract each TPS value.

**Example parsing:**
```
Input: "TPS: 20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)"
After SPLIT(output, "TPS: "): ["", "20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)"]
tps_section = "20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)"
After SPLIT(tps_section, " ("): ["20.00", "5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)"]
tps_str = "20.00"
tps_fs_out = 20.00
After SPLIT(tps_split2[1], ","): ["5s)", " 20.00 (1m)", " 20.00 (5m)", " 20.00 (15m)"]
After REPLACE(tps_split3[1], " ", ""): "20.00 (1m)"
# Float conversion extracts the numeric value: 20.00
tps_om_out = 20.00
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
   - This line contains: `├─ 5s - 0.08, 0.04, 0.18`
   - Note: TabTPS also provides 10s and 60s averages, but we're extracting the 5s values

2. **Extract MSPT values:**
   - Split by `"5s - "` to get the values part
   - Split by `","` to get individual values
   - `mspt_ave_out` = Average MSPT (0.08)
   - `mspt_min_out` = Minimum MSPT (0.04)
   - `mspt_max_out` = Maximum MSPT (0.18)

**Example parsing:**
```
Input line: "├─ 5s - 0.08, 0.04, 0.18"
After SPLIT(mspt_line, "5s - "): ["├─ ", "0.08, 0.04, 0.18"]
mspt_values_str = "0.08, 0.04, 0.18"
After SPLIT(mspt_values_str, ","): ["0.08", " 0.04", " 0.18"]
mspt_ave_out = 0.08
mspt_min_out = 0.04
mspt_max_out = 0.18
```

### Step 7: Extract CPU Usage

```pseudo
# Extract CPU: Split by "CPU: " then by ","
ARRAY cpu_split1 = SPLIT(output, "CPU: ")
STRING cpu_section = cpu_split1[1]
ARRAY cpu_split2 = SPLIT(cpu_section, ',')

# Remove % signs
STRING cpu_sys_str = REPLACE(cpu_split2[0], "%", "")
STRING cpu_proc_str = REPLACE(cpu_split2[1], "% (sys.", "")
FLOAT cpu_sys_out = cpu_sys_str
FLOAT cpu_proc_out = cpu_proc_str
```

**Breaking it down:**

1. **Extract CPU section:**
   - Split by `"CPU: "` to get the CPU line
   - Format: `CPU: 0.14%, 0.07% (sys., proc.)`

2. **Parse CPU values:**
   - Split by `","` to separate values: `["0.14%", " 0.07% (sys., proc.)"]`
   - Remove `"%"` from the first value to get system CPU: `"0.14"`
   - Remove `"% (sys."` from the second value to get process CPU: `" 0.07% (proc.)"` → `" 0.07% (proc.)"`
   - Note: The `REPLACE(cpu_split2[1], "% (sys.", "")` removes `"% (sys."` which leaves `" 0.07% (proc.)"`, but the float conversion will extract the numeric value
   - `cpu_sys_out` = System CPU usage (0.14)
   - `cpu_proc_out` = Process CPU usage (0.07)

**Example parsing:**
```
Input: "CPU: 0.14%, 0.07% (sys., proc.)"
After SPLIT(output, "CPU: "): ["", "0.14%, 0.07% (sys., proc.)"]
cpu_section = "0.14%, 0.07% (sys., proc.)"
After SPLIT(cpu_section, ","): ["0.14%", " 0.07% (sys., proc.)"]
After REPLACE(cpu_split2[0], "%", ""): "0.14"
After REPLACE(cpu_split2[1], "% (sys.", ""): " 0.07% (proc.)"
# Float conversion extracts the numeric value: 0.07
cpu_sys_out = 0.14
cpu_proc_out = 0.07
```

### Step 8: Extract Memory Usage

```pseudo
# Extract Memory: Split by "RAM: " then by "/" then by " "
ARRAY mem_split1 = SPLIT(output, "RAM: ")
STRING mem_section = mem_split1[1]
ARRAY mem_split2 = SPLIT(mem_section, "/")
STRING mem_used_str = REPLACE(mem_split2[0], "M", "")  # "457M"
STRING mem_stack_section = mem_split2[1]  # "616M (max. 10240M)"
INT mem_used_out = mem_used_str

ARRAY mem_stack_split = SPLIT(mem_stack_section, ' ')
STRING mem_stack_str = REPLACE(mem_stack_split[0], 'M', '')  # "616M"
STRING mem_max_str = REPLACE(mem_stack_split[2], 'M)[|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||]', '')  # "10240M)"
INT mem_stack_out = mem_stack_str
INT mem_max_out = mem_max_str
```

**Breaking it down:**

1. **Extract RAM section:**
   - Format: `RAM: 457M/616M (max. 10240M)`
   - Note: No spaces around the `/` separator

2. **Parse memory values:**
   - Split by `"/"` to separate used from total: `["457M", "616M (max. 10240M)"]`
   - Remove `"M"` from first value: `mem_used_out` = Used memory (457)
   - Split the second part by spaces: `["616M", "(max.", "10240M)"]`
   - Remove `"M"` from first element: `mem_stack_out` = Stack memory (616)
   - Remove `"M)[||||...]"` from third element: `mem_max_out` = Maximum memory (10240)
   - Note: The REPLACE removes the `M)` and the visual bar chart that follows

**Example parsing:**
```
Input: "RAM: 457M/616M (max. 10240M)[|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||]"
After SPLIT(output, "RAM: "): ["", "457M/616M (max. 10240M)[||||...]"]
mem_section = "457M/616M (max. 10240M)[||||...]"
After SPLIT(mem_section, "/"): ["457M", "616M (max. 10240M)[||||...]"]
After REPLACE(mem_split2[0], "M", ""): "457"
mem_used_out = 457
After SPLIT(mem_stack_section, " "): ["616M", "(max.", "10240M)[||||...]"]
After REPLACE(mem_stack_split[0], "M", ""): "616"
After REPLACE(mem_stack_split[2], "M)[||||...]", ""): "10240"
mem_stack_out = 616
mem_max_out = 10240
```

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
mem_max=10240, mem_stack=616, mem_used=457, cpu_sys=0.14, cpu_proc=0.07, mspt_ave=0.08, mspt_min=0.04, mspt_max=0.18, tps=20.00, tps_one_min=20.00, tps_five_min=20.00, tps_fifteen_min=20.00
```

### Complete RCON Script

Here's the complete script (replace `<CODE>` with your RCON password and `<CODE_LEN>` with your password length):

```pseudo
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1              // REQUEST_ID (pick any positive int)
WRITE_INT 3              // AUTH
WRITE_STRING_LEN "<CODE>" "<CODE_LEN>"
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

# Parse TabTPS output
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
# Clean whitespace and parse (may need TRIM function)
FLOAT mspt_ave_out = mspt_array[0]
FLOAT mspt_min_out = mspt_array[1]
FLOAT mspt_max_out = mspt_array[2]

# Extract CPU: Split by "CPU: " then by ","
ARRAY cpu_split1 = SPLIT(output, "CPU: ")
STRING cpu_section = cpu_split1[1]
ARRAY cpu_split2 = SPLIT(cpu_section, ',')

# Remove % signs
STRING cpu_sys_str = REPLACE(cpu_split2[0], "%", "")
STRING cpu_proc_str = REPLACE(cpu_split2[1], "% (sys.", "")
FLOAT cpu_sys_out = cpu_sys_str
FLOAT cpu_proc_out = cpu_proc_str

# Extract Memory: Split by "RAM: " then by "/" then by " "
ARRAY mem_split1 = SPLIT(output, "RAM: ")
STRING mem_section = mem_split1[1]
ARRAY mem_split2 = SPLIT(mem_section, "/")
STRING mem_used_str = REPLACE(mem_split2[0], "M", "")  # "457M"
STRING mem_stack_section = mem_split2[1]  # "616M (max. 10240M)"
INT mem_used_out = mem_used_str

ARRAY mem_stack_split = SPLIT(mem_stack_section, ' ')
STRING mem_stack_str = REPLACE(mem_stack_split[0], 'M', '')  # "616M"
STRING mem_max_str = REPLACE(mem_stack_split[2], 'M)[|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||]', '')  # "10240M)"
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

## Source Engine Query Demo (Valheim)

This example shows how to:
- Send a challenge request to a Source Engine server
- Receive and extract the challenge value
- Send a query with the challenge
- Parse server information (name, map, players, etc.)

**Note:** This protocol is used by Valheim and other Source Engine-based games. The example works for Valheim servers!

### Understanding the Protocol

**Source Engine Protocol:**
- Uses UDP connection
- Requires a challenge-response mechanism to prevent DDoS attacks
- Two-step process: Challenge Request → Query Request
- Magic bytes: `0xFFFFFFFF` (4 bytes)
- Integers and shorts are little-endian (network byte order)

**Protocol Flow:**
1. **Challenge Request:** Send magic bytes + query type to get a challenge number
2. **Challenge Response:** Receive magic bytes + challenge number
3. **Query Request:** Send magic bytes + query type + challenge number
4. **Query Response:** Receive server information

**Games using Source Engine Query Protocol:**
- **Valheim** (uses this protocol!)
- Counter-Strike: Source
- Counter-Strike: Global Offensive
- Team Fortress 2
- Left 4 Dead
- Left 4 Dead 2
- Half-Life 2: Deathmatch
- And many more

### Step 1: Challenge Request Packet

First, we need to request a challenge from the server.

```pseudo
PACKET_START
WRITE_INT 0xFFFFFFFF
WRITE_BYTE 0x54
WRITE_STRING "Source Engine Query"
PACKET_END
```

**Breaking it down:**

1. **`WRITE_INT 0xFFFFFFFF`**
   - Magic bytes: `0xFFFFFFFF` (4 bytes)
   - This identifies the packet as a Source Engine query
   - Note: `WRITE_INT` is little-endian by default, but `0xFFFFFFFF` is the same value in both endianness, so it works correctly

2. **`WRITE_BYTE 0x54`**
   - Query type: `0x54` = `'T'` in ASCII
   - This is the challenge request type

3. **`WRITE_STRING "Source Engine Query"`**
   - Query string (null-terminated)
   - This is the standard query string for Source Engine

**What happens:**
- Server receives challenge request
- Server generates a random challenge number
- Server responds with the challenge

### Step 2: Parse Challenge Response

The server responds with a challenge number we need to use in the next request.

```pseudo
RESPONSE_START
EXPECT_MAGIC 0xFFFFFFFF
READ_BYTE response_type
READ_INT challenge
RESPONSE_END
```

**Breaking it down:**

1. **`EXPECT_MAGIC 0xFFFFFFFF`**
   - Validates the response starts with magic bytes
   - Raises an error if they don't match

2. **`READ_BYTE response_type`**
   - Response type (should be `0x41` = `'A'` for challenge response)
   - We read it but may not need to validate it

3. **`READ_INT challenge`**
   - The challenge number (4 bytes, little-endian)
   - **This is critical** - we'll use this in the next packet
   - Store this value - it will be needed for the query request

**What you'll get:**
- `challenge` variable contains a 32-bit integer (e.g., `1234567890`)
- This challenge must be included in the query request

### Step 3: Query Request Packet

Now we send the actual query with the challenge number.

```pseudo
PACKET_START
WRITE_INT 0xFFFFFFFF
WRITE_BYTE 0x54
WRITE_STRING "Source Engine Query"
WRITE_INT challenge
PACKET_END
```

**Breaking it down:**

1. **`WRITE_INT 0xFFFFFFFF`**
   - Magic bytes (same as before)

2. **`WRITE_BYTE 0x54`**
   - Query type: `'T'` (same as challenge request)

3. **`WRITE_STRING "Source Engine Query"`**
   - Query string (same as before)

4. **`WRITE_INT challenge`**
   - **The challenge number from step 2**
   - This proves we received the challenge response
   - Server will only respond if this matches

**Why the challenge?**
- Prevents DDoS attacks by requiring clients to complete a challenge-response
- Server can rate-limit challenge requests separately from queries
- Ensures the client is legitimate

### Step 4: Parse Query Response

The server responds with detailed server information.

```pseudo
RESPONSE_START
EXPECT_MAGIC 0xFFFFFFFF
READ_BYTE response_type
READ_BYTE protocol
READ_STRING_NULL server_name
READ_STRING_NULL map_name
READ_STRING_NULL folder
READ_STRING_NULL game
READ_SHORT app_id
READ_BYTE players
READ_BYTE max_players
READ_BYTE bots
READ_BYTE server_type
READ_BYTE environment
READ_BYTE visibility
READ_BYTE vac
READ_STRING_NULL version
RESPONSE_END
```

**Breaking it down:**

1. **`EXPECT_MAGIC 0xFFFFFFFF`**
   - Validates response starts with magic bytes

2. **`READ_BYTE response_type`**
   - Response type (should be `0x49` = `'I'` for info response)

3. **`READ_BYTE protocol`**
   - Protocol version number

4. **`READ_STRING_NULL server_name`**
   - Server name (null-terminated string)

5. **`READ_STRING_NULL map_name`**
   - Current map name

6. **`READ_STRING_NULL folder`**
   - Game folder (e.g., "cstrike", "tf", "left4dead")

7. **`READ_STRING_NULL game`**
   - Game name (e.g., "Counter-Strike: Source")

8. **`READ_SHORT app_id`**
   - Steam App ID (2 bytes, little-endian)

9. **`READ_BYTE players`**
   - Current player count

10. **`READ_BYTE max_players`**
    - Maximum players

11. **`READ_BYTE bots`**
    - Bot count

12. **`READ_BYTE server_type`**
    - Server type: `'d'` = dedicated, `'l'` = listen

13. **`READ_BYTE environment`**
    - Environment: `'w'` = Windows, `'l'` = Linux

14. **`READ_BYTE visibility`**
    - Visibility: `0` = public, `1` = private

15. **`READ_BYTE vac`**
    - VAC secured: `0` = no, `1` = yes

16. **`READ_STRING_NULL version`**
    - Server version string

**What you'll get:**
All these values are stored as variables and can be used in output formatting.

### Step 5: Format Output

```pseudo
OUTPUT_SUCCESS
RETURN "server_name_out=server_name, map_name_out=map_name, folder_out=folder, game_out=game, app_id_out=app_id, players_out=players, max_players_out=max_players, bots_out=bots, server_type_out=server_type, environment_out=environment, visibility_out=visibility, vac_out=vac, version_out=version"
OUTPUT_END

OUTPUT_ERROR
RETURN "server=HOST, error=ERROR"
OUTPUT_END
```

**Breaking it down:**

- **Success output:** Returns all extracted server information as key-value pairs
- **Error output:** Returns error information if something goes wrong

**Final output format:**
```
server_name_out=My CS:GO Server, map_name_out=de_dust2, folder_out=csgo, game_out=Counter-Strike: Global Offensive, app_id_out=730, players_out=12, max_players_out=24, bots_out=0, server_type_out=100, environment_out=119, visibility_out=0, vac_out=1, version_out=1.38.4.5
```

### Complete Source Engine Script

Here's the complete script:

```pseudo
PACKET_START
WRITE_INT 0xFFFFFFFF
WRITE_BYTE 0x54
WRITE_STRING "Source Engine Query"
PACKET_END

RESPONSE_START
EXPECT_MAGIC 0xFFFFFFFF
READ_BYTE response_type
READ_INT challenge
RESPONSE_END

PACKET_START
WRITE_INT 0xFFFFFFFF
WRITE_BYTE 0x54
WRITE_STRING "Source Engine Query"
WRITE_INT challenge
PACKET_END

RESPONSE_START
EXPECT_MAGIC 0xFFFFFFFF
READ_BYTE response_type
READ_BYTE protocol
READ_STRING_NULL server_name
READ_STRING_NULL map_name
READ_STRING_NULL folder
READ_STRING_NULL game
READ_SHORT app_id
READ_BYTE players
READ_BYTE max_players
READ_BYTE bots
READ_BYTE server_type
READ_BYTE environment
READ_BYTE visibility
READ_BYTE vac
READ_STRING_NULL version
RESPONSE_END

OUTPUT_SUCCESS
RETURN "server_name_out=server_name, map_name_out=map_name, folder_out=folder, game_out=game, app_id_out=app_id, players_out=players, max_players_out=max_players, bots_out=bots, server_type_out=server_type, environment_out=environment, visibility_out=visibility, vac_out=vac, version_out=version"
OUTPUT_END

OUTPUT_ERROR
RETURN "server=HOST, error=ERROR"
OUTPUT_END
```

### Key Points

1. **Challenge-Response Pattern:**
   - Always request challenge first
   - Use the challenge in the query request
   - Server validates the challenge before responding
   - Challenge must be used immediately (don't delay between packets)

2. **Magic Bytes:**
   - Always `0xFFFFFFFF` at the start of packets
   - Use `EXPECT_MAGIC` to validate responses
   - Magic bytes appear in both requests and responses

3. **Endianness:**
   - Integers are little-endian (default)
   - Shorts are little-endian (default)
   - Magic bytes (`0xFFFFFFFF`) work the same in both endianness

4. **Null-Terminated Strings:**
   - All strings are null-terminated
   - Use `READ_STRING_NULL` to read them (reads until `0x00`)
   - Use `WRITE_STRING` to write them (auto-adds null terminator)

5. **Byte Values:**
   - Server type: `'d'` (100) = dedicated, `'l'` (108) = listen
   - Environment: `'w'` (119) = Windows, `'l'` (108) = Linux
   - Visibility: `0` = public, `1` = private
   - VAC: `0` = no, `1` = yes (Valheim typically returns `0`)

### Valheim-Specific Notes

- **Port:** Default query port is `2457` (UDP)
- **Folder:** Typically `valheim`
- **Game:** Usually `valheim`
- **App ID:** Varies, but commonly used for Valheim
- **VAC:** Usually `0` (Valheim doesn't use VAC)

### Common Source Engine Games

| Game | Folder | App ID | Notes |
|------|--------|--------|-------|
| **Valheim** | valheim | varies | Uses Source Engine query protocol |
| Counter-Strike: Source | cstrike | 240 | |
| Counter-Strike: Global Offensive | csgo | 730 | |
| Team Fortress 2 | tf | 440 | |
| Left 4 Dead | left4dead | 500 | |
| Left 4 Dead 2 | left4dead2 | 550 | |
| Half-Life 2: Deathmatch | hl2mp | 320 | |

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

### Testing Source Engine Script

1. **Set up your Source Engine server:**
   - Ensure server is running
   - Note server address and port
   - Verify server allows query requests

2. **Configure in Net Sentinel:**
   - Protocol: `UDP`
   - Address: Server IP or hostname
   - Port: Server query port
     - Valheim: `2457` (default UDP query port)
     - Other Source Engine games: Usually the game port (e.g., 27015)

3. **Test the script:**
   - Use the test endpoint
   - Verify challenge request succeeds
   - Check challenge is received correctly
   - Confirm query request with challenge works
   - Verify all server information is parsed correctly

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

### Source Engine Issues

**Problem: Challenge request times out**
- **Solution:**
  - Check server is online and accessible
  - Verify port is correct (query port, not game port)
  - Check firewall allows UDP packets
  - Ensure server allows query requests (some servers disable them)

**Problem: Query request fails after challenge**
- **Solution:**
  - Verify challenge value is correctly read
  - Check challenge is included in query request
  - Ensure challenge hasn't expired (use it immediately)
  - Verify magic bytes are correct (`0xFFFFFFFF`)

**Problem: Response parsing fails**
- **Solution:**
  - Check `EXPECT_MAGIC` validates correctly
  - Verify response type matches expected (`0x49` for info)
  - Ensure all strings are read with `READ_STRING_NULL`
  - Check byte order (little-endian for integers/shorts)

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

