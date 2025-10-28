mod map_env;

use crate::map_env::{EnvMapper, Placeholder};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "env-mapper",
    about = "Apply env-based replacements to JS/HTML files, example: https://github.com/TikTzuki/tiktuzki-scripts/tree/master/examples/env-mapper"
)]
struct Args {
    #[arg(short, long, default_value = "/js_source_code/dist")]
    dir: String,

    #[arg(
        short,
        long,
        long_help = "env value: VITE_API_URL=__VITE_API_URL__",
        default_value = ".env.production"
    )]
    production_env_file: String,

    #[arg(
        long,
        long_help = "Template place holder: 1. __KEY__ \n 2. {{KEY}} \n 3. ${KEY} \n 4. ${{KEY}}",
        default_value = "1"
    )]
    placeholder: u8,

    #[arg(
        short = 'e',
        long,
        long_help = "Runtime env value file to override current envs, default: None"
    )]
    runtime_env_file: Option<String>,

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
    mapper
        .map_env(
            args.dir,
            args.production_env_file,
            args.runtime_env_file,
            args.output_dir,
            args.suffixes,
            args.worker,
            Placeholder::from(args.placeholder),
        )
        .await
}
