use std::env;
use std::process;
use oqab::cli::CommandLineParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments using the Command pattern
    match CommandLineParser::parse_args(&args) {
        Ok(command) => {
            // Execute the command and exit with its return code
            process::exit(command.execute());
        }
        Err(error_message) => {
            eprintln!("{}", error_message);
            process::exit(1);
        }
    }
}
