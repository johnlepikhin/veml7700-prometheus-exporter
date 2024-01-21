use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use slog::{o, Drain};
use slog_scope::error;
use structdoc::StructDoc;

mod config;
mod get;
mod run;

const CONFIG_DEFAULT_PATH: &str = "/etc/veml7700-prometheus-exporter.yaml";

// CLI subcommands
#[derive(Subcommand)]
enum CommandLine {
    /// Dump parsed config file. Helps to find typos
    DumpConfig,
    /// Print config file documentation
    ConfigDocumentation,
    /// Run exporter
    Run,
    /// Get current sensor value
    Get,
}

/// Example of simple cli program
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Application {
    /// Path to configuration file
    #[clap(short, default_value = CONFIG_DEFAULT_PATH)]
    config_path: String,
    /// Subcommand
    #[clap(subcommand)]
    command: CommandLine,
}

impl Application {
    fn init_syslog_logger(log_level: slog::Level) -> Result<()> {
        let logger = slog_syslog::SyslogBuilder::new()
            .facility(slog_syslog::Facility::LOG_USER)
            .level(log_level)
            .unix("/dev/log")
            .start()?;

        let logger = slog::Logger::root(logger.fuse(), o!());
        slog_scope::set_global_logger(logger).cancel_reset();
        Ok(())
    }

    fn init_env_logger() -> Result<()> {
        let drain =
            slog_term::CompactFormat::new(slog_term::TermDecorator::new().stderr().build()).build();
        // let drain = new(drain);
        let drain = std::sync::Mutex::new(drain.fuse());
        let logger = slog::Logger::root(drain.fuse(), o!());
        slog_scope::set_global_logger(logger).cancel_reset();
        Ok(())
    }

    fn init_logger(&self, config: &config::Config) -> Result<()> {
        if std::env::var("RUST_LOG").is_ok() {
            Self::init_env_logger()?
        } else {
            Self::init_syslog_logger(config.log_level.into())?
        }
        slog_stdlog::init()?;

        Ok(())
    }

    fn config_documentation() {
        println!(
            "Configuration file format. Default path is {}\n\n{}",
            CONFIG_DEFAULT_PATH,
            crate::config::Config::document()
        )
    }

    fn run_command(&self) -> Result<()> {
        match &self.command {
            CommandLine::DumpConfig => {
                let config = config::Config::read(&self.config_path).expect("Config");
                let config =
                    serde_yaml::to_string(&config).with_context(|| "Failed to dump config")?;
                println!("{}", config);
                Ok(())
            }
            CommandLine::ConfigDocumentation => {
                Self::config_documentation();
                Ok(())
            }
            CommandLine::Run => {
                let config = config::Config::read(&self.config_path).expect("Config");
                self.init_logger(&config).expect("Logger");

                run::Run::new(config)?.run()?;
                Ok(())
            }
            CommandLine::Get => {
                let config = config::Config::read(&self.config_path).expect("Config");
                self.init_logger(&config).expect("Logger");

                get::Get::get(config)?;
                Ok(())
            }
        }
    }

    pub fn run(&self) {
        if let Err(err) = self.run_command() {
            error!("Failed with error: {:#}", err);
        }
    }
}

fn main() {
    Application::parse().run();
}
