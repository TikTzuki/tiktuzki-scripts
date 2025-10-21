mod map_env;

use crate::map_env::EnvMapper;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "env-mapper",
    about = "Apply env-based replacements to JS/HTML files"
)]
struct Args {
    #[arg(short, long, default_value = "/js_source_code/dist")]
    dir: String,

    #[arg(
        short,
        long,
        long_help = "env value: VITE_API_URL=\\${VITE_API_URL}",
        default_value = ".env.production"
    )]
    production_env_file: String,

    #[arg(
        short = 'e',
        long,
        long_help = "Dynamic env value file to override current envs, default: None"
    )]
    dynamic_env_file: Option<String>,

    #[arg(short, long, value_delimiter = ',', default_value = "js,html")]
    suffixes: Vec<String>,

    #[arg(
        short = 'o',
        long,
        long_help = "Output directory for processed files, default: overwrite"
    )]
    output_dir: Option<String>,

    #[arg(
        short,
        long,
        default_value = "1",
        long_help = "Number of parallel workers"
    )]
    worker: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("Args: {:?}", args);
    let mapper = map_env::VITEnvMapper {};
    mapper.map_env(
        args.dir,
        args.production_env_file,
        args.dynamic_env_file,
        args.output_dir,
        args.suffixes,
        args.worker,
    ).await
}
