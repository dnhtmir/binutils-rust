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
}
