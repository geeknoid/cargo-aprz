use super::config::Config;
use crate::Result;
use crate::expr::evaluate;
use crate::metrics::default_metrics;
use camino::Utf8PathBuf;
use chrono::Local;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct ValidateArgs {
    /// Path to configuration file (default is `rank.toml`)
    #[arg(long, short = 'c', value_name = "PATH")]
    pub config: Option<Utf8PathBuf>,
}

#[expect(clippy::unnecessary_wraps, reason = "Consistent interface with other subcommands")]
pub fn validate_config(args: &ValidateArgs) -> Result<()> {
    let workspace_root = Utf8PathBuf::from(".");
    let config_path = args.config.as_ref();

    match Config::load(&workspace_root, config_path) {
        Ok(config) => {
            // Validate that all expressions can be evaluated against default metrics
            let metrics: Vec<_> = default_metrics().collect();

            if let Err(e) = evaluate(
                &config.deny_if_any,
                &config.accept_if_any,
                &config.accept_if_all,
                &metrics,
                Local::now(),
            ) {
                eprintln!("❌ Configuration validation failed: {e}");
                std::process::exit(1);
            }

            println!("Configuration file is valid");
            if let Some(path) = config_path {
                println!("Config file: {path}");
            } else {
                println!("Using default configuration (no config file found)");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Configuration validation failed: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init::{InitArgs, init_config};

    #[test]
    fn test_default_config_is_valid() {
        // Create a temporary file path for the test
        let temp_dir = std::env::temp_dir();
        let config_path =
            Utf8PathBuf::from(temp_dir.to_string_lossy().to_string()).join(format!("test_config_{}.toml", std::process::id()));

        // Step 1: Generate default configuration using init_config
        let init_args = InitArgs {
            output: config_path.clone(),
        };
        let init_result = init_config(&init_args);

        // Clean up on any error during init
        if init_result.is_err() {
            let _ = std::fs::remove_file(&config_path);
            panic!("init_config should succeed: {:?}", init_result.err());
        }

        // Step 2: Load the generated configuration
        let workspace_root = Utf8PathBuf::from(".");
        let config_result = Config::load(&workspace_root, Some(&config_path));

        // Clean up the file
        let _ = std::fs::remove_file(&config_path);

        let config = config_result.expect("Config::load should succeed for generated default config");

        // Step 3: Validate that expressions can be evaluated against default metrics
        let metrics: Vec<_> = default_metrics().collect();
        let result = evaluate(
            &config.deny_if_any,
            &config.accept_if_any,
            &config.accept_if_all,
            &metrics,
            Local::now(),
        );

        assert!(
            result.is_ok(),
            "Default configuration expressions should evaluate successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_default_config_matches_embedded() {
        // Verify that Config::default() produces the same config as parsing DEFAULT_CONFIG_TOML
        let default_config = Config::default();
        let parsed_config: Config =
            toml::from_str(super::super::config::DEFAULT_CONFIG_TOML).expect("DEFAULT_CONFIG_TOML should parse successfully");

        // Compare the serialized forms to ensure they're equivalent
        let default_toml = toml::to_string(&default_config).expect("default config should serialize");
        let parsed_toml = toml::to_string(&parsed_config).expect("parsed config should serialize");

        assert_eq!(
            default_toml, parsed_toml,
            "Config::default() should match parsing DEFAULT_CONFIG_TOML"
        );
    }
}
