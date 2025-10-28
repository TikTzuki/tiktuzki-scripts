# Dyson Boot

A convenient Rust crate for quickly bootstrapping application configuration with minimal boilerplate.

## Features

- 🚀 **Zero-boilerplate configuration loading** - Generate configuration loaders with a simple macro
- 🔄 **Multi-source configuration** - Automatically loads from config files and environment variables
- 🔒 **Thread-safe singleton pattern** - Lazily-initialized, globally accessible configuration
- 📝 **Type-safe** - Leverages Rust's type system with `serde` for deserialization
- 🌍 **Environment variable override** - Override any config value via environment variables

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dyson_boot = "0.1"
config = "0.14"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

## Quick Start

### Basic Usage

```rust
use dyson_boot::settings_struct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AppConfig {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

// Generate configuration loader with defaults
settings_struct!(AppConfig);

fn main() {
    // Access configuration anywhere in your app
    let config = get_app_config();
    println!("Server running on {}:{}", config.host, config.port);
}
```

Create a `config.json` file:

```json
{
  "host": "localhost",
  "port": 8080,
  "debug": true
}
```

### Custom Configuration

You can customize the configuration source:

```rust
settings_struct!(
    AppConfig,
    "MY_CONFIG_DIR",      // Environment variable for config directory
    "app_config.json",    // Config file name
    "APP",                // Environment variable prefix
    "__",                 // Environment variable separator
    ","                   // List separator
);
```

### Environment Variable Override

Override any config value using environment variables with the specified prefix and separator:

```bash
# Override host and port
export APP__host=0.0.0.0
export APP__port=3000

# Override nested config (if you have nested structs)
export APP__database__url=postgres://localhost/mydb
```

## How It Works

The `settings_struct!` macro generates:

1. A `load()` method that reads configuration from files and environment variables
2. A thread-safe static instance using `once_cell::Lazy`
3. A getter function `get_{struct_name}()` that returns an `Arc<YourConfig>`

The configuration is loaded lazily on first access and cached for the lifetime of the application.

## Configuration Priority

Configuration values are loaded in the following order (later sources override earlier ones):

1. Configuration file (JSON, TOML, YAML, etc.)
2. Environment variables with the specified prefix

## Examples

### With Custom Environment Variables

```rust
use dyson_boot::settings_struct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

settings_struct!(
    DatabaseConfig,
    "DB_CONFIG_DIR",
    "database.json",
    "DATABASE",
    "__",
    ","
);

fn main() {
    let db_config = get_database_config();
    println!("Connecting to: {}", db_config.url);
}
```

Set environment variables:

```bash
export DB_CONFIG_DIR=/etc/myapp
export DATABASE__url=postgres://prod-server/mydb
export DATABASE__max_connections=100
```

## Testing

The crate includes testing utilities. See the `tests/` directory for examples:

```rust
use tempfile::TempDir;
use std::{env, fs};

#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_json = r#"{"host": "test", "port": 9000}"#;
    
    fs::write(temp_dir.path().join("test.json"), config_json).unwrap();
    env::set_var("TEST_CONFIG_DIR", temp_dir.path());
    
    settings_struct!(TestConfig, "TEST_CONFIG_DIR", "test.json", "TEST", "__", ",");
    let config = get_test_config();
    
    assert_eq!(config.host, "test");
    assert_eq!(config.port, 9000);
}
```

## Requirements

- Rust 1.70 or later
- Your config struct must implement `Deserialize` and `Serialize` from `serde`

## License

Licensed under the same terms as the parent workspace.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

