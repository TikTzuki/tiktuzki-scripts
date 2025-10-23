# dyson_log

Plug-and-play logging for Rust services built on `tracing-subscriber` with:

- Daily-rotating JSON log files
- Pretty console logs during development
- Optional Tokio Console integration for async task debugging
- Controlled entirely by environment variables

## Install

Add this to your `Cargo.toml`:

```toml
[dependencies]
dyson_log = "0"
tracing = "0.1" # to use the `info!`, `warn!`, etc. macros
```

If you're in a workspace using this repo locally, you can depend on it by path while developing:

```toml
[dependencies]
dyson_log = { path = "../../crates/dyson_log" }
tracing = "0.1"
```

## Usage

Call `init_log()` once at startup and keep the returned guard alive for the entire process lifetime to ensure logs are flushed.

```rust
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (honors environment variables below)
    let _guard = dyson_log::init_log();

    info!("service starting");
    // ... your app ...
    warn!("service stopping");
    Ok(())
}
```

## Environment variables

- ENABLE_LOG: Enable/disable all logging
  - Default: `1`
  - Example: `ENABLE_LOG=0` to disable logging entirely
- LOG_PATH: Directory for log files
  - Default: `logs`
  - Example: `LOG_PATH=/var/log/myapp`
- LOG_FILE: Log filename (rotated daily in LOG_PATH)
  - Default: `app.log`
  - Example: `LOG_FILE=server.log`
- RUST_LOG: `tracing` filter
  - Examples: `RUST_LOG=info`, `RUST_LOG=debug,my_crate=trace,hyper=warn`
  - Uses `tracing_subscriber::EnvFilter::from_default_env()`
- TOKIO_CONSOLE_ENABLE: Enable Tokio Console subscriber
  - Default: disabled
  - Example: `TOKIO_CONSOLE_ENABLE=1`
  - Requires running the tokio-console app to view runtime metrics

## What you get

- Console logs: human-readable with targets and line numbers.
- File logs: structured JSON lines, with spans and line numbers, one file per day (daily rotation).

Example JSON line (shape will vary):

```json
{
  "timestamp":"2025-01-01T12:00:00Z",
  "level":"INFO",
  "target":"my_app::service",
  "message":"service starting",
  "span":{"name":"request","id":1},
  "line":42
}
```

## Tips

- Set `RUST_LOG` for production to reduce noise while keeping diagnostics when needed.
- Keep the returned `WorkerGuard` (from `init_log()`) in a variable to ensure file logs are flushed on shutdown.
- Combine with `tracing` spans for rich, contextual logs across async boundaries.

## License

Licensed under either of
- Apache License, Version 2.0
- MIT license

at your option.

