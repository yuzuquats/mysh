use colored::Colorize;
use futures::Future;

use crate::command_arg::CommandArg;
use crate::command_metadata::CommandMetadata;
use crate::error::Error;

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
