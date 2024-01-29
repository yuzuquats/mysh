use colored::Colorize;
use futures::Future;

pub use mysh_derive::*;

use crate::error::Error;

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

pub struct CommandList<Info> {
  pub commands: Vec<Box<dyn CommandMetadata<Info>>>,
}

impl<Info> CommandList<Info>
where
  Info: Clone,
{
  pub fn print_help(&self) {
    println!("\nUsage: [name] [command]\n");

    println!("Commands:");
    for command in &self.commands {
      println!("    {:10} {}", command.name().bold(), command.description());
      if command.help().len() > 0 {
        println!("    {:10} {}", "", command.help());
      }
    }
    println!("");
  }

  pub fn exec(
    &self,
    info: Info,
    argv: Vec<String>,
  ) -> impl Future<Output = Result<(), Error>> + '_ {
    // todo: something something need to make sure it's thread safe
    self.exec_raw(info, argv)
  }
}

impl<Info> CommandList<Info> {
  async fn exec_raw(&self, info: Info, argv: Vec<String>) -> Result<(), Error>
  where
    Info: Clone,
  {
    let name = argv.get(0).expect("");
    if name == "help" {
      let Some(subcommand_name) = argv.get(1) else {
        self.print_help();
        return Ok(());
      };

      let Some(subcommand) = self.commands.iter().find(|c| c.name() == subcommand_name) else {
        println!("Command not found: {}", subcommand_name);
        return Ok(());
      };

      subcommand.print_help();
      return Ok(());
    }

    let Some(command) = self.commands.iter().find(|c| c.name() == name) else {
      self.print_help();
      return Ok(());
    };

    command.call_with_argv(info, argv)?.await
  }
}
