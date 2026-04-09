#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(rustdoc::all)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::too_long_first_doc_paragraph)]
#![warn(
    clippy::todo,
    reason = "在`git commit`之前，请确认代码中没有`todo!()`"
)]

pub mod ca;
pub mod config;
mod error;
pub mod gym;
pub mod hdjw;
pub mod lab;
mod login;
pub mod netflow;
pub mod pt;
mod utils;
pub mod wxpay;
pub mod xgxt;
pub use error::Error;

#[cfg(test)]
pub mod test;
