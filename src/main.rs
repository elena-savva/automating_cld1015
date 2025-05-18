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
    
    // Define the VISA resource string for CLD1015
    let cld1015_resource = CString::new("USB::4883::32847::M01053290::0::INSTR").unwrap();
    
    // Define the VISA resource string for the HP-70952B OSA
    let osa_resource = CString::new("GPIB0::23::INSTR").unwrap();
    
    // Open a session to the CLD1015
    let mut cld1015 = rm.open(
        &cld1015_resource.into(),
        AccessMode::NO_LOCK,
        Duration::from_secs(1),
    )?;
    
    // Open a session to the OSA
    let mut osa = rm.open(
        &osa_resource.into(),
        AccessMode::NO_LOCK,
        Duration::from_secs(1),
    )?;
    
    // Send the *CLS command to the CLD1015 to clear errors
    cld1015.write_all(b"*CLS\n").map_err(io_to_vs_err)?;
    
    // Clear the OSA and perform instrument preset
    osa.write_all(b"CLS;IP;\n").map_err(io_to_vs_err)?;
    
    // Send the *IDN? command to verify CLD1015 connection
    cld1015.write_all(b"*IDN?\n").map_err(io_to_vs_err)?;
    
    // Read the response from the CLD1015
    let mut response = String::new();
    {
        // Create a new scope to ensure the BufReader is dropped before we use cld1015 again
        let mut reader = BufReader::new(&cld1015);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    // Print the CLD1015 response
    println!("CLD1015 Response: {}", response);

    // Check for errors on CLD1015
    cld1015.write_all(b"SYST:ERR?\n").map_err(io_to_vs_err)?;
    
    let mut response = String::new();
    {
        // Create a new scope to ensure the BufReader is dropped before we use cld1015 again
        let mut reader = BufReader::new(&cld1015);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    println!("Initial error check on CLD1015: {}", response.trim());
    
    // Check the OSA identity
    osa.write_all(b"ID?;\n").map_err(io_to_vs_err)?;
    
    let mut response = String::new();
    {
        let mut reader = BufReader::new(&osa);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    // Print the OSA response
    println!("Optical Spectrum Analyzer Response: {}", response);
    
    // Set the CLD1015 to operate in Constant Current mode
    cld1015.write_all(b"SOURce:FUNCtion:MODE CURRent\n").map_err(io_to_vs_err)?;

    // Set current limit to a safe value
    cld1015.write_all(b"SOURce:CURRent:LIMit:AMPLitude 100MA\n").map_err(io_to_vs_err)?;
    
    // Configure and run the current sweep
    let start_ma = 0.0;     // Start at 0 mA
    let stop_ma = 100.0;    // End at 100 mA
    let step_ma = 0.1;      // 1 mA steps
    let dwell_time_ms = 50; // 100ms stabilization delay
    
    experiment::run_current_sweep(&mut cld1015, &mut osa, start_ma, stop_ma, step_ma, dwell_time_ms)?;
    
    Ok(())
}