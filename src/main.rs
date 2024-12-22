use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("Arguments {:?}", args);

    let program_name: &String = &args[0];
    println!("Program name: {}", program_name);

    if args.len() > 1 {
        for arg in &args[1..] {
            println!("Argument: {}", arg);
        }
    } else {
        println!("No additional arguments provided.");
    }

    if args.len() < 3 {
        return;
    }

    let command: &String = &args[1];
    match command.as_str() {
        "hexdump" => hexdump(&args[2]),
        _ => eprintln!("unknown command: {}", command),
    }
}

fn hexdump(filename: &String) {
    let mut file: File = match File::open(filename) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("error opening file {}: {}", filename, err);
            return;
        }
    };

    let mut buffer: [u8; 2] = [0u8, 255];
    let mut offset: usize = 0;
    let mut map = HashMap::new();

    println!("hexdump of file {}:", filename);
    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        for chunk in buffer[..bytes_read].chunks(16) {
            let offset_str = format!("{:08x}", offset - offset % 8);

            let value = map
                .entry(offset_str.clone())
                .or_insert(("".to_string(), "".to_string()));

            for byte in &buffer[..bytes_read] {
                value.0.push_str(&format!("{:02x}  ", byte));
            }

            for byte in chunk {
                if byte.is_ascii_graphic() || *byte == b' ' {
                    value.1.push_str(&format!("{}", *byte as char));
                } else {
                    value.1.push_str(".");
                }
            }

            offset += chunk.len();

            if offset % 8 == 0 {
                println!("{}: {} | {}", offset_str, value.0, value.1);
            }
        }
    }
}
