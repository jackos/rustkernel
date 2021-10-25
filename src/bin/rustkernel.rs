use rustkernel::{handlers, program::Program};
use std::net::TcpListener;

fn main() {
    let host: &str = "127.0.0.1:8787";

    let listener = TcpListener::bind(host).expect("Could not start listener");
    println!("Listening at {}...", host);

    // This program will remain in state while the server is running
    let mut program = Program::new();

    for stream in listener.incoming() {
        // Create a program if it doesn't yet exist
        let stream = stream.expect("Could not iterate over stream");

        handlers::handle_connection(stream, &mut program);
    }

    println!("Shutting down.");
}
