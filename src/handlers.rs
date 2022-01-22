use std::io::prelude::*;
use std::net::TcpStream;
use String;

use crate::Program;

/// The requests from VS Code are marshalled through `Serde` via this struct
/// # Terms
/// - index: the current vertical position of the cell that was executed
/// - fragment: a unique ID for each cell, if they change order this id remains the same
/// - filename: Where the file was executed from, important to reset the code if the user changes files
/// - contents: the contents / source code of a cell
pub struct CodeRequest {
    pub index: u32,
    pub fragment: u32,
    pub filename: String,
    pub workspace: String,
    pub contents: String,
}

/// All requests run through here
/// # Parameters
/// stream: Contains the http request, when it's read from it modifies
/// internal state, which is why it needs to be `mut`. Also contains
/// a reference to the `Program` which lives for the lifetime of the
/// server running. It retains information between requests
pub fn code_request(mut stream: TcpStream, program: &mut Program) {
    // Set a buffer and read in the stream
    let mut buffer = [0; 8192];
    stream.read(&mut buffer).expect("Error reading stream");

    // Convert the utf8 to a string
    let req = String::from_utf8_lossy(&buffer);
    let mut req_iter = req.split("\0");

    let cr = CodeRequest {
        index: req_iter.next().unwrap().parse().unwrap(),
        fragment: req_iter.next().unwrap().parse().unwrap(),
        filename: req_iter.next().unwrap().to_string(),
        workspace: req_iter.next().unwrap().to_string(),
        contents: req_iter.next().unwrap().to_string(),
    };

    // If the filename is different to last run, this resets
    // the state of `Program`
    if cr.filename != program.filename {
        *program = Program::new();
    }
    program.filename = cr.filename.to_owned();
    program.workspace = cr.workspace.to_owned();

    // If there is a cell already there, update existing, otherwise create
    match program.cells.get(&cr.fragment) {
        Some(_) => program.update_cell(&cr),
        None => program.create_cell(&cr),
    }

    // Runs through all the cells and creates a program to run
    program.write_to_file(cr.fragment);

    // Run the program and return response to caller
    let response = program.run();
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    // Format the file for anyone looking at the source code
    program.fmt();
}
