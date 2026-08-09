//! Small demo command wired through the full stack: frontend invoke → Tauri
//! command → shared core crate. Proves the JS/Rust bridge and the workspace
//! are wired together.

/// Build a greeting using the shared core crate.
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! — from easy-music-core (Rust)")
}
