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

pub use command_arg::{CommandArg, parse_arguments};
pub use command_metadata::CommandMetadata;
pub use error::{Error, Result};
pub use futures;
pub use reedline::ExternalPrinter;
pub use shell::{DefaultLineReader, PromptText};
pub use shell::{Scripts, Shell};

pub mod json {
  pub use serde_json::Value;
  pub use serde_json::to_value;
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
    ExceptionWithTrace, FrameSymbol, extract_anyhow_sources, extract_error_sources,
  };
}

/// Prompts the user with a confirmation message and returns true if they confirm.
/// The message is displayed with colored formatting.
pub fn confirm_action(message: &str) -> anyhow::Result<bool> {
  use colored::Colorize;
  use std::io::{BufRead, Write};

  print!(
    "{} {} {} ",
    "⚠️ ".yellow().bold(),
    message.red().bold(),
    "Are you sure? [y/N]:".yellow()
  );
  std::io::stdout()
    .flush()
    .context("failed to flush stdout")?;

  let mut input = String::new();
  std::io::stdin()
    .lock()
    .read_line(&mut input)
    .context("failed to read confirmation")?;

  let input = input.trim().to_lowercase();
  Ok(input == "y" || input == "yes")
}

use anyhow::Context;
