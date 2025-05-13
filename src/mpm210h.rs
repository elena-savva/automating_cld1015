use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum PowerUnit {
    DBm,
    MW,
}

impl PowerUnit {
    pub fn to_command_value(&self) -> &'static str {
        match self {
            PowerUnit::DBm => "0",
            PowerUnit::MW => "1",
        }
    }
}

pub struct MPM210H {
    stream: TcpStream,
    module: u8,
    port: u8,
}

impl MPM210H {
    /// Connect to MPM210H power meter
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        
        // Set read timeout to prevent hanging
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        
        let mpm = MPM210H {
            stream,
            module: 0,
            port: 1,
        };
        
        // Check connection by querying device identity
        mpm.query("*IDN?")?;
        
        Ok(mpm)
    }
    
    /// Set the module number (0-4)
    pub fn set_module(&mut self, module: u8) -> io::Result<()> {
        if module > 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Module number must be between 0 and 4",
            ));
        }
        self.module = module;
        Ok(())
    }
    
    /// Set the port number (1-4)
    pub fn set_port(&mut self, port: u8) -> io::Result<()> {
        if port < 1 || port > 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Port number must be between 1 and 4",
            ));
        }
        self.port = port;
        Ok(())
    }
    
    /// Set the wavelength for calibration (1250-1630 nm)
    pub fn set_wavelength(&mut self, wavelength_nm: f64) -> io::Result<()> {
        if wavelength_nm < 1250.0 || wavelength_nm > 1630.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Wavelength must be between 1250 and 1630 nm",
            ));
        }
        
        // Set wavelength for the specific module and port
        let cmd = format!("DWAV {},{},{:.3}", self.module, self.port, wavelength_nm);
        self.write_command(&cmd)
    }
    
    /// Set the measurement mode
    pub fn set_measurement_mode(&mut self, mode: &str) -> io::Result<()> {
        let valid_modes = ["CONST1", "CONST2", "SWEEP1", "SWEEP2", "FREERUN"];
        if !valid_modes.contains(&mode) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid measurement mode: {}", mode),
            ));
        }
        
        self.write_command(&format!("WMOD {}", mode))
    }
    
    /// Set the averaging time in milliseconds
    pub fn set_averaging_time(&mut self, time_ms: f64) -> io::Result<()> {
        if time_ms < 0.01 || time_ms > 10000.0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Averaging time must be between 0.01 and 10000 ms",
            ));
        }
        
        self.write_command(&format!("AVG {:.2}", time_ms))
    }
    
    /// Set the power unit (dBm or mW)
    pub fn set_power_unit(&mut self, unit: PowerUnit) -> io::Result<()> {
        self.write_command(&format!("UNIT {}", unit.to_command_value()))
    }
    
    /// Perform zeroing to calibrate the power meter
    pub fn zero(&mut self) -> io::Result<()> {
        println!("Performing zero calibration...");
        self.write_command("ZERO")?;
        
        // Zero command takes about 3 seconds to complete
        std::thread::sleep(Duration::from_secs(3));
        Ok(())
    }
    
    /// Read the optical power from the specified module and port
    pub fn read_power(&mut self) -> io::Result<f64> {
        // Use READ? command to get the power reading
        let response = self.query(&format!("READ? {}", self.module))?;
        
        // The response format is "power1,power2,power3,power4"
        // We need to parse out the power for our specific port
        let powers: Vec<&str> = response.trim().split(',').collect();
        
        if powers.len() < self.port as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid response: {}", response),
            ));
        }
        
        // Parse the power value from the string
        match powers[(self.port - 1) as usize].parse::<f64>() {
            Ok(val) => Ok(val),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse power value: {}", e),
            )),
        }
    }
    
    /// Write a command to the device
    fn write_command(&mut self, cmd: &str) -> io::Result<()> {
        // Add LF termination and write the command
        let cmd_with_term = format!("{}\n", cmd);
        self.stream.write_all(cmd_with_term.as_bytes())?;
        
        // MPM-210H needs a 10ms delay after each command
        std::thread::sleep(Duration::from_millis(10));
        
        Ok(())
    }
    
    /// Send a query and read the response
    fn query(&self, cmd: &str) -> io::Result<String> {
        let mut s = self.stream.try_clone()?;
        
        // Add LF termination and write the command
        let cmd_with_term = format!("{}\n", cmd);
        s.write_all(cmd_with_term.as_bytes())?;
        
        // MPM-210H needs a 10ms delay after each command
        std::thread::sleep(Duration::from_millis(10));
        
        // Read the response
        let mut reader = BufReader::new(s);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        
        Ok(response)
    }
    
    /// Check for any errors
    pub fn check_errors(&self) -> io::Result<String> {
        self.query("ERR?")
    }
}