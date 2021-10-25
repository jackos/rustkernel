use crate::handlers::CodeRequest;
use std::collections::HashMap;
use std::env;
use std::fs::write;

#[derive(Debug)]
pub struct Program {
    pub temp_file: String,         // Temp file location
    pub executed_filename: String, // The most recently executed filename
    pub functions: String,         // The code that is pulled out of the main function
    pub cells: HashMap<u32, Cell>, // Represents a cell from VS Code notebook
}

#[derive(Debug)]
pub struct Cell {
    pub fragment: u32,    // What index the cell was at, at time of execution
    pub index: u32,       // Current index by order in VS Code
    pub contents: String, // What's inside the cell
    pub executing: bool,  // The cell that is currently being executed
    pub filename: String, // What file the executing cell is from
}

impl Program {
    pub fn new() -> Program {
        let mut temp_file = env::temp_dir()
            .to_str()
            .expect("Error getting temp directory")
            .to_string();
        temp_file = temp_file;

        Program {
            temp_file,
            executed_filename: "".to_string(),
            functions: "".to_string(),
            cells: HashMap::new(),
        }
    }

    pub fn create_cell(&mut self, cr: &CodeRequest) {
        println!("Creating cell with fragment id: {}", cr.fragment);
        let first_cell = Cell {
            contents: cr.contents.clone(),
            executing: true,
            filename: cr.filename.clone(),
            fragment: cr.fragment,
            index: cr.index,
        };
        self.cells.insert(cr.fragment, first_cell);
    }

    pub fn update_cell(&mut self, cr: &CodeRequest) {
        println!("Updating cell with fragment id: {}", cr.fragment);
        if let Some(cell) = self.cells.get_mut(&cr.fragment) {
            cell.contents = cr.contents.clone();
            cell.executing = true;
            cell.filename = cr.filename.clone();
            cell.index = cr.index;
        }
    }

    pub fn write_to_file(&mut self) {
        // Sort based on current index's in VS Code
        let mut cells_vec: Vec<(u32, &String)> = self
            .cells
            .iter()
            .map(|(_, cell)| (cell.index, &cell.contents))
            .collect();
        cells_vec.sort_by(|a, b| a.0.cmp(&b.0));

        // Write the file output
        let mut output = "fn main() {\n".to_string();
        for (_, contents) in cells_vec {
            output += "\n";
            output += contents;
        }
        output += "\n\n}";

        // Get temp file names
        let mut rust_file = String::from(&self.temp_file);
        let mut cargo_file = String::from(&self.temp_file);
        rust_file += "/main.rs";
        cargo_file += "/Cargo.toml";
        let cargo_contents = "[package]\nname = 'output'\nversion = '0.0.1'\nedition = '2021'\n[[bin]]\nname = 'ouput'\npath = 'main.rs'\n";

        // Write the files
        write(&rust_file, &output).expect("Error writing file");
        write(&cargo_file, &cargo_contents).expect("Error writing file");
    }
}
