mod filter_env;
mod map_env;
mod replace_env;

use crate::filter_env::FilterConfig;
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

    #[arg(long, value_delimiter = ',', long_help = "Include exact ENV variables")]
    include_exact: Vec<String>,

    #[arg(
        long = "include",
        value_delimiter = ',',
        long_help = "Include regex ENV variables"
    )]
    include_regex: Vec<String>,

    #[arg(long, value_delimiter = ',', long_help = "Exclude exact ENV variables")]
    exclude_exact: Vec<String>,

    #[arg(
        long = "exclude",
        value_delimiter = ',',
        long_help = "Exclude regex ENV variables"
    )]
    exclude_regex: Vec<String>,
}

impl Args {
    /// Build FilterConfig from CLI args
    fn build_filter_config(&self) -> anyhow::Result<FilterConfig> {
        let builder = FilterConfig::builder();

        let builder = if !self.include_exact.is_empty() {
            builder.include_exact(self.include_exact.clone())
        } else {
            builder
        };

        let builder = if !self.include_regex.is_empty() {
            builder.include_regex(self.include_regex.clone())?
        } else {
            builder
        };

        let builder = if !self.exclude_exact.is_empty() {
            builder.exclude_exact(self.exclude_exact.clone())
        } else {
            builder
        };

        let builder = if !self.exclude_regex.is_empty() {
            builder.exclude_regex(self.exclude_regex.clone())?
        } else {
            builder
        };

        Ok(builder.build())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("Args: {:?}", args);
    let mapper = map_env::VITEnvMapper {};
    let filter_config = args.build_filter_config()?;
    mapper
        .map_env(
            args.dir,
            args.production_env_file,
            args.runtime_env_file,
            args.output_dir,
            args.suffixes,
            args.worker,
            Placeholder::from(args.placeholder),
            filter_config
        )
        .await
}
