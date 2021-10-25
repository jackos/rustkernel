use rustkernel::handlers;
use rustkernel::Program;
use std::net::TcpListener;

fn main() {
    // Start the server
    let host: &str = "127.0.0.1:8787";
    let listener = TcpListener::bind(host).expect("Could not start listener");
    println!("Listening at {}...", host);

    // The Program will remain in state while the server is running
    let mut program = Program::new();

    // This is the main loop over incoming streams
    for stream in listener.incoming() {
        let stream = stream.expect("Could not iterate over stream");
        handlers::code_request(stream, &mut program);
    }
}
