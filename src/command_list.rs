use std::collections::HashMap;

use colored::Colorize;

use crate::command_metadata::CommandMetadata;

pub struct CommandList<Info> {
  commands: HashMap<String, Box<dyn CommandMetadata<Info>>>,
}

impl<Info> CommandList<Info>
where
  Info: Clone,
{
  pub fn new() -> Self {
    CommandList {
      commands: HashMap::new(),
    }
  }

  fn print_indent(&self, level: usize) {
    print!("{}", " ".repeat(4 + (level) * 11));
  }

  pub fn print_help(&self, level: usize) {
    for (_name, command) in &self.commands {
      self.print_indent(level);
      println!("{:10} {}", command.name().bold(), command.description());
      if command.help().len() > 0 {
        self.print_indent(level);
        println!("{:10} {}", "", command.help());
      }
    }
  }

  pub fn add_command<C>(&mut self, command: C)
  where
    C: CommandMetadata<Info> + Sized + 'static,
  {
    self
      .commands
      .insert(command.name().to_string(), Box::new(command));
  }

  pub fn find_command(&self, name: &str) -> Option<&Box<dyn CommandMetadata<Info>>> {
    self.commands.get(name)
  }
}
