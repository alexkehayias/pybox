mod sandbox;

use anyhow::Result;
use std::io::{self, Read};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: pybox2 [-] <code | stdin>");
        std::process::exit(-1);
    }

    // Read code from stdin if '-' is passed
    let code = if args.len() == 1 && args[0] == "-" {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input.trim().to_string()
    } else {
        args.last().unwrap().clone()
    };

    // Create sandbox and execute code
    let mut sandbox = sandbox::PySandbox::new(None)?;

    match sandbox.exec(&code) {
        Ok(result) => {
            // Guest stdout/stderr are untrusted side-channel output. Route them
            // to the host's stderr so the host's stdout carries only the result.
            if !result.stdout.is_empty() {
                eprint!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            println!("{}", result.value);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(-1);
        }
    }

    Ok(())
}
