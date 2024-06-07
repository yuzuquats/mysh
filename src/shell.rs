use std::{collections::HashMap, ops::Deref};

use anyhow::anyhow;
use futures::Future;
use reedline::{DefaultPrompt, DefaultPromptSegment, Prompt};

use crate::{command_list::CommandList, command_metadata::CommandMetadata};

pub trait Callable {
  fn call_with_argv(
    &self,
    argv: Vec<String>,
  ) -> crate::Result<std::pin::Pin<Box<dyn Future<Output = crate::Result<()>>>>>;

  fn print_help(&self);
}

pub struct SubcommandList<Info>
where
  Info: Clone,
{
  info: Info,
  commands: CommandList<Info>,
}

impl<Info> SubcommandList<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    SubcommandList {
      info,
      commands: CommandList::new(),
    }
  }

  pub fn add_command<C>(mut self, command: C) -> Self
  where
    C: CommandMetadata<Info> + Sized + 'static,
  {
    self.commands.add_command(command);
    self
  }
}

impl<T: Clone> Callable for SubcommandList<T> {
  fn call_with_argv(
    &self,
    argv: Vec<String>,
  ) -> crate::Result<std::pin::Pin<Box<dyn Future<Output = crate::Result<()>>>>> {
    let subcommand_name = argv.get(1).ok_or(anyhow!(
      "Please provide a subcommand. ie. ./[bin] [command] [subcommand]"
    ))?;
    let subcommand = self.commands.find_command(&subcommand_name).ok_or(anyhow!(
      "No such subcommand. ie. ./[bin] [command] [subcommand]"
    ))?;
    let mut argv = argv.clone();
    argv.remove(0);
    subcommand.call_with_argv(self.info.clone(), argv)
  }

  fn print_help(&self) {
    self.commands.print_help(1);
  }
}

pub struct Shell<Info>
where
  Info: Clone,
{
  info: Info,
  commands: CommandList<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  prompt: Box<dyn Prompt>,
}

impl<Info> Shell<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    let prompt = DefaultPrompt {
      left_prompt: DefaultPromptSegment::Empty,
      right_prompt: DefaultPromptSegment::Empty,
    };
    Shell {
      info,
      commands: CommandList::new(),
      prompt: Box::new(prompt),
      subcommands: HashMap::new(),
    }
  }

  pub fn set_prompt<P>(mut self, prompt: P) -> Self
  where
    P: Prompt + Sized + 'static,
  {
    self.prompt = Box::new(prompt);
    self
  }

  pub fn add_command<C>(mut self, command: C) -> Self
  where
    C: CommandMetadata<Info> + Sized + 'static,
  {
    self.commands.add_command(command);
    self
  }

  pub fn add_subcommand<SubcommandInfo>(
    mut self,
    namespace: &str,
    commands: SubcommandList<SubcommandInfo>,
  ) -> Self
  where
    SubcommandInfo: Clone + 'static,
  {
    self
      .subcommands
      .insert(namespace.to_string(), Box::new(commands));
    self
  }

  pub async fn run(self) {
    crate::run_loop::run(
      self.info,
      self.commands,
      self.subcommands,
      self.prompt.deref(),
    )
    .await;
  }
}
