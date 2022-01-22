use rustkernel::handlers;
use rustkernel::Program;
use std::net::TcpListener;

fn main() {
    // Start the server
    let host: &str = "127.0.0.1:8787";
    let listener = TcpListener::bind(host).expect("Could not start listener");
    println!(
        r"
-------------------------------------------------------------
-- Rustkernel - For VS Code Extension - Rustnote ------------
-------------------------------------------------------------
Rustkernel is running on {host}

The default path for your notes is ~/rustnote
Change the path in File > Preferences > Settings > 'rustnote'

--------------------------------------------------------------
-- Default Keybindings ---------------------------------------
--------------------------------------------------------------
alt+f: Search notes in VS Code, add rustnote path to workspace
alt+p: Preview notes as static website using mdBook
alt+o: Open main.rs to see what code is being generated

Change in File > Preferences > Keyboard Shortcuts > 'rustnote'
"
    );
    // The Program will remain in state while the server is running
    let mut program = Program::new();

    // This is the main loop over incoming streams
    for stream in listener.incoming() {
        let stream = stream.expect("Could not iterate over stream");
        handlers::code_request(stream, &mut program);
    }
}
