pub use self::program::{Cell, Program};

pub mod handlers;
pub mod program;

use regex::Regex;
use std::borrow::Cow;

/// Extracts body of request using a regex capture group and parsing the content length
/// It then splits the request in two and takes the second block following the http 1.1
/// spec of the body separating it by two CRLF's
pub fn extract_body(req: Cow<str>) -> String {
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
