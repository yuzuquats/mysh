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
    print!("{}", " ".repeat(4 + (level) * 13));
  }

  pub fn print_help(&self, level: usize) {
    let mut keys: Vec<_> = self.commands.keys().collect();
    keys.sort();

    for key in keys {
      let command = self.commands.get(key).expect("");
      self.print_indent(level);
      println!("{:12} {}", command.name().bold(), command.description());
      for help in command.help() {
        if help.len() > 0 {
          self.print_indent(level);
          println!("{:12} {}", "", help);
        }
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
