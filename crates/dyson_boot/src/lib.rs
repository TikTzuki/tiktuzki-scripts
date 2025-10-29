//! # Dyson Boot
//!
//! A convenient Rust crate for quickly bootstrapping application configuration with minimal boilerplate.
//!
//! ## Features
//!
//! - Zero-boilerplate configuration loading with a simple macro
//! - Multi-source configuration (files + environment variables)
//! - Thread-safe singleton pattern with lazy initialization
//! - Type-safe deserialization using `serde`
//! - Environment variable override support
//!
//! ## Quick Example
//!
//! ```rust,no_run
//! use dyson_boot::settings_struct;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Deserialize, Serialize)]
//! struct AppConfig {
//!     pub host: String,
//!     pub port: u16,
//!     pub debug: bool,
//! }
//!
//! // Generate configuration loader with defaults
//! settings_struct!(AppConfig);
//!
//! fn main() {
//!     // Access configuration anywhere in your app
//!     let config = get_app_config();
//!     println!("Server running on {}:{}", config.host, config.port);
//! }
//! ```
//!
//! ## How It Works
//!
//! The `settings_struct!` macro generates:
//! 1. A `load()` method that reads configuration from files and environment variables
//! 2. A thread-safe static instance using `once_cell::Lazy`
//! 3. A getter function `get_{struct_name}()` that returns an `Arc<YourConfig>`
//!
//! The configuration is loaded lazily on first access and cached for the application lifetime.
//!
//! ## Configuration Priority
//!
//! Values are loaded in this order (later sources override earlier ones):
//! 1. Configuration file (JSON, TOML, YAML, etc.)
//! 2. Environment variables with the specified prefix
//!
//! See the [`settings_struct`] macro documentation for more details.

/// Settings module
// ///
// /// Contains helpers and types for loading and managing application configuration:
// /// - `settings_struct!` macro generated loader for typed config
// /// - Multi-source loading (files + environment) with override precedence
// /// - Thread-safe cached instance for global access
pub mod setting;
