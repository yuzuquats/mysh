use crate::command::{CommandList, CommandMetadata};

pub struct Shell<Info>
where
  Info: Clone,
{
  info: Info,
  commands: CommandList<Info>,
}

impl<Info> Shell<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    Shell {
      info,
      commands: CommandList { commands: vec![] },
    }
  }

  pub fn add_command<C>(mut self, command: C) -> Self
  where
    C: CommandMetadata<Info> + Sized + 'static,
  {
    self.commands.commands.push(Box::new(command));
    self
  }

  pub async fn run(self) {
    crate::run(self.info, self.commands).await;
  }
}
