# RCON TabTPS Output Parsing Guide

This guide shows how to parse TabTPS plugin output from Minecraft RCON to extract TPS, MSPT, CPU, and memory statistics.

## Overview

The TabTPS plugin provides detailed server performance metrics via the `mspt` command. This document shows how to:
1. Authenticate with RCON
2. Execute the `mspt` command
3. Parse the output to extract specific metrics

## Sample Output

The `mspt` command returns output like:
```
[TabTPS] Server Tick InformationTPS: 20.00 (5s), 20.00 (1m), 20.00 (5m), 20.00 (15m)MSPT - Average, Minimum, Maximum ├─ 5s - 0.05, 0.02, 0.09 ├─ 10s - 0.06, 0.02, 0.09 └─ 60s - 0.11, 0.01, 20.29CPU: 0.21%, 0.13% (sys., proc.)RAM: 928M/1120M (max. 10240M)[||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||]
```

## Target Values to Extract

From the output above, we want to extract:
- **tps**: `20.00` (from "TPS: 20.00 (5s)")
- **mspt_ave**: `0.05` (from "├─ 5s - 0.05, 0.02, 0.09")
- **mspt_min**: `0.02` (from "├─ 5s - 0.05, 0.02, 0.09")
- **mspt_max**: `0.09` (from "├─ 5s - 0.05, 0.02, 0.09")
- **cpu**: `[0.21, 0.13]` (from "CPU: 0.21%, 0.13%")
- **mem_used**: `928` (from "RAM: 928M/1120M")
- **mem_stack**: `1120` (from "RAM: 928M/1120M")
- **mem_max**: `10240` (from "max. 10240M")

## Complete Pseudo-Code

```pseudo
# RCON Authentication
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1              # REQUEST_ID
WRITE_INT 3              # AUTH
WRITE_STRING_LEN "<PASSCODE>" 32
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END

RESPONSE_START
READ_INT RESPONSE_LEN_AUTH
READ_INT RESPONSE_ID_AUTH
READ_INT RESPONSE_TYPE_AUTH
READ_STRING_NULL COMMAND_OUT_AUTH
RESPONSE_END

# Execute mspt command
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

# Parse the TabTPS output
CODE_START
# Extract TPS (from "TPS: 20.00 (5s)")
STRING tps_raw = COMMAND_OUT
# Find "TPS: " and extract the number before " (5s)"
# This would require string manipulation functions

# Extract MSPT values (from "├─ 5s - 0.05, 0.02, 0.09")
# Find "├─ 5s - " and extract the three comma-separated values
STRING mspt_line = COMMAND_OUT
# Parse: "├─ 5s - 0.05, 0.02, 0.09"
# Split by comma after "├─ 5s - " to get [0.05, 0.02, 0.09]
ARRAY mspt_values = SPLIT(mspt_line, ',')
FLOAT mspt_ave = mspt_values[0]
FLOAT mspt_min = mspt_values[1]
FLOAT mspt_max = mspt_values[2]

# Extract CPU values (from "CPU: 0.21%, 0.13%")
STRING cpu_raw = COMMAND_OUT
# Find "CPU: " and extract "0.21%, 0.13%"
# Split by comma and remove % signs
ARRAY cpu_parts = SPLIT(cpu_raw, ',')
FLOAT cpu_sys = cpu_parts[0]  # "0.21%"
FLOAT cpu_proc = cpu_parts[1]  # "0.13%"
ARRAY cpu = [cpu_sys, cpu_proc]

# Extract Memory values (from "RAM: 928M/1120M (max. 10240M)")
STRING mem_raw = COMMAND_OUT
# Find "RAM: " and extract "928M/1120M (max. 10240M)"
# Split by "/" to get ["928M", "1120M (max. 10240M)"]
ARRAY mem_parts = SPLIT(mem_raw, '/')
INT mem_used = mem_parts[0]  # "928M" -> 928
# Split mem_parts[1] by " " to get ["1120M", "(max.", "10240M)"]
ARRAY mem_stack_parts = SPLIT(mem_parts[1], ' ')
INT mem_stack = mem_stack_parts[0]  # "1120M" -> 1120
INT mem_max = mem_stack_parts[2]  # "10240M)" -> 10240
CODE_END
```

## Parsing Strategy

### Step 1: Extract TPS

The TPS value appears as `TPS: 20.00 (5s)`. We need to:
1. Find the position of "TPS: "
2. Extract the number before " (5s)"

**Note**: This requires string manipulation functions that may need to be added (like `SUBSTRING`, `FIND`, or regex support).

### Step 2: Extract MSPT Values

The MSPT line format is: `├─ 5s - 0.05, 0.02, 0.09`

1. Find the line containing "├─ 5s - "
2. Extract everything after "├─ 5s - "
3. Split by comma to get the three values
4. Parse each value as a float

### Step 3: Extract CPU Values

The CPU line format is: `CPU: 0.21%, 0.13%`

1. Find the position of "CPU: "
2. Extract "0.21%, 0.13%"
3. Split by comma
4. Remove "%" from each value
5. Parse as floats

### Step 4: Extract Memory Values

The RAM line format is: `RAM: 928M/1120M (max. 10240M)`

1. Find the position of "RAM: "
2. Extract "928M/1120M (max. 10240M)"
3. Split by "/" to separate used and stack
4. Parse "928M" to get `mem_used = 928`
5. Split the second part by space to get stack and max
6. Parse "1120M" to get `mem_stack = 1120`
7. Parse "10240M" to get `mem_max = 10240`

## Implementation Notes

### Current Limitations

The current pseudo-code system supports:
- ✅ `SPLIT()` function for splitting strings
- ✅ `REPLACE()` function for string replacement
- ✅ Variable declarations and assignments
- ❌ String position finding (SUBSTRING, FIND, INDEX_OF)
- ❌ Regular expressions
- ❌ Number extraction from strings

### Workarounds

Until additional string functions are added, you may need to:

1. **Use SPLIT strategically**: Split by known delimiters to isolate sections
2. **Use REPLACE to clean**: Remove unwanted characters before parsing
3. **Chain operations**: Use multiple SPLIT operations to narrow down to the target value

### Example: Extracting TPS (Workaround)

```pseudo
CODE_START
STRING output = COMMAND_OUT

# Step 1: Split by "TPS: " to get the part after TPS
ARRAY tps_parts = SPLIT(output, 'TPS: ')
# tps_parts[1] should contain "20.00 (5s), 20.00 (1m)..."

# Step 2: Split by " (" to get just the number
STRING tps_with_space = tps_parts[1]
ARRAY tps_number_parts = SPLIT(tps_with_space, ' (')
# tps_number_parts[0] should be "20.00"

# Step 3: Convert to float (may need additional parsing)
FLOAT tps = tps_number_parts[0]
CODE_END
```

### Example: Extracting MSPT Values

```pseudo
CODE_START
STRING output = COMMAND_OUT

# Step 1: Find the 5s MSPT line
ARRAY lines = SPLIT(output, '├─')
# Find the line containing "5s - "
STRING mspt_line = ""
FOR line IN lines:
  IF line CONTAINS "5s - ":
    mspt_line = line
    BREAK

# Step 2: Extract values after "5s - "
ARRAY mspt_sections = SPLIT(mspt_line, '5s - ')
STRING mspt_values_str = mspt_sections[1]
# mspt_values_str should be "0.05, 0.02, 0.09"

# Step 3: Split by comma
ARRAY mspt_array = SPLIT(mspt_values_str, ',')
# mspt_array = ["0.05", " 0.02", " 0.09"]

# Step 4: Clean and parse (remove spaces, convert to float)
# Note: This may require additional string functions
FLOAT mspt_ave = mspt_array[0]
FLOAT mspt_min = mspt_array[1]  # May need to trim whitespace
FLOAT mspt_max = mspt_array[2]  # May need to trim whitespace
CODE_END
```

## Recommended String Functions to Add

To make parsing easier, consider adding these functions:

1. **`FIND(str, substr)`** - Returns the index of a substring
2. **`SUBSTRING(str, start, length)`** - Extracts a portion of a string
3. **`TRIM(str)`** - Removes leading/trailing whitespace
4. **`EXTRACT_NUMBER(str)`** - Extracts the first number from a string
5. **`MATCH(str, pattern)`** - Regular expression matching

## Alternative: Server-Side Parsing

If the parsing becomes too complex with current functions, consider:

1. **Modify the TabTPS plugin** to output JSON format
2. **Use a different command** that returns structured data
3. **Add a custom parsing endpoint** on the server side

## Complete Example with Current Functions

Here's a more complete example using only SPLIT and REPLACE:

```pseudo
# RCON Authentication
PACKET_START
WRITE_INT PACKET_LEN
WRITE_INT 1
WRITE_INT 3
WRITE_STRING_LEN "<PASSCODE>" 32
WRITE_BYTE 0
WRITE_BYTE 0
PACKET_END

RESPONSE_START
READ_INT RESPONSE_LEN_AUTH
READ_INT RESPONSE_ID_AUTH
READ_INT RESPONSE_TYPE_AUTH
READ_STRING_NULL COMMAND_OUT_AUTH
RESPONSE_END

# Execute mspt command
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
ARRAY tps_split1 = SPLIT(output, 'TPS: ')
STRING tps_section = tps_split1[1]
ARRAY tps_split2 = SPLIT(tps_section, ' (')
STRING tps_str = tps_split2[0]
FLOAT tps = tps_str

# Extract MSPT: Find "├─ 5s - " line
ARRAY mspt_lines = SPLIT(output, '├─')
STRING mspt_line = ""
FOR line IN mspt_lines:
  IF line CONTAINS "5s - ":
    mspt_line = line
    BREAK

# Extract MSPT values from the line
ARRAY mspt_split1 = SPLIT(mspt_line, '5s - ')
STRING mspt_values_str = mspt_split1[1]
ARRAY mspt_array = SPLIT(mspt_values_str, ',')
# Clean whitespace and parse (may need TRIM function)
FLOAT mspt_ave = mspt_array[0]
FLOAT mspt_min = mspt_array[1]
FLOAT mspt_max = mspt_array[2]

# Extract CPU: Split by "CPU: " then by ","
ARRAY cpu_split1 = SPLIT(output, 'CPU: ')
STRING cpu_section = cpu_split1[1]
ARRAY cpu_split2 = SPLIT(cpu_section, ',')
# Remove % signs
STRING cpu_sys_str = cpu_split2[0]
STRING cpu_proc_str = cpu_split2[1]
REPLACE(cpu_sys_str, '%', '')
REPLACE(cpu_proc_str, '%', '')
FLOAT cpu_sys = cpu_sys_str
FLOAT cpu_proc = cpu_proc_str
ARRAY cpu = [cpu_sys, cpu_proc]

# Extract Memory: Split by "RAM: " then by "/" then by " "
ARRAY mem_split1 = SPLIT(output, 'RAM: ')
STRING mem_section = mem_split1[1]
ARRAY mem_split2 = SPLIT(mem_section, '/')
STRING mem_used_str = mem_split2[0]  # "928M"
STRING mem_stack_section = mem_split2[1]  # "1120M (max. 10240M)"
REPLACE(mem_used_str, 'M', '')
INT mem_used = mem_used_str

ARRAY mem_stack_split = SPLIT(mem_stack_section, ' ')
STRING mem_stack_str = mem_stack_split[0]  # "1120M"
STRING mem_max_str = mem_stack_split[2]  # "10240M)"
REPLACE(mem_stack_str, 'M', '')
REPLACE(mem_max_str, 'M)', '')
INT mem_stack = mem_stack_str
INT mem_max = mem_max_str
CODE_END
```

## Expected Variables After Parsing

After executing the code above, you should have these variables:

- `tps`: `20.00`
- `mspt_ave`: `0.05`
- `mspt_min`: `0.02`
- `mspt_max`: `0.09`
- `cpu`: `[0.21, 0.13]`
- `mem_used`: `928`
- `mem_stack`: `1120`
- `mem_max`: `10240`

## Testing

To test the parsing:

1. Use the RCON authentication and command execution pseudo-code
2. Verify `COMMAND_OUT` contains the TabTPS output
3. Execute the CODE block to parse the values
4. Check the Variables section in the UI to see the extracted values

## Troubleshooting

### "Variable not found" errors
- Ensure the CODE block is executed after the RESPONSE_END
- Check that `COMMAND_OUT` was successfully read

### Parsing incorrect values
- Verify the TabTPS output format matches the expected format
- Check for whitespace issues (may need TRIM function)
- Verify SPLIT delimiters match the actual output format

### Array index out of bounds
- The output format may have changed
- Add validation to check array length before accessing indices
- Use IF statements to handle missing data gracefully

## Future Enhancements

Consider adding these features to make parsing easier:

1. **Regular expression support**: `MATCH(str, pattern)` to extract values
2. **Number extraction**: `EXTRACT_NUMBER(str)` to find and parse numbers
3. **String trimming**: `TRIM(str)` to remove whitespace
4. **Substring extraction**: `SUBSTRING(str, start, end)` for precise extraction
5. **Type conversion helpers**: Automatic conversion from string to number when assigning

## References

- [RCON Protocol Guide](./rcon.md) - RCON connection and authentication
- [Pseudo-Code Documentation](./pseudo-code-docs.md) - Complete pseudo-code reference
- [TabTPS Plugin](https://github.com/pl3xgaming/Pl3xMap) - Minecraft plugin documentation

