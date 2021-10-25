use crate::handlers::CodeRequest;
use std::collections::HashMap;
use std::env;

#[derive(Debug)]
pub struct Program {
    pub temp_file: String,         // Temp file location
    pub executed_filename: String, // The most recently executed filename
    pub functions: String,         // The code that is pulled out of the main function
    pub cells: HashMap<i32, Cell>, // Represents a cell from VS Code notebook
}

#[derive(Debug)]
pub struct Cell {
    pub fragment: i32,    // What index the cell was at, at time of execution
    pub index: i32,       // Current index by order in VS Code
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
        temp_file = temp_file + "/main.rs";

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
        println!("{:?}", self);
    }
}
