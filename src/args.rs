use std::fs::File;
use std::io::Read;

use crate::types::VcProveInput;

pub fn get_zk_task_name() -> String {
    let args: Vec<String> = std::env::args().collect();
    let default = "check_vc";
    args.get(1).map(|x| &**x).unwrap_or(default).to_string()
}

pub fn get_zk_task_input() -> Option<VcProveInput> {
    let args: Vec<String> = std::env::args().collect();
    let file_path = if let Some(file_path) = args.get(2) {
        file_path
    } else {
        return None;
    };

    // Attempt to open the file
    let mut file = File::open(file_path).unwrap_or_else(|e| {
        println!("Error: Failed to open file: {}", e);
        std::process::exit(1);
    });

    // Read the file contents into a string
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|e| {
        println!("Error: Failed to read file: {}", e);
        std::process::exit(1);
    });

    // Parse the JSON string into our Person struct
    let deserialized = serde_json::from_str::<VcProveInput>(&contents).unwrap_or_else(|e| {
        println!("Error: Failed to parse JSON: {}", e);
        std::process::exit(1);
    });

    Some(deserialized)
}
