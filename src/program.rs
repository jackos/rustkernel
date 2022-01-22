//! This module is responsible for keeping the program in state
//! between API calls. The `Program` struct holds the cells in a
//! hashmap
use crate::handlers::CodeRequest;
use std::collections::HashMap;
use std::fs;
use std::fs::write;
use std::process::Command;

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
    pub filename: String,
    pub workspace: String,
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
        Program {
            filename: String::new(),
            workspace: String::new(),
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
        let mut crates = String::new();
        let mut outer_scope = String::new();
        let mut inner_scope = String::new();
        let mut contains_main = false;
        for cell in cells_vec {
            let lines = cell.contents.split('\n');
            for line in lines {
                let line = line.trim();
                // If it contains main we won't add a main function
                if line.starts_with("fn main()") {
                    contains_main = true;
                    continue;
                }
                // Don't print if it's not the executing cell
                if line.starts_with("print") && cell.fragment != fragment {
                    continue;
                }

                if line.starts_with("use") {
                    outer_scope += line;
                    outer_scope += "\n";
                    if !line.starts_with("use std") {
                        let (_, full_path) = line.split_once(' ').unwrap();
                        let (crate_name, _) = full_path.split_once(':').unwrap();
                        let crate_name_fixed = str::replace(crate_name, "_", "-");
                        crates += &crate_name_fixed;
                        crates += "=\"*\"\n";
                    }
                } else {
                    inner_scope += line;
                    inner_scope += "\n";
                }
            }
            if contains_main {
                inner_scope = inner_scope.trim_end().to_string();
                inner_scope.pop();
                contains_main = false;
            }
        }

        let mut output = "#![allow(dead_code)]\n".to_string();
        output += outer_scope.as_str();
        output += "fn main() {\n";
        output += &inner_scope;
        output += "}";

        // Use the folder name sent from VS Code to create the
        // file structure required for a cargo project
        let mut dir = self.workspace.to_owned();

        let mut cargo_file = dir.clone();
        cargo_file += "/Cargo.toml";

        dir += "/src";
        let mut main_file = dir.to_owned();
        main_file += "/main.rs";

        if let Err(err) = fs::create_dir_all(&dir) {
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                panic!("Can't create dir: {}, err {}", &dir, err)
            }
        };

        let cargo_contents = format!(
            "{}\n{}",
            r#"
[package]
name = 'output'
version = '0.0.1'
edition = '2021'
[dependencies]
"#,
            crates
        );

        // Write the files
        write(&main_file, &output).expect("Error writing file");
        write(&cargo_file, &cargo_contents).expect("Error writing file");
    }

    /// Run the program by running `cargo run` in the temp directory
    /// then uses regex to determine what part of the output to send back
    /// to the caller
    pub fn run(&self) -> String {
        let output = Command::new("cargo")
            .current_dir(&self.workspace)
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

    pub fn fmt(&self) {
        Command::new("cargo")
            .current_dir(&self.workspace)
            .arg("fmt")
            .output()
            .expect("Failed to run cargo fmt");
    }
}
