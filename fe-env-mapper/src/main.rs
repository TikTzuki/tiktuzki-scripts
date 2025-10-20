mod map_env;

use crate::map_env::EnvMapper;
use clap::Parser;
use std::io::{self};

#[derive(Parser, Debug)]
#[command(
    name = "env-mapper",
    about = "Apply env-based replacements to JS/HTML files"
)]
struct Args {
    #[arg(short, long, default_value = "/usr/share/nginx/html")]
    dir: String,
    #[arg(short, long, default_value = "pattern.env")]
    pattern_file: String,
    #[arg(short, long, default_value = "1")]
    worker: u8,
    #[arg(short, long, default_value = "1")]
    suffixes: Vec<String>,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("Args: {:?}", args);
    let mapper = map_env::VITEnvMapper {};
    mapper.map_env(args.dir, args.pattern_file, args.suffixes, args.worker)
}
