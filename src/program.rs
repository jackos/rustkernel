use regex::Regex;
use substring::Substring;

use crate::handlers::CodeRequest;
use std::collections::HashMap;
use std::env;
use std::fs::write;

/// This is what sits in state as long as the program is running
/// when a new request is made, it will check if the same cell has
/// been executed by checking `cells` which is a `HashMap`
/// # Terms
/// - temp_dir: Whatever the OS temp directory is
/// - executed_filename: Last executed filename so we can restart state if necessary
/// - functions: will rip out anything inside a fn() and put it in the outer scope
/// - cells: VS Code name for a notebook cell
#[derive(Debug)]
pub struct Program {
    pub temp_dir: String,
    pub executed_filename: String,
    pub functions: String,
    pub cells: HashMap<u32, Cell>,
}

/// A cell represent a notebook cell in VS Code
///
/// # Terms
/// - fragment: unique id of the cell that executed
/// - index: current index by order in VS Code
/// - contents: the source code from inside the cell
/// - executing: if true this cell is where the request came from,
/// used to determine what output to return to caller
#[derive(Debug)]
pub struct Cell {
    pub fragment: u32,
    pub index: u32,
    pub contents: String,
    pub executing: bool,
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

        Program {
            temp_dir: temp_file,
            executed_filename: "".to_string(),
            functions: "".to_string(),
            cells: HashMap::new(),
        }
    }

    /// Initial cell creation if it doesn't exist yet
    /// Contents Use clone as we need to retain
    /// a pointer to data that will stick around after the request has finished
    pub fn create_cell(&mut self, cr: &CodeRequest) {
        println!("Creating cell with fragment id: {}", cr.fragment);
        let first_cell = Cell {
            contents: cr.contents.clone(),
            executing: true,
            fragment: cr.fragment,
            index: cr.index,
        };
        self.cells.insert(cr.fragment, first_cell);
    }

    /// As the cells are implemented with a HashMap this is a fast lookup,
    /// Only updates what's required
    pub fn update_cell(&mut self, cr: &CodeRequest) {
        println!("Updating cell with fragment id: {}", cr.fragment);
        if let Some(cell) = self.cells.get_mut(&cr.fragment) {
            cell.contents = cr.contents.clone();
            cell.executing = true;
            cell.index = cr.index;
        }
    }

    /// First sorts the hashmap by index into a vector, then writes the output with
    /// `rustkernel-start` and `rustkernel-end` to determine which cell's was executing
    // so we return only that output.
    pub fn write_to_file(&mut self, fragment: u32) {
        // Sort based on current index's in VS Code
        let mut cells_vec: Vec<&Cell> = self.cells.iter().map(|(_, cell)| (cell)).collect();
        cells_vec.sort_by(|a, b| a.index.cmp(&b.index));

        // Write the file output
        let mut output = "fn main() {\n".to_string();
        for cell in cells_vec {
            if cell.fragment == fragment {
                output += "\nprintln!(\"rustkernel-start\");"
            }
            output += "\n";
            output += &cell.contents;
            if cell.fragment == fragment {
                output += "\nprintln!(\"rustkernel-end\");"
            }
        }
        output += "\n\n}";

        // Get temp file names
        let mut rust_file = String::from(&self.temp_dir);
        let mut cargo_file = String::from(&self.temp_dir);
        rust_file += "/main.rs";
        cargo_file += "/Cargo.toml";
        let cargo_contents = "[package]\nname = 'output'\nversion = '0.0.1'\nedition = '2021'\n[[bin]]\nname = 'ouput'\npath = 'main.rs'\n";

        // Write the files
        write(&rust_file, &output).expect("Error writing file");
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

        let mut response_code = "HTTP/1.1 200 OK";
        let err = String::from_utf8(output.stderr).expect("Failed to parse utf8");

        if err.contains("error: ") || err.contains("panicked at") {
            response_code = "HTTP/1.1 422 Unprocessable Entity";
            return format!(
                "{}\r\nContent-Length: {}\r\n\r\n{}",
                response_code,
                err.len(),
                err
            );
        }

        let out = String::from_utf8(output.stdout).expect("Failed to parse utf8");
        let re_start = Regex::new(r"rustkernel-start").expect("Couldn't compile start regex");
        let re_end = Regex::new(r"rustkernel-end").expect("Couldn't compile end regex");

        let start = re_start
            .find(&out)
            .expect("Couldn't find start of output")
            .end();
        let end = re_end
            .find(&out)
            .expect("Couldn't find end of output")
            .start();

        let result = out.substring(start, end);
        println!("body: {}", &result);

        format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            response_code,
            result.len(),
            result
        )
    }
}
