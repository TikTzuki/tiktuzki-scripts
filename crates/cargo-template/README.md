# cargo-template

A command-line tool for applying predefined templates to Rust Cargo projects. This tool helps standardize workspace and
package configurations by rendering template configurations into existing `Cargo.toml` files.

## Features

- **Template-based Configuration**: Apply predefined templates to workspace and package `Cargo.toml` files
- **Workspace Inheritance**: Configures packages to inherit settings from workspace using `workspace = true`
- **Metadata Standardization**: Ensures consistent package metadata across projects

## Installation

### From Source

```bash
cargo install --path .
```

### As Part of Workspace

Add to your workspace's `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/cargo-template",
]
```

## Usage

### Basic Usage

Apply a template to the current directory:

```bash
cargo-template render-template <template-name>
```

Apply a template to a specific directory:

```bash
cargo-template render-template <template-name> --target /path/to/project
```

### Available Templates

The tool includes several built-in templates:

- **`workspace`**: Basic workspace configuration with metadata and common settings
- **`axum`**: Template for Axum web framework projects
- **`sqlx_postgres`**: Template for SQLx PostgreSQL projects

### Examples

#### Apply workspace template to current directory

```bash
cargo-template render-template workspace
```

#### Apply axum template to a specific package

```bash
cd my-workspace/crates/api
cargo-template render-template axum
```

## How It Works

### For Workspace Cargo.toml

When applied to a workspace `Cargo.toml`, the tool:

1. Updates `[workspace.package]` metadata (version, edition, authors, license, etc.)
2. Retrieves the Git repository URL and sets it automatically
3. Adds or updates `[workspace.dependencies]` from the template
4. Preserves existing settings not specified in the template

### For Package Cargo.toml

When applied to a package `Cargo.toml`, the tool:

1. Updates `[package]` metadata (name, description)
2. Configures package to inherit workspace settings using `workspace = true`:
    - `version.workspace = true`
    - `edition.workspace = true`
    - `authors.workspace = true`
    - etc.
3. Adds dependencies with `workspace = true` flag

## Template Structure

Templates are TOML files located in the `templates/` directory with the following structure:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.90"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
homepage = "https://example.com"
description = "My awesome project"

[workspace.dependencies]
tokio = { version = "1.42", features = ["full"] }
anyhow = "1.0"
# ... more dependencies
```

## Creating Custom Templates

1. Create a new directory in `templates/`:
   ```bash
   mkdir templates/my-template
   ```

2. Create a `Cargo.toml` with your desired configuration:
   ```bash
   touch templates/my-template/Cargo.toml
   ```

3. Define your template structure following the format above

4. Rebuild the tool (templates are embedded at compile time):
   ```bash
   cargo build --release
   ```

5. Use your template:
   ```bash
   cargo-template render-template my-template
   ```

### Key Traits

#### `CargoManager`

The core trait for managing Cargo.toml files:

```rust
pub trait CargoManager {
    fn format_workspace_package(&mut self, table: &Table) -> Result<()>;
    fn get_dependency(&self, name: &Key) -> Result<&Item>;
    fn remove_dependency(&mut self, name: &Key) -> Result<()>;
    fn update_dependency(&mut self, name: &Key, spec: Option<&Item>) -> Result<()>;
    fn file(&self) -> &PathBuf;
    fn doc(&self) -> &DocumentMut;
    fn commit(&self) -> Result<()>;
}
```

## Dependencies

- **`clap`**: Command-line argument parsing
- **`anyhow`**: Error handling
- **`toml_edit`**: TOML file manipulation while preserving formatting
- **`include_dir`**: Embedding template files at compile time

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

## License

This project inherits the license from the parent workspace.

## Contributing

Contributions are welcome! Please ensure:

1. Code is properly documented
2. Tests pass
3. Code follows Rust best practices
4. Commit messages are clear and descriptive
