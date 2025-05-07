use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Duration;
use visa_rs::prelude::*;

use crate::cld1015::io_to_vs_err;

/// Performs a current sweep with the CLD1015 laser diode controller
pub fn run_current_sweep(
    instr: &mut Instrument,
    start_ma: f64,
    stop_ma: f64,
    step_ma: f64,
    dwell_time_ms: u64,
) -> visa_rs::Result<()> {
    // Create a CSV file to save results
    let mut file = File::create("current_sweep_results.csv").unwrap();
    writeln!(file, "Current (mA)").unwrap();
    
    // Calculate number of points
    let num_points = ((stop_ma - start_ma) / step_ma).floor() as usize + 1;
    println!("Starting current sweep with {} points", num_points);
    
    // Initial current to 0 for safety
    let cmd = format!("SOURce:CURRent:LEVel:IMMediate:AMPLitude 0.0\n");
    instr.write_all(cmd.as_bytes()).map_err(io_to_vs_err)?;
    
    // Turn laser ON
    instr.write_all(b"OUTPut:STATe 1\n").map_err(io_to_vs_err)?;
    println!("Laser turned ON");
    
    // Wait for initial stabilization
    std::thread::sleep(Duration::from_millis(500));
    
    // Perform the sweep
    for i in 0..num_points {
        let current_ma = start_ma + (i as f64 * step_ma);
        
        // Convert mA to A for the device
        let current_a = current_ma / 1000.0;
        
        // Set the current
        let cmd = format!("SOURce:CURRent:LEVel:IMMediate:AMPLitude {:.6}\n", current_a);
        instr.write_all(cmd.as_bytes()).map_err(io_to_vs_err)?;
        
        println!("Set current to {:.2} mA", current_ma);
        
        // Wait for stabilization
        std::thread::sleep(Duration::from_millis(dwell_time_ms));
        
        // Write to results file
        writeln!(file, "{:.2}", current_ma).unwrap();
    }
    
    // Turn laser OFF
    instr.write_all(b"OUTPut:STATe 0\n").map_err(io_to_vs_err)?;
    println!("Laser turned OFF");
    
    // Check for errors
    instr.write_all(b"SYST:ERR?\n").map_err(io_to_vs_err)?;
    
    let mut response = String::new();
    {
        // Create a new scope to ensure the BufReader is dropped before we use instr again
        let mut reader = BufReader::new(&*instr);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    println!("Final error check: {}", response.trim());
    
    println!("Current sweep completed successfully");
    println!("Results saved to current_sweep_results.csv");
    
    Ok(())
}