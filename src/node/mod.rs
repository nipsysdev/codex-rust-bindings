//! Node management for Storage
//!
//! This module provides functionality for creating, configuring, starting,
//! stopping, and destroying Storage nodes.

pub mod config;
pub mod lifecycle;

pub use config::{LogFormat, LogLevel, NetworkPreset, RepoKind, StorageConfig};
pub use lifecycle::StorageNode;
