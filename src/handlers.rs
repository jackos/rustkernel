use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::net::TcpStream;
use String;

use crate::Program;

/// Format the request from VS Code comes through in
///
/// # Terms
/// - index: the current vertical position of the cell that was executed
/// - fragment: a unique ID for each cell, if they change order this id remains the same
/// - filename: Where the file was executed from, important to reset the code if the user changes files
/// - contents: the contents / source code of a cell
#[derive(Serialize, Deserialize, Debug)]
pub struct CodeRequest {
    pub index: u32,
    pub fragment: u32,
    pub filename: String,
    pub contents: String,
}

/// All requests run through here, it
/// # Parameters
/// stream: Contains the http request, it's read from which modifies
/// it's internal state, which is why it needs to `mut`. Also contains
/// a reference to the `Program` which lives for the lifetime of the
/// server running. It retains information between requests
pub fn code_request(mut stream: TcpStream, program: &mut Program) {
    // Set a buffer and read in the stream
    let mut buffer = [0; 8192];
    stream.read(&mut buffer).expect("Error reading stream");

    // Convert the utf8 to a string, lossy means unknown characters
    // are replaced with `?`
    // `Cow` is `copy on write` but not using it as a mutable
    let req = String::from_utf8_lossy(&buffer);

    // Uses the library function to extract the body from a HTTP 1.1 request
    let body = crate::extract_body(&req);

    // Marshalls the json
    let cr: CodeRequest = serde_json::from_str(&body).expect("Error parsing JSON");

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
}
