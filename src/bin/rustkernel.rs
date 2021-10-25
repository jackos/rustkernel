use rustkernel::handlers::handle_connection;
use rustkernel::ThreadPool;
use std::net::TcpListener;

const ADDR: &str = "127.0.0.1:8787";

fn main() {
    let listener = TcpListener::bind(ADDR).expect("Could not start listener");
    println!("Listening at {}...", ADDR);

    let pool = match ThreadPool::new(1) {
        Err(e) => panic!("Error creating pool: {}", e),
        Ok(pool) => pool,
    };

    for stream in listener.incoming() {
        let stream = stream.expect("Could not iterate over stream");

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}
