use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command>", args[0]);
        println!("Available commands: next, color, quit");
        return Ok(());
    }
    
    let command = &args[1];
    let path = Path::new("/tmp/raifus_control");
    
    // Validate command
    match command.as_str() {
        "next" | "color" | "quit" => {
            // Write command to the control file
            let mut file = File::create(path)?;
            file.write_all(command.as_bytes())?;
            println!("Command '{}' sent to Raifus", command);
        },
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: next, color, quit");
        }
    }
    
    Ok(())
}
