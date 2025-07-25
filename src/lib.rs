#![feature(associated_type_defaults)]
#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(iter_from_coroutine)]
#![feature(backtrace_frames)]

mod command_arg;
mod command_list;
mod command_metadata;
mod error;
mod exception;
mod run_loop;
mod shell;
mod tokenizer;

pub use mysh_derive::*;

pub use command_arg::{parse_arguments, CommandArg};
pub use command_metadata::CommandMetadata;
pub use error::{Error, Result};
pub use futures;
pub use reedline::ExternalPrinter;
pub use shell::{DefaultLineReader, PromptText};
pub use shell::{Scripts, Shell};

pub mod json {
  pub use serde_json::to_value;
  pub use serde_json::Value;
}

#[macro_export]
macro_rules! shell {
  ($cmd:expr, [$($arg:expr),* $(,)?]) => {{
    use anyhow::Context;
    std::process::Command::new($cmd)
      .args(&[$($arg),*])
      .status()
      .context("cmd failed")?
  }};
}

pub mod __dev {
  pub use crate::error::ToTrace;
  pub use crate::exception::{
    extract_anyhow_sources, extract_error_sources, ExceptionWithTrace, FrameSymbol,
  };
}
