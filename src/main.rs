use std::collections::BTreeMap; // For ordered map
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::task;

const FILE_CHUNK_SIZE: usize = 4096;
const SPLIT_CHUNK_SIZE: usize = 16;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} hexdump <filename>", args[0]);
        return;
    }

    let command: &String = &args[1];
    match command.as_str() {
        "hexdump" => hexdump(&args[2]).await,
        _ => eprintln!("unknown command: {}", command),
    }
}

async fn hexdump(filename: &String) {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error opening file {}: {}", filename, err);
            return;
        }
    };

    // Shared map for storing results (ordered by offset using BTreeMap)
    let map = Arc::new(Mutex::new(BTreeMap::new()));

    // Read the file in chunks
    let mut buffer: [u8; FILE_CHUNK_SIZE] = [0u8; FILE_CHUNK_SIZE];
    let mut file: File = file;
    let mut offset: usize = 0;

    println!("Hexdump of file {}:", filename);

    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        let map = Arc::clone(&map);
        let chunk = buffer[..bytes_read].to_vec();
        let offset_start = offset;

        // Spawn a worker to process the chunk asynchronously
        // TODO: improve worker orchestration
        // TODO: make benchmark tests
        // TODO: accept arguments for chunk sizes
        // TODO: implement other functions
        task::spawn(async move {
            process_chunk(map, offset_start, &chunk).await;
        });

        offset += bytes_read;
    }

    // Wait for all workers to finish
    tokio::task::yield_now().await;

    // Lock the map and print its ordered contents
    let map = map.lock().unwrap();
    for (offset, (hex_dump, ascii_dump)) in map.iter() {
        println!("{:08x}: {} | {}", offset, hex_dump, ascii_dump);
    }
}

async fn process_chunk(
    map: Arc<Mutex<BTreeMap<usize, (String, String)>>>,
    offset_start: usize,
    chunk: &[u8],
) {
    for (i, small_chunk) in chunk.chunks(SPLIT_CHUNK_SIZE).enumerate() {
        let offset = offset_start + (i * 8); // Calculate the offset for the small chunk
        process_small_chunks(Arc::clone(&map), offset, small_chunk).await;
    }
}

async fn process_small_chunks(
    map: Arc<Mutex<BTreeMap<usize, (String, String)>>>,
    offset_start: usize,
    chunk: &[u8],
) {
    let mut hex_dump: String = String::new();
    let mut ascii_dump: String = String::new();

    for (i, byte) in chunk.iter().enumerate() {
        // Append hex representation
        hex_dump.push_str(&format!("{:02x} ", byte));

        // Append ASCII representation (printable or '.')
        if byte.is_ascii_graphic() || *byte == b' ' {
            ascii_dump.push(*byte as char);
        } else {
            ascii_dump.push('.');
        }

        // Add extra space for alignment after 8 bytes
        if (i + 1) % 8 == 0 {
            hex_dump.push_str(" ");
        }
    }

    // Lock the map and insert the processed chunk
    let mut map = map.lock().unwrap();
    map.insert(offset_start, (hex_dump, ascii_dump));
}
