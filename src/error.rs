use crate::__dev::ExceptionWithTrace;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  #[error("arg parse error: {0}")]
  ArgParseError(String),
  #[error("Please provide a subcommand. Available subcommands: {0}")]
  MissingSubcommand(String),
  #[error("No such subcommand. ie. ./[bin] [command] [subcommand]")]
  NoSuchSubcommand,
  #[error("Command not found: {0}")]
  CommandNotFound(String),
  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ToTrace {
  fn to_trace(&self) -> ExceptionWithTrace;
}

impl ToTrace for Error {
  fn to_trace(&self) -> ExceptionWithTrace {
    let message = Some(format!("{}", self));
    let mut backtrace = None;

    let sources = match self {
      Error::Other(error) => {
        backtrace = Some(error.backtrace());

        let mut sources = Vec::new();
        let mut current_source = error.source();
        while let Some(source) = current_source {
          sources.push(format!("{}", source));
          current_source = source.source();
        }

        sources
      }
      Error::ArgParseError(_) => vec![],
      Error::MissingSubcommand(_) => vec![],
      Error::NoSuchSubcommand => vec![],
      Error::CommandNotFound(_) => vec![],
    };

    let mut exception = ExceptionWithTrace::with_sources(message, sources, backtrace);

    exception.filtered_range.0 = Some("anyhow::__private::format_err".to_string());
    exception.filtered_range.1 = Some("mysh::run_loop::run_once_or_loop".to_string());

    exception
  }
}

#[cfg(test)]
mod test {
  use super::{Error, ToTrace};
  use anyhow::{Context, anyhow};
  use std::io;

  #[test]
  pub fn test_error_to_trace_anyhow() {
    // Create an error chain
    let err = match test_error_parent() {
      Ok(_) => panic!("should not OK"),
      Err(e) => Error::Other(e),
    };

    let exception = err.to_trace();

    // Verify the primary message
    assert!(exception.message.is_some(), "Should have a message");

    // For anyhow errors, we expect source chain to be populated
    assert!(!exception.sources.is_empty(), "Should have sources");

    // Combine message and sources for checking
    let all_text = format!(
      "{} {}",
      exception.message.as_ref().unwrap(),
      exception.sources.join(" ")
    );

    // Should contain some of our error context
    assert!(
      all_text.contains("parent") || all_text.contains("context 1") || all_text.contains("oh no"),
      "Error chain should contain expected context"
    );

    // Verify that frames are not empty
    assert!(!exception.frames.is_empty(), "Should have stack frames");

    // Verify filtered_range is set to the custom values
    assert_eq!(
      exception.filtered_range.0,
      Some("anyhow::__private::format_err".to_string())
    );
    assert_eq!(
      exception.filtered_range.1,
      Some("mysh::run_loop::run_once_or_loop".to_string())
    );
  }

  #[test]
  pub fn test_error_to_trace_io() {
    // Create an IO error and wrap it
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let err = Error::Other(anyhow::Error::new(io_error));

    let exception = err.to_trace();

    // Verify the primary message
    assert!(exception.message.is_some(), "Should have a message");
    assert!(
      exception
        .message
        .as_ref()
        .unwrap()
        .contains("file not found"),
      "Message should contain error text"
    );

    // Verify that frames are not empty
    assert!(!exception.frames.is_empty(), "Should have stack frames");

    // Verify filtered_range is set to the custom values
    assert_eq!(
      exception.filtered_range.0,
      Some("anyhow::__private::format_err".to_string())
    );
    assert_eq!(
      exception.filtered_range.1,
      Some("mysh::run_loop::run_once_or_loop".to_string())
    );
  }

  // Helper functions to create nested errors
  fn test_error_parent() -> anyhow::Result<()> {
    test_error_child_1().context("parent")?;
    Ok(())
  }

  fn test_error_child_1() -> anyhow::Result<()> {
    test_error_child_2().context("context 1")?;
    Ok(())
  }

  fn test_error_child_2() -> anyhow::Result<()> {
    Err(anyhow!("oh no"))?;
    Ok(())
  }
}
