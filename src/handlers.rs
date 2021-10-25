use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

pub fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "Hello")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "Hello after sleep")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "../html/404.html")
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
