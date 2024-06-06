use crate::{error::Error, CommandArg};
use colored::Colorize;
use futures::Future;

#[derive(Clone)]
pub struct DefaultArg;

impl CommandArg for DefaultArg {
  fn display_help() -> &'static str {
    todo!()
  }
}

pub trait CommandMetadata<Info> {
  fn name(&self) -> &'static str;
  fn description(&self) -> &'static str;
  fn long_description(&self) -> Option<&'static str>;
  fn call_with_argv(
    &self,
    info: Info,
    argv: Vec<String>,
  ) -> Result<std::pin::Pin<Box<dyn Future<Output = Result<(), Error>>>>, Error>;
  fn help(&self) -> &'static str;

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
      for option in options.split_whitespace() {
        println!("    {}", option);
      }
    }
  }
}
