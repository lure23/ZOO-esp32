/*
* Showcase reading the BOOT button.
*/
#![no_std]
#![no_main]

use anyhow::Result;

#[allow(unused_imports)]
use defmt::{info, error};
use defmt_rtt as _;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    prelude::*,
};

use semihosting::process;

mod common;
use common::init_defmt;

use devkit::BootButton;

#[entry]
fn main() -> ! {
    init_defmt();

    match main2() {
        Err(e) => panic!("Failed with: {:?}", e),
        Ok(()) => process::exit(0)      // back to developer's command line
    }
}

fn main2() -> Result<()> {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let bb = Button::new( Input::new(peripherals.GPIO9, Pull::Down) );  // BOOT button

    info!("Current state of BOOT button: {}", bb.xxx() );

    Ok(())
}
