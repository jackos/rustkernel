use regex::Regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::io::prelude::*;
use std::net::TcpStream;
use String;

use crate::program::Program;

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeRequest {
    pub index: i32,
    pub fragment: i32,
    pub filename: String,
    pub contents: String,
}

/// Extracts body of request using a regex capture group and parsing the content length
/// It then splits the request in two and takes the second block following the http 1.1
/// spec of the body separating it by two CRLF's
fn extract_body(req: Cow<str>) -> String {
    let re = Regex::new(r"Content-Length: (\d.*)").unwrap();
    let capture = re.captures(&req).expect("Error with regex");
    // Get the body content length
    let content_len: usize = capture
        .get(1)
        .expect("Regex failed to match Content-Length")
        .as_str()
        .trim()
        .parse()
        .expect("Failed to parse content length");

    // Use an iterator to get the second block
    let mut para = req.split("\r\n\r\n");
    para.next();
    let body = String::from(para.next().expect("Error getting body from request"));
    body[0..content_len].to_string()
}

pub fn handle_connection(mut stream: TcpStream, program: &mut Program) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Error reading stream");
    let req = String::from_utf8_lossy(&buffer);
    let body = extract_body(req);

    let cr: CodeRequest = serde_json::from_str(&body).expect("Error parsing JSON");

    match program.cells.get(&cr.fragment) {
        Some(_) => program.update_cell(&cr),
        None => program.create_cell(&cr),
    }

    // Well formed request to root is all we'll
    let post = b"POST / HTTP/1.1";

    let (status_line, filename) = if buffer.starts_with(post) {
        ("HTTP/1.1 200 OK", "Hello")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "The only route that exists is /")
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        filename.len(),
        filename
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
