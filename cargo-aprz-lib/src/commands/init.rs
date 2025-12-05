use super::config::Config;
use crate::Result;
use camino::Utf8PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Output configuration file path
    #[arg(value_name = "PATH", default_value = "aprz.toml")]
    pub output: Utf8PathBuf,
}

pub fn init_config(args: &InitArgs) -> Result<()> {
    Config::save_default(&args.output)?;
    println!("Generated default configuration file: {}", args.output);
    Ok(())
}
