//! This module is responsible for keeping the program in state
//! between API calls. The `Program` struct holds the cells in a
//! hashmap
use crate::handlers::CodeRequest;
use std::collections::HashMap;
use std::env;
use std::fs::write;

/// This is what sits in state as long as the program is running
/// when a new request is made, it will check if the same cell has
/// been executed by checking `cells` which is a `HashMap`
///
/// # Terms
/// - temp_dir: Whatever the OS temp directory is
/// - filename: Last executed filename so we can restart state if necessary
/// - cells: VS Code name for a notebook cell

#[derive(Debug)]
pub struct Program {
    pub temp_dir: String,
    pub filename: String,
    pub cells: HashMap<u32, Cell>,
}

/// A cell represent a notebook cell in VS Code
///
/// # Terms
/// - fragment: unique id of the cell that executed
/// - index: current index by order in VS Code
/// - contents: the source code from inside the cell
/// used to determine what output to return to caller
#[derive(Debug)]
pub struct Cell {
    pub fragment: u32,
    pub index: u32,
    pub contents: String,
}

impl Program {
    /// Create a new program which is retained in state
    /// between `http` requests
    pub fn new() -> Program {
        let mut temp_file = env::temp_dir()
            .to_str()
            .expect("Error getting temp directory")
            .to_string();
        temp_file = temp_file;

        println!("Ctrl+Click to view rust file: {}/main.rs", temp_file);
        println!("Ctrl+Click to view cargo file: {}/Cargo.toml", temp_file);
        Program {
            temp_dir: temp_file,
            filename: String::new(),
            cells: HashMap::new(),
        }
    }

    /// Initial cell creation if it doesn't exist yet
    /// Contents Use clone as we need to retain
    /// a pointer to data that will stick around after the request has finished
    pub fn create_cell(&mut self, cr: &CodeRequest) {
        let cell = Cell {
            contents: cr.contents.clone(),
            fragment: cr.fragment,
            index: cr.index,
        };
        self.cells.insert(cr.fragment, cell);
    }

    /// As the cells are implemented with a HashMap this is a fast lookup,
    /// Only updates what's required
    pub fn update_cell(&mut self, cr: &CodeRequest) {
        if let Some(cell) = self.cells.get_mut(&cr.fragment) {
            cell.contents = cr.contents.clone();
            cell.index = cr.index;
        }
    }

    /// First sorts the hashmap by index into a vector, then writes the output with
    /// `rustkernel-start` and `rustkernel-end` to determine which cell's was executing
    // so we return only that output.
    pub fn write_to_file(&mut self, fragment: u32) {
        // Sort based on current cell indices in VS Code
        let mut cells_vec: Vec<&Cell> = self.cells.iter().map(|(_, cell)| (cell)).collect();
        cells_vec.sort_by(|a, b| a.index.cmp(&b.index));

        // Write the file output
        let mut output = "fn main() {\n".to_string();
        let mut crates = String::new();
        let mut outer_scope = String::new();
        for cell in cells_vec {
            let lines = cell.contents.split('\n');
            for line in lines {
                let line = line.trim();
                // Don't print if it's not the executing cell
                if line.starts_with("print") && cell.fragment != fragment {
                } else if line.starts_with("use") {
                    outer_scope += line;
                    outer_scope += "\n";
                    let (_, full_path) = line.split_once(' ').unwrap();
                    let (crate_name, _) = full_path.split_once(':').unwrap();
                    crates += crate_name;
                    crates += "=\"*\"\n";
                } else {
                    output += line;
                    output += "\n";
                }
            }
        }
        output += "\n\n}";
        let mut main = outer_scope.clone();
        main += &output;

        // Get temp file names
        let mut rust_file = String::from(&self.temp_dir);
        let mut cargo_file = String::from(&self.temp_dir);
        rust_file += "/main.rs";
        cargo_file += "/Cargo.toml";
        let cargo_contents = format!(
            "{}\n{}",
            r#"
[package]
name = 'output'
version = '0.0.1'
edition = '2021'
[[bin]]
name = 'ouput'
path = 'main.rs'
[dependencies]
"#,
            crates
        );

        // Write the files
        write(&rust_file, &main).expect("Error writing file");
        write(&cargo_file, &cargo_contents).expect("Error writing file");
    }

    /// Run the program by running `cargo run` in the temp directory
    /// then uses regex to determine what part of the output to send back
    /// to the caller
    pub fn run(&self) -> String {
        use std::process::Command;
        let output = Command::new("cargo")
            .current_dir(&self.temp_dir)
            .arg("run")
            .output()
            .expect("Failed to run cargo");

        let err = String::from_utf8(output.stderr).expect("Failed to parse utf8");

        if err.contains("error: ") || err.contains("panicked at") {
            // 1 denotes an error
            return format!("1\0{}", err);
        }

        let output = String::from_utf8(output.stdout).unwrap();
        format!("0\0{}", output)
    }
}
