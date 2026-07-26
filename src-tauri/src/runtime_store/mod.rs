pub(crate) mod commands;
mod config;
mod connection;
mod control;
mod deadline;
mod error;
mod lifecycle;
mod migrations;
// Phase 1B.2 intentionally delivers a private Rust service before a later
// runtime consumer is authorized. Keep that bounded dormant API warning-free.
#[allow(dead_code)]
mod models;
mod path_policy;
mod repositories;
mod types;
mod worker;

pub(crate) use config::RuntimeStoreConfig;
pub(crate) use lifecycle::RuntimeStoreLifecycle;
pub(crate) use worker::RuntimeStoreManager;

#[cfg(test)]
mod phase_1b3_tests;
#[cfg(test)]
mod repository_tests;
#[cfg(test)]
mod tests;
