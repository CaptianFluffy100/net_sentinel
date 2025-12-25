# Net Sentinel

A network monitoring service built with Rust and Axum that tracks ISP IPs, websites, and game servers.

## Features

- **Prometheus Metrics**: `/metrics` endpoint for monitoring
- **Web UI**: Clean dark mode interface at `/` for managing ISP IPs
- **JSON File Storage**: Simple persistent storage for configuration
- **REST API**: Full CRUD operations for ISP management

## Current Status

Currently supports:
- ✅ ISP IP monitoring (add, view, delete)
- ✅ Prometheus metrics endpoint
- ✅ Web UI for configuration

Planned:
- 🔲 Website monitoring
- 🔲 Game server monitoring (Minecraft, etc.)
- 🔲 Health checks and status reporting

## Running

```bash
cargo run
```

The server will start on `http://localhost:3000`

## API Endpoints

- `GET /` - Web UI for managing ISPs
- `GET /metrics` - Prometheus metrics
- `GET /api/isps` - List all ISP IPs
- `POST /api/isps` - Create a new ISP IP
- `DELETE /api/isps/:id` - Delete an ISP IP

## Storage

The application uses JSON file storage and creates a `net_sentinel.json` file automatically in the current working directory on first run.

