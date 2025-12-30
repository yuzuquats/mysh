use crate::{CommandArg, error::Error};
use colored::Colorize;
use futures::Future;
use serde_json::Value;

impl CommandArg for () {
  fn display_help() -> Vec<String> {
    vec![]
  }
}

pub trait CommandMetadata<Info> {
  fn name(&self) -> &'static str;
  fn description(&self) -> &'static str;
  fn long_description(&self) -> Option<&'static str>;
  fn confirmation_message(&self) -> Option<&'static str> {
    None
  }
  fn call_with_argv(
    &self,
    info: Info,
    argv: Vec<String>,
  ) -> Result<std::pin::Pin<Box<dyn Future<Output = Result<Value, Error>>>>, Error>;
  fn help(&self) -> Vec<String>;

  fn print_help(&self) {
    let options = self.help();

    println!(
      "\n{}\n    {} {}",
      "Name:".bold(),
      self.name().bold(),
      if options.len() > 0 { "[OPTIONS]" } else { "" }
    );
    println!(
      "\n{}\n    {}\n",
      "Description:".bold(),
      self.long_description().unwrap_or(self.description())
    );
    if options.len() > 0 {
      println!("{}", "Options:".bold());
      for option in options {
        println!("    {}", option);
      }
    }
  }
}
