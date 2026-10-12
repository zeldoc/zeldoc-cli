//! The `zeldoc` command-line client for Zeldoc.ai.
//!
//! Each feature (`auth`, `key_details`, `models`, `search`, `update`, `usage`) has a
//! command (its clap arguments and what it prints), a service that talks to the API, and its
//! data transfer objects and request parameters, one type per file.

pub mod api_request_error;
pub mod api_request_handler;
pub mod auth;
pub mod cli;
pub mod command;
pub mod constants;
pub mod credentials;
pub mod key_details;
pub mod models;
pub mod number_text;
pub mod search;
pub mod terminal_text;
pub mod text_table;
pub mod update;
pub mod usage;
