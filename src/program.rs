use crate::handlers::CodeRequest;
use std::collections::HashMap;
use std::env;

pub struct Program {
    pub temp_file: String,         // Temp file location
    pub executed_filename: String, // The most recently executed filename
    pub functions: String,         // The code that is pulled out of the main function
    pub cells: HashMap<i32, Cell>, // Represents a cell from VS Code notebook
}

pub struct Cell {
    pub fragment: i32,    // What index the cell was at, at time of execution
    pub index: i32,       // Current index by order in VS Code
    pub contents: String, // What's inside the cell
    pub executing: bool,  // The cell that is currently being executed
    pub filename: String, // What file the executing cell is from
}

impl<'a> Program {
    pub fn new(cr: &'a mut CodeRequest) -> Program {
        let mut temp_file = env::temp_dir()
            .to_str()
            .expect("Error getting temp directory")
            .to_string();
        temp_file = temp_file + "/main.rs";

        let mut program = Program {
            temp_file,
            executed_filename: cr.filename.clone(),
            functions: "".to_string(),
            cells: HashMap::new(),
        };
        let first_cell = Cell {
            contents: cr.contents.clone(),
            executing: true,
            filename: cr.filename.clone(),
            fragment: cr.fragment,
            index: cr.index,
        };
        program.cells.insert(0, first_cell);
        program
    }

    pub fn add_cell(cell: Cell) {}
}
