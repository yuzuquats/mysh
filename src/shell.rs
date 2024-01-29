use std::ops::Deref;

use reedline::{DefaultPrompt, DefaultPromptSegment, Prompt};

use crate::command::{CommandList, CommandMetadata};

pub struct Shell<Info>
where
  Info: Clone,
{
  info: Info,
  commands: CommandList<Info>,
  prompt: Box<dyn Prompt>,
}

impl<Info> Shell<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    let prompt = DefaultPrompt {
      left_prompt: DefaultPromptSegment::Empty,
      right_prompt: DefaultPromptSegment::CurrentDateTime,
    };
    Shell {
      info,
      commands: CommandList { commands: vec![] },
      prompt: Box::new(prompt),
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
    self.commands.commands.push(Box::new(command));
    self
  }

  pub async fn run(self) {
    crate::run_loop::run(self.info, self.commands, self.prompt.deref()).await;
  }
}
