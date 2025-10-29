//! Integration tests for the `settings_struct!` macro
//!
//! This module tests the configuration loading functionality provided by the
//! `settings_struct!` macro, including both file-based and environment variable-based
//! configuration sources.

#[cfg(test)]
mod tests {
    use dyson_boot::settings_struct;
    use envtestkit::lock::lock_test;
    use envtestkit::set_env;
    use serde::{Deserialize, Serialize};
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::{env, fs};
    use tempfile::TempDir;

    /// Creates a test configuration file in the given temporary directory
    ///
    /// # Arguments
    ///
    /// * `dir` - The temporary directory where the config file will be created
    /// * `content` - The JSON content to write to the config file
    ///
    /// # Returns
    ///
    /// The path to the created configuration file
    fn create_test_config_file(dir: &TempDir, file_name: &str, content: &str) -> PathBuf {
        let config_path = dir.path().join(file_name);
        fs::write(&config_path, content).unwrap();
        config_path
    }

    /// Tests the `settings_struct!` macro with default configuration loading
    ///
    /// This test verifies that:
    /// - Configuration can be loaded from a TOML file using default settings
    /// - The macro works without explicit `load_env` parameter
    /// - File-based configuration is correctly parsed and accessible
    ///
    /// Configuration file:
    /// - Format: TOML (`config.toml`)
    /// - Contains `id` and `value` fields
    ///
    /// Environment variables used:
    /// - `APP__CONFIG_DIR`: Directory containing the configuration file
    #[test]
    fn test_setting_struct_macro_with_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_json = r#"
        id = "test-123"
        value = 100
        "#;
        create_test_config_file(&temp_dir, "config.toml", config_json);

        let _lock = lock_test(); // Ensures exclusive access to environment during this test
        let _test_env = set_env("APP__CONFIG_DIR".into(), temp_dir.path().to_str().unwrap());

        #[derive(Debug, Serialize, Deserialize, Clone, Default)]
        struct TestConfig {
            pub id: String,
            pub value: i32,
        }

        settings_struct!(TestConfig);

        let config = get_test_config();
        assert_eq!(config.id, "test-123");
        assert_eq!(config.value, 100);
    }

    /// Tests the `settings_struct!` macro with environment variable loading enabled
    ///
    /// This test verifies that:
    /// - Configuration can be loaded from environment variables
    /// - The environment variable prefix ("TEST"), separator ("__"), and list separator (",") work correctly
    /// - Values from environment variables are loaded when configuration file is empty
    ///
    /// Configuration file:
    /// - Format: JSON (`config.json`)
    /// - Contains empty object `{}`
    ///
    /// Environment variables used:
    /// - `APP__CONFIG_DIR`: Directory containing the configuration file
    /// - `TEST__ID`: Test ID value (maps to `id` field)
    /// - `TEST__VALUE`: Test numeric value (maps to `value` field)
    #[test]
    fn test_setting_struct_macro_with_env() {
        let temp_dir = TempDir::new().unwrap();
        create_test_config_file(&temp_dir, "config.json", r#"{}"#);

        let _lock = lock_test(); // Ensures exclusive access to environment during this test
        let _test_env = set_env("APP__CONFIG_DIR".into(), temp_dir.path().to_str().unwrap());
        let _test_env = set_env("TEST__ID".into(), "test-123");
        let _test_env = set_env("TEST__VALUE".into(), "100");

        #[derive(Debug, Serialize, Deserialize, Clone, Default)]
        struct TestConfig {
            pub id: String,
            pub value: i32,
        }
        settings_struct!(
            TestConfig,
            "APP__CONFIG_DIR",
            "config.json",
            load_env = ("TEST", "__", ",")
        );

        let config = get_test_config();
        assert_eq!(config.id, "test-123");
        assert_eq!(config.value, 100);
    }

    /// Tests the `settings_struct!` macro without environment variable loading
    ///
    /// This test verifies that:
    /// - Configuration is loaded only from the configuration file
    /// - Environment variables are ignored when prefix is empty ("")
    /// - File-based configuration is used regardless of environment variable presence
    ///
    /// Configuration file:
    /// - Format: JSON (`config_without_env.json`)
    /// - Contains `id` ("test-123") and `value` (100)
    ///
    /// Environment variables set (but ignored due to empty prefix):
    /// - `APP__CONFIG_DIR`: Directory containing the configuration file
    /// - `TEST__ID`: Set to "unexpected" but should be ignored
    #[test]
    fn test_setting_struct_macro_without_env() {
        let temp_dir = TempDir::new().unwrap();
        let config_json = r#"{"id": "test-123", "value": 100}"#;
        create_test_config_file(&temp_dir, "config_without_env.json", config_json);

        let _lock = lock_test(); // Ensures exclusive access to environment during this test
        let _test_env = set_env("APP__CONFIG_DIR".into(), temp_dir.path().to_str().unwrap());
        let _test_env = set_env("TEST__ID".into(), "unexpected");

        #[derive(Debug, Serialize, Deserialize, Clone, Default)]
        struct TestConfig {
            pub id: String,
            pub value: i32,
        }

        settings_struct!(
            TestConfig,
            "APP__CONFIG_DIR",
            "config_without_env.json",
            load_env = ("", "", "")
        );

        let config = get_test_config();
        assert_eq!(config.id, "test-123");
        assert_eq!(config.value, 100);
    }
}
