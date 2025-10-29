/// Generates a configuration loader for a struct with file and environment variable support.
///
/// This macro creates a complete configuration loading system including:
/// - A `load()` method that reads from config files and environment variables
/// - A thread-safe static singleton instance
/// - A getter function to access the configuration from anywhere in your app
///
/// # Usage
///
/// ## Basic Usage (with defaults)
///
/// ```rust,no_run
/// use dyson_boot::settings_struct;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// struct AppConfig {
///     pub host: String,
///     pub port: u16,
/// }
///
/// // Uses default values:
/// // - Config dir: APP__CONFIG_DIR env var (defaults to ".")
/// // - Config file: app_config.json
/// // - Env prefix: APP
/// // - Env separator: __
/// // - List separator: ,
/// settings_struct!(AppConfig);
///
/// fn main() {
///     let config = get_app_config();
///     println!("Host: {}", config.host);
/// }
/// ```
///
/// ## Custom Configuration
///
/// ```rust,no_run
/// use dyson_boot::settings_struct;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// struct DatabaseConfig {
///     pub url: String,
///     pub max_connections: u32,
/// }
///
/// settings_struct!(
///     DatabaseConfig,
///     "DB_CONFIG_DIR",       // Environment variable for config directory
///     "database.json",       // Config file name
///     load_env = (
///         "DATABASE",        // Environment variable prefix
///         "__",              // Environment variable separator
///         ","
///     )                       // List separator for array values
/// )
///
/// fn main() {
///     let db_config = get_database_config();
///     println!("DB URL: {}", db_config.url);
/// }
/// ```
///
/// # Generated Code
///
/// For a struct named `AppConfig`, the macro generates:
/// - `impl AppConfig { fn load(config_path: PathBuf) -> anyhow::Result<Self> }`
/// - A static `APP_CONFIG: Lazy<Arc<AppConfig>>`
/// - A function `get_app_config() -> Arc<AppConfig>`
///
/// # Environment Variables
///
/// The macro supports overriding config values via environment variables:
///
/// ```bash
/// # Override top-level fields
/// export APP__host=0.0.0.0
/// export APP__port=3000
///
/// # Override nested fields (if using nested structs)
/// export APP__database__url=postgres://localhost/mydb
/// ```
///
/// # Configuration File Location
///
/// The config file path is determined by:
/// 1. Reading the directory from the specified environment variable (e.g., `APP__CONFIG_DIR`)
/// 2. If not set, defaults to current directory (`.`)
/// 3. Joins the directory with the config file name
///
/// # Panics
///
/// The generated code will panic and exit the process if:
/// - The configuration file cannot be found
/// - The configuration file contains invalid data
/// - Required fields are missing
///
/// # Requirements
///
/// Your configuration struct must:
/// - Implement `serde::Deserialize` and `serde::Serialize`
/// - Have all fields that can be deserialized from the config file format
///
/// # Examples
///
/// ## With Environment Variable Override
///
/// ```rust,no_run
/// use dyson_boot::settings_struct;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// struct ServerConfig {
///     pub bind_address: String,
///     pub port: u16,
///     pub workers: usize,
/// }
///
/// settings_struct!(
///     ServerConfig,
///     "SERVER_CONFIG_DIR",
///     "server.json",
///     load_env = ("SERVER",
///     "__",
///     ",")
/// );
///
/// fn main() {
///     // Set environment: export SERVER__port=9000
///     let config = get_server_config();
///     println!("Server will bind to {}:{}", config.bind_address, config.port);
/// }
/// ```
///
/// ## Multiple Configurations
///
/// You can use this macro multiple times for different config structs:
///
/// ```rust,no_run
/// use dyson_boot::settings_struct;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// struct AppConfig { pub name: String }
///
/// #[derive(Debug, Clone, Deserialize, Serialize)]
/// struct DbConfig { pub url: String }
///
/// settings_struct!(AppConfig);
/// settings_struct!(DbConfig, "DB_CONFIG_DIR", "db.json",load_env = ("DB", "__", ","));
///
/// fn main() {
///     let app = get_app_config();
///     let db = get_db_config();
///     println!("App: {}, DB: {}", app.name, db.url);
/// }
/// ```
#[macro_export]
macro_rules! settings_struct {

    ($name:ty) => {
      settings_struct!($name, "APP__CONFIG_DIR", "config.toml", load_env = ("APP", "__", ","));
    };

    (
        $name:ty,
        $CONFIG_DIR_VAR_NAME:expr,
        $config_file:expr,
        load_env = ($prefix:expr, $separator:expr, $list_separator:expr)
    ) => {
        impl $name {
            fn load(config_path: ::std::path::PathBuf) -> ::anyhow::Result<Self> {
                let name_str = stringify!($name);
                let path_str = config_path.to_string_lossy().to_string();
                if !config_path.exists() {
                    return Err(::anyhow::anyhow!("Configuration file '{}' not found", path_str));
                }

                let mut builder = config::Config::builder()
                    .add_source(config::File::from(config_path));
                if !$separator.is_empty() {
                    builder = builder.add_source(
                        config::Environment::with_prefix($prefix)
                            .separator($separator)
                            .list_separator($list_separator),
                    );
                }
                let settings = builder.build()?;

                let app_config: $name = settings.try_deserialize()?;
                println!(
                    "Loaded config {} from {}\n {}",
                    name_str,
                    path_str,
                    ::serde_json::to_string_pretty(&app_config)?
                );
                Ok(app_config)
            }
        }

        ::paste::paste! {
            static [<$name:snake:upper>] : ::once_cell::sync::Lazy<::std::sync::Arc<$name>> = ::once_cell::sync::Lazy::new(|| {
                let binding = std::env::var($CONFIG_DIR_VAR_NAME).unwrap_or_else(|_| ".".to_string());
                let config_dir = ::std::path::Path::new(&binding);
                let config_file = config_dir.join($config_file);
                match $name::load(config_file) {
                    Ok(cfg) => ::std::sync::Arc::new(cfg),
                    Err(e) => {
                        eprintln!("Failed to load config {}", e);
                        std::process::exit(1);
                    }
                }
            });
        }

        ::paste::paste! {
            pub fn [<get_ $name:snake:lower>]() -> ::std::sync::Arc<$name> {
                [<$name:snake:upper>].clone()
            }
        }
    };
}

