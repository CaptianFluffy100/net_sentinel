This project is licensed under the GNU General Public License v3.0 (GPLv3).
See the LICENSE file for details.


# Net Sentinel

A network monitoring service built with Rust and Axum that tracks ISP connectivity, website availability, and game server status using custom pseudo-code scripts.

## Features

- **ISP Monitoring**: Check internet connectivity by testing multiple ISP endpoints
- **Website Monitoring**: Monitor website availability with external and direct IP connectivity options
- **Game Server Monitoring**: Monitor any game server using custom pseudo-code scripts (supports TCP/UDP protocols)
- **Prometheus Metrics**: `/metrics` endpoint for integration with monitoring systems
- **Web UI**: Clean interface at `/` for managing all monitored resources
- **JSON File Storage**: Simple persistent storage for configuration
- **REST API**: Full CRUD operations for ISPs, websites, and game servers

## What It Does

✅ **Internet Connectivity Monitoring**
- Tests multiple ISP IP addresses to determine if internet is up
- Reports connectivity status as Prometheus metrics

✅ **Website Monitoring**
- Checks website availability via normal HTTP/HTTPS requests
- Supports direct IP connectivity (bypassing DNS)
- Tracks both external and direct connection status

✅ **Game Server Monitoring**
- Supports any game server protocol via pseudo-code scripts
- TCP and UDP protocol support
- Custom packet construction and response parsing
- Extracts server metrics (players, version, performance, etc.)
- Supports complex protocols like RCON, Minecraft, Source Engine, and more

✅ **Prometheus Integration**
- Exports all metrics in Prometheus format
- Includes labels for detailed filtering and aggregation

## What It Doesn't Do

❌ **Not a full monitoring solution** - Focuses on network/server availability, not system metrics
❌ **No alerting** - Only provides metrics, doesn't send notifications
❌ **No historical data** - Metrics are current state only (use Prometheus for history)
❌ **No authentication** - API endpoints are unprotected (use reverse proxy for production)

## Current Status

**Fully Working:**
- ✅ ISP IP monitoring (add, view, delete)
- ✅ Website monitoring (add, view, delete, external/direct checks)
- ✅ Game server monitoring (add, view, delete, test)
- ✅ Pseudo-code script execution (packet building, response parsing, code blocks)
- ✅ Prometheus metrics endpoint
- ✅ Web UI for configuration
- ✅ REST API for all resources

**Pseudo-Code Features:**
- ✅ Packet construction (WRITE_* commands)
- ✅ Response parsing (READ_* commands)
- ✅ Variable declarations and manipulation
- ✅ String functions (SPLIT, REPLACE)
- ✅ Control flow (IF statements, FOR loops)
- ✅ JSON parsing and nested field access
- ✅ Output formatting for Prometheus metrics
- ✅ Multiple packet/response pairs
- ✅ Connection management (TCP/UDP)

## Running

```bash
cargo run
```

The server will start on `http://localhost:3100`

## API Endpoints

### Web Interface
- `GET /` - Web UI for managing ISPs, websites, and game servers

### Metrics
- `GET /metrics` - Prometheus metrics endpoint

### ISP Management
- `GET /api/isps` - List all ISP IPs
- `POST /api/isps` - Create a new ISP IP
- `DELETE /api/isps/:id` - Delete an ISP IP

### Website Management
- `GET /api/websites` - List all websites
- `POST /api/websites` - Create a new website
- `DELETE /api/websites/:id` - Delete a website

### Game Server Management
- `GET /api/gameservers` - List all game servers
- `POST /api/gameservers` - Create a new game server
- `POST /api/gameservers/test` - Test a game server configuration (without saving)
- `DELETE /api/gameservers/:id` - Delete a game server
- `POST /api/gameservers/:id/test` - Test an existing game server

## Storage

The application uses JSON file storage and creates a `net_sentinel.json` file automatically in the current working directory on first run. This file contains all configuration for ISPs, websites, and game servers.

## Documentation

Comprehensive documentation is available in the `doc/` directory:

- **[Overview](doc/00-overview.md)** - What Net Sentinel is and how it works
- **[Beginner's Guide](doc/01-beginners-guide.md)** - Learn to write pseudo-code scripts
- **[Syntax Reference](doc/02-pseudo-code-syntax.md)** - Complete command reference
- **[Examples](doc/03-examples.md)** - Real-world protocol examples
- **[How It Works](doc/04-how-it-works.md)** - Internal implementation details
- **[Hands-On Demo](doc/05-hands-on-demo.md)** - Complete walkthrough of RCON and Minecraft examples

## Example: Monitoring a Minecraft Server

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
```

See the [documentation](doc/README.md) for more examples and tutorials.

