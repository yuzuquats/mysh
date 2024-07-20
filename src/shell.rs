use std::{
  borrow::Cow,
  cell::RefCell,
  collections::HashMap,
  env,
  ops::Deref,
  sync::{Arc, RwLock},
};

use anyhow::{anyhow, Context};
use futures::Future;
use reedline::{
  DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Prompt, PromptHistorySearchStatus,
  Reedline,
};

use crate::{command_list::CommandList, command_metadata::CommandMetadata, run_loop::LineReader};

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
  linereader: Option<Box<dyn LineReader>>,
}

impl<Info> Shell<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    Shell {
      info,
      commands: CommandList::new(),
      linereader: None,
      subcommands: HashMap::new(),
    }
  }

  pub fn set_line_reader<P>(mut self, linereader: P) -> Self
  where
    P: LineReader + Sized + 'static,
  {
    self.linereader = Some(Box::new(linereader));
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
      &mut (*self
        .linereader
        .unwrap_or_else(|| Box::new(DefaultLineReader::new()))),
    )
    .await;
  }
}

#[derive(Clone)]
pub struct PromptText(Arc<RwLock<Option<String>>>);

impl PromptText {
  pub fn new() -> PromptText {
    PromptText(Arc::new(RwLock::new(None)))
  }

  pub fn set(&self, t: Option<String>) {
    *self.0.write().expect("") = t;
  }

  pub fn render_as_reedline_prompt(&self) -> DefaultPrompt {
    DefaultPrompt {
      left_prompt: match self.0.read().expect("").clone() {
        Some(t) => DefaultPromptSegment::Basic(t),
        None => DefaultPromptSegment::Empty,
      },
      right_prompt: DefaultPromptSegment::Empty,
    }
  }
}

impl Prompt for PromptText {
  fn render_prompt_left(&self) -> std::borrow::Cow<str> {
    Cow::Owned(self.0.read().expect("").clone().unwrap_or("".to_string()))
  }

  fn render_prompt_right(&self) -> std::borrow::Cow<str> {
    Cow::Borrowed("")
  }

  fn render_prompt_indicator(
    &self,
    prompt_mode: reedline::PromptEditMode,
  ) -> std::borrow::Cow<str> {
    Cow::Borrowed(" || ")
  }

  fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
    Cow::Borrowed("::: ")
  }

  fn render_prompt_history_search_indicator(
    &self,
    history_search: reedline::PromptHistorySearch,
  ) -> std::borrow::Cow<str> {
    let prefix = match history_search.status {
      PromptHistorySearchStatus::Passing => "",
      PromptHistorySearchStatus::Failing => "failing ",
    };
    // NOTE: magic strings, given there is logic on how these compose I am not sure if it
    // is worth extracting in to static constant
    Cow::Owned(format!(
      "({}reverse-search: {}) ",
      prefix, history_search.term
    ))
  }
}

pub struct DefaultLineReader {
  reedline: Reedline,
  pub prompt: PromptText,
}

impl DefaultLineReader {
  pub fn new() -> DefaultLineReader {
    let path = env::current_dir().expect("");
    println!("The current directory is {}", path.display());

    let history = Box::new(
      FileBackedHistory::with_file(100, "history.txt".into())
        .expect("Error configuring history with file"),
    );
    let reedline = Reedline::create()
      // .with_completer(completer)
      // .with_partial_completions(partial_completions)
      .with_history(history);

    DefaultLineReader {
      reedline,
      prompt: PromptText::new(),
    }
  }
}

impl LineReader for DefaultLineReader {
  fn read_line(&mut self) -> anyhow::Result<reedline::Signal> {
    self.reedline.read_line(&self.prompt).context("")
  }
}
