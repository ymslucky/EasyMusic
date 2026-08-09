//! easy-music-plugin-sdk — types and contracts shared between the plugin host
//! (easy-music-core / src-tauri) and the plugin runtime.
//!
//! This crate is deliberately dependency-light so it can be consumed by both
//! the core crate and any future host-side tooling without pulling in heavy
//! deps.

pub mod manifest;
pub mod permission;
pub mod hook;

pub use manifest::{PluginManifest, PluginEngine};
pub use permission::Permission;
pub use hook::PluginHook;
