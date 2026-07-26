//! Reads configuration and connects to the configured SML source, then
//! dispatches to either the daemon (publishes to MQTT) or the debug
//! subcommand (prints raw SML values to stdout).

use crate::cli::{Args, Commands};
use crate::config::{Config, Connection, get_config_path};
use crate::error::WattwolfError;
use clap::Parser;
use serialport::{Parity, StopBits};
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;
use tracing::info;

mod cli;
mod config;
mod daemon;
mod debug;
mod error;
mod model;
mod mqtt;
mod obis;

fn main() -> Result<(), WattwolfError> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let config = args.config;
    let config = get_config_path(config).ok_or(WattwolfError::ConfigNotFound)?;

    let config = Config::from_file(&config)?;

    info!(?config, "Loaded configuration");

    // Create reader based on connection type
    let reader: Box<dyn Read> = match &config.connection {
        Connection::Socket(addr) => {
            info!(address = %addr, "Connecting to TCP socket");
            Box::new(TcpStream::connect(addr).map_err(WattwolfError::ConnectionError)?)
        }
        Connection::Device(device) => {
            info!(device = %device, "Opening serial port");
            let port = serialport::new(device, 9_600)
                .stop_bits(StopBits::One)
                .parity(Parity::None)
                .timeout(Duration::from_secs(5))
                .open()
                .map_err(WattwolfError::SerialPortError)?;
            Box::new(port)
        }
    };

    match args.command {
        Some(Commands::Debug) => debug::run(reader),
        None => daemon::run(&config, reader),
    }
}
