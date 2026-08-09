//! easy-music-plugin-sdk — types and contracts shared between the plugin host
//! (easy-music-core / src-tauri) and the plugin runtime.
//!
//! This crate is deliberately dependency-light so it can be consumed by both
//! the core crate and any future host-side tooling without pulling in heavy
//! deps.

pub mod hook;
pub mod manifest;
pub mod permission;

pub use hook::PluginHook;
pub use manifest::{PluginEngine, PluginManifest};
pub use permission::Permission;
