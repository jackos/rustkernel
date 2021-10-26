// Makes `Cell` and `Program` available from root e.g. `use rustkernel::Program`
pub use self::program::{Cell, Program};

// Links the code from `handlers.rs` and `program.rs`
pub mod handlers;
pub mod program;

use regex::Regex;

/// Extracts body of request using a regex capture group and parsing the content length.
/// Splits the request in two and takes the second block following the http 1.1 standard
/// of a separation by two newlines
///
/// # Examples
/// ```
/// use rustkernel::extract_body;
/// let req = r"POST / HTTP/1.1
/// Content-Type: text/plain;charset=UTF-8
/// Accept: */*
/// Content-Length: 16
/// User-Agent: node-fetch/1.0 (+https://github.com/bitinn/node-fetch)
/// Accept-Encoding: gzip,deflate
/// Connection: close
/// Host: 127.0.0.1:8787
///
/// body-placeholder";
/// let res = extract_body(req);
/// println!("{}", res);
/// assert_eq!(res, "body-placeholder");
/// ```
pub fn extract_body(req: &str) -> String {
    let double_newline =
        Regex::new(r"(?:\r\r|\n\n|\r\n\r\n)(.*)$").expect("Couldn't compile double newline regex");
    let b = double_newline.captures(&req).expect("Error capturing body");
    String::from(
        b.get(1)
            .expect("Regex failed to match double newline")
            .as_str()
            .trim(),
    )
}

#[cfg(test)]
mod tests {
    use crate::extract_body;

    #[test]
    fn returning_body_from_request() {
        let req = "POST / HTTP/1.1
Content-Type: text/plain;charset=UTF-8
Accept: */*
Content-Length: 16
User-Agent: node-fetch/1.0 (+https://github.com/bitinn/node-fetch)
Accept-Encoding: gzip,deflate
Connection: close
Host: 127.0.0.1:8787

{\"test\":\"cool\"};";
        let res = extract_body(req);
        println!("{}", res);
        assert_eq!(&res, "{\"test\":\"cool\"};");
    }

    #[test]
    #[should_panic]
    fn test_panics_on_malformed_request() {
        let req = r"POST / HTTP/1.1
Content-Type: text/plain;charset=UTF-8
Accept: */*
Content-Length: 120
User-Agent: node-fetch/1.0 (+https://github.com/bitinn/node-fetch)
Accept-Encoding: gzip,deflate
Connection: close
Host: 127.0.0.1:8787
body-placeholder";
        extract_body(req);
    }
}
