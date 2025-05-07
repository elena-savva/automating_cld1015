#![allow(unused)]
mod cld1015;
mod experiment;

use std::ffi::CString;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Duration;
use visa_rs::prelude::*;

use cld1015::io_to_vs_err;

fn main() -> visa_rs::Result<()> {
    // Initialize the VISA resource manager
    let rm = DefaultRM::new()?;
    
    // Define the VISA resource string for your instrument
    let resource_string = CString::new("USB::4883::32847::M01053290::0::INSTR").unwrap();
    
    // Open a session to the resource
    let mut instr = rm.open(
        &resource_string.into(),
        AccessMode::NO_LOCK,
        Duration::from_secs(1),
    )?;
    
    // Send the *CLS command to the instrument to clear errors
    instr.write_all(b"*CLS\n").map_err(io_to_vs_err)?;
    
    // Send the *IDN? command to the instrument to verify connection
    instr.write_all(b"*IDN?\n").map_err(io_to_vs_err)?;
    
    // Read the response from the instrument
    let mut response = String::new();
    {
        // Create a new scope to ensure the BufReader is dropped before we use instr again
        let mut reader = BufReader::new(&instr);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    // Print the response
    println!("Instrument Response: {}", response);
    
    // Set the device to operate in Constant Current mode
    instr.write_all(b"SOURce:FUNCtion:MODE CURRent\n").map_err(io_to_vs_err)?;
    
    // Configure and run the current sweep
    let start_ma = 0.0;     // Start at 0 mA
    let stop_ma = 100.0;    // End at 100 mA
    let step_ma = 5.0;      // 5 mA steps
    let dwell_time_ms = 50; // 50ms stabilization delay
    
    experiment::run_current_sweep(&mut instr, start_ma, stop_ma, step_ma, dwell_time_ms)?;
    
    Ok(())
}