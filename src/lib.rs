// Makes `Cell` and `Program` available from root e.g. `use rustkernel::Program`
pub use self::program::{Cell, Program};

// Links the code from `handlers.rs` and `program.rs`
pub mod handlers;
pub mod program;
