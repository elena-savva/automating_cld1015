use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Duration;
use visa_rs::prelude::*;

use crate::cld1015::io_to_vs_err;

/// Performs a current sweep with the CLD1015 laser diode controller
/// and captures spectral data from the HP-70952B optical spectrum analyzer
pub fn run_current_sweep(
    cld1015: &mut Instrument,
    osa: &mut Instrument,
    start_ma: f64,
    stop_ma: f64,
    step_ma: f64,
    dwell_time_ms: u64,
) -> visa_rs::Result<()> {
    // Create a CSV file to save results
    let mut file = File::create("current_sweep_results.csv").unwrap();
    writeln!(file, "Current (mA),Peak Wavelength (nm),Peak Power (dBm)").unwrap();
    
    // Calculate number of points
    let num_points = ((stop_ma - start_ma) / step_ma).floor() as usize + 1;
    println!("Starting current sweep with {} points", num_points);
    
    // Configure the OSA for measurements
    osa.write_all(b"IP;\n").map_err(io_to_vs_err)?; // Instrument preset
    osa.write_all(b"SNGLS;\n").map_err(io_to_vs_err)?; // Set to single sweep mode
    
    // Turn laser OFF
    cld1015.write_all(b"OUTPut:STATe 0\n").map_err(io_to_vs_err)?;
    println!("Laser turned OFF");

    // Wait for initial stabilization
    std::thread::sleep(Duration::from_millis(500));
    
    // Turn laser ON
    cld1015.write_all(b"OUTPut:STATe 1\n").map_err(io_to_vs_err)?;
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
        cld1015.write_all(cmd.as_bytes()).map_err(io_to_vs_err)?;
        
        println!("Set current to {:.2} mA", current_ma);
        
        // Wait for stabilization
        std::thread::sleep(Duration::from_millis(dwell_time_ms));
        println!("Starting sweep");
        // Trigger a new sweep on the OSA and confirm it's done before proceeding
        osa.write_all(b"TS;DONE?;\n").map_err(io_to_vs_err)?; // Take sweep
        let mut done_resp = String::new();
        {
            let mut reader = BufReader::new(&*osa);
            reader.read_line(&mut done_resp).map_err(io_to_vs_err)?;
        }
        if done_resp.trim() != "1" {
            println!("Warning: Sweep not confirmed complete. Response: {}", done_resp.trim());
        } else {
            println!("Finished sweep");
        }
        
        
        // Find peak
        osa.write_all(b"MKPK HI;\n").map_err(io_to_vs_err)?; // Mark highest signal level
        
        // Get peak wavelength
        osa.write_all(b"MKWL?;\n").map_err(io_to_vs_err)?;
        let mut peak_wavelength = String::new();
        {
            let mut reader = BufReader::new(&*osa);
            reader.read_line(&mut peak_wavelength).map_err(io_to_vs_err)?;
        }
        let peak_wavelength_nm = peak_wavelength.trim().parse::<f64>().unwrap_or(0.0) * 1.0e9; // Convert from meters to nm
        
        // Get peak amplitude
        osa.write_all(b"MKA?;\n").map_err(io_to_vs_err)?;
        let mut peak_power = String::new();
        {
            let mut reader = BufReader::new(&*osa);
            reader.read_line(&mut peak_power).map_err(io_to_vs_err)?;
        }
        let peak_power_dbm = peak_power.trim().parse::<f64>().unwrap_or(-100.0);
        
        
        // Print measured values
        println!("  Peak Wavelength: {:.3} nm", peak_wavelength_nm);
        println!("  Peak Power: {:.2} dBm", peak_power_dbm);
        
        // Write to results file
        writeln!(file, "{:.2},{:.4},{:.2}", 
                current_ma, peak_wavelength_nm, peak_power_dbm).unwrap();
    }
    
    // Turn laser OFF
    cld1015.write_all(b"OUTPut:STATe 0\n").map_err(io_to_vs_err)?;
    println!("Laser turned OFF");

    osa.write_all(b"SWEEP OFF;\n").map_err(io_to_vs_err)?; // Turn off

    
    // Check for errors on CLD1015
    cld1015.write_all(b"SYST:ERR?\n").map_err(io_to_vs_err)?;
    
    let mut response = String::new();
    {
        // Create a new scope to ensure the BufReader is dropped before we use cld1015 again
        let mut reader = BufReader::new(&*cld1015);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    println!("Final error check on CLD1015: {}", response.trim());
    
    // Check for errors on OSA
    osa.write_all(b"XERR?;\n").map_err(io_to_vs_err)?;
    
    let mut response = String::new();
    {
        let mut reader = BufReader::new(&*osa);
        reader.read_line(&mut response).map_err(io_to_vs_err)?;
    }
    
    println!("Final error check on OSA: {}", response.trim());
    
    println!("Current sweep completed successfully");
    println!("Results saved to current_sweep_results.csv");
    
    Ok(())
}