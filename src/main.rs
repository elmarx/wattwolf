//! Reads data from a serial port and prints the contained sml messages to stdout

use crate::model::Measurement;
use anyhow::Context;
use serialport::{Parity, StopBits};
use sml_rs::parser::complete::File;
use std::time::Duration;

mod model;

fn main() -> anyhow::Result<()> {
    let ports = serialport::available_ports().context("No ports found!")?;

    let port_name = "/dev/ttyUSB0";

    println!("Connecting to port {}", port_name);
    let port = serialport::new(port_name, 9_600)
        .stop_bits(StopBits::One)
        .parity(Parity::None)
        .timeout(Duration::from_millis(5000))
        .open()
        .context("Failed to open port")?;

    let mut reader = sml_rs::SmlReader::from_reader(port);

    let res = reader.next::<File>().context("could not read from port")?;
    let res = res?;
    let measurement: Measurement = res.try_into()?;
    println!("{:#?}", measurement);

    Ok(())
}
