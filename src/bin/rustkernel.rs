use rustkernel::handlers::handle_connection;
use rustkernel::program::Program;
use rustkernel::thread_pool::ThreadPool;
use std::net::TcpListener;

fn main() {
    let host: &str = "127.0.0.1:8787";

    let listener = TcpListener::bind(host).expect("Could not start listener");
    println!("Listening at {}...", host);

    let pool = match ThreadPool::new(1) {
        Err(e) => panic!("Error creating pool: {}", e),
        Ok(pool) => pool,
    };

    for stream in listener.incoming() {
        let stream = stream.expect("Could not iterate over stream");

        let x = pool.execute(|| {
            handle_connection(stream);
        });

        pool.workers[0].program = Some(x);
    }

    println!("Shutting down.");
}
