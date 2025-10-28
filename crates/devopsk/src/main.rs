//! DevOps Toolkit (`devopsk`)
//!
//! Small command-line toolkit with helpers for common DevOps tasks.
//!
//! Current features
//! ----------------
//! - `wg` subcommand: WireGuard helper to create, revoke and list clients.
//!
//! Quick usage
//! -----------
//! Build and run the binary, then invoke the `wg` subcommand:
//! ```sh
//! cargo run -p devopsk -- wg --wg-cmd "new-client alice"
//! cargo run -p devopsk -- wg --wg-cmd "revoke-client alice"
//! cargo run -p devopsk -- wg --wg-cmd "list-clients"
//! ```
//!
//! Examples
//! --------
//! - Create a client named `alice`:
//!   `cargo run -p devopsk -- wg --wg-cmd "new-client alice"`
//! - Revoke `alice`:
//!   `cargo run -p devopsk -- wg --wg-cmd "revoke-client alice"`
//!
//! Generating documentation
//! ------------------------
//! Generate and open HTML docs with:
//! ```sh
//! cargo doc --open
//! ```
//!
//! Contributing
//! ------------
//! Contributions welcome. Add new subcommands and update this top-level crate doc to explain them.


use clap::{Parser, Subcommand};
use tokio;

#[derive(Parser, Debug)]
#[command(name = "devopsk", about = "DevOps Toolkit")]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Wg {
        #[arg(long_help = "WireGuard command string. \n 1. new-client <client_name> \n 2. revoke-client <client_name> \n 3. list-clients")]
        wg_cmd: String
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.cmd {
        Command::Wg { wg_cmd: cmd } => {
        }
    }

    Ok(())
}
