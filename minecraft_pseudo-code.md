# Minecraft Status Query Pseudo-code

This script mirrors the `probe_server` handshake + status request sequence from the Rust implementation documented in `gameserver_check.md`. It sends the two packets that Minecraft expects and then reads the JSON payload returned by the server. Use `HOSTNAME` and `PORT` placeholders (for example `10.0.2.27` and `26000`) when you build packets.

## Packet 1 – Handshake (example `10.0.2.27:26000`)

```
PACKET_START
WRITE_BYTE 0x00               # Packet ID for handshake
WRITE_BYTE 0x47               # Protocol version (71 in decimal)
WRITE_BYTE HOSTNAME_LEN       # Replace with actual hostname length
WRITE_STRING HOSTNAME         # Server host (e.g. 10.0.2.27)
WRITE_SHORT_BE PORT           # Server port (example 26000)
WRITE_BYTE 0x01               # Next state = 1 (status)
PACKET_END
```

## Packet 2 – Status Request

```
PACKET_START
WRITE_BYTE 0x01               # Packet length (calculated implicitly by builder)
WRITE_BYTE 0x00               # Packet ID for status request
PACKET_END
```

## Response Parsing

```
RESPONSE_START
READ_BYTE LENGTH_VARINT       # Minecraft prefixes responses with a varint length (skip/consume)
READ_BYTE PACKET_ID           # Expect 0x00 for status response
READ_STRING_NULL JSON_PAYLOAD # JSON string terminated by length varint (parse raw string)
RESPONSE_END
```

`JSON_PAYLOAD` should be parsed using the JSON parser configured for `McStatus` after the response is captured.

Error handling should follow the guidelines in `gameserver_check.md`: emit syntax, validation, or network errors if values deviate from expectations (wrong packet IDs, truncated JSON, timeouts, etc.).

## Concrete Example for `10.0.2.27:26000`

```
PACKET_START
WRITE_VARINT PACKET_LEN
WRITE_VARINT 0x00
WRITE_VARINT 0x47
WRITE_VARINT 0x09               # Length of "10.0.2.27"
WRITE_STRING_LEN "10.0.2.27" 9
WRITE_SHORT_BE 26000
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
RETURN "net_sentinel_gameserver_up{server='HOST', protocol=JSON_PAYLOAD.version.protocol} 1"
OUTPUT_END

OUTPUT_ERROR
RETURN "net_sentinel_gameserver_up{server='HOST', error=ERROR} 0"
OUTPUT_END
```

Use this example to test the interpreter by substituting the placeholder commands with the defined bytes for `10.0.2.27:26000`.

