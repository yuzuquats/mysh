use std::{
  borrow::Cow,
  collections::HashMap,
  sync::{Arc, RwLock},
};

use anyhow::{anyhow, Context};
use futures::Future;
use reedline::{
  DefaultPrompt, DefaultPromptSegment, FileBackedHistory, Prompt, PromptHistorySearchStatus,
  Reedline,
};
use serde_json::Value;

use crate::{command_list::CommandList, command_metadata::CommandMetadata, run_loop::LineReader};

pub trait Callable {
  fn call_with_argv(
    &self,
    argv: Vec<String>,
  ) -> crate::Result<std::pin::Pin<Box<dyn Future<Output = crate::Result<Value>>>>>;

  fn print_help(&self);
}

pub struct Scripts<Info>
where
  Info: Clone,
{
  pub info: Info,
  pub commands: CommandList<Info>,
}

impl<Info> Scripts<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    Scripts {
      info,
      commands: CommandList::new(),
    }
  }

  pub async fn run(self) {
    crate::run_loop::run(self, HashMap::new(), &mut DefaultLineReader::new()).await;
  }

  pub fn add_command<C>(mut self, command: C) -> Self
  where
    C: CommandMetadata<Info> + Sized + 'static,
  {
    self.commands.add_command(command);
    self
  }

  pub async fn run_command(&self, command: &str) -> crate::Result<crate::json::Value> {
    let argv = command
      .split(" ")
      .map(|s| s.to_string())
      .collect::<Vec<String>>();

    let subcommand_name = argv.get(0).ok_or(anyhow!("Command name not found"))?;
    let subcommand = self.commands.find_command(&subcommand_name).ok_or(anyhow!(
      "No such subcommand. ie. ./[bin] [command] [subcommand]"
    ))?;

    Ok(subcommand.call_with_argv(self.info.clone(), argv)?.await?)
  }
}

impl<T: Clone> Callable for Scripts<T> {
  fn call_with_argv(
    &self,
    argv: Vec<String>,
  ) -> crate::Result<std::pin::Pin<Box<dyn Future<Output = crate::Result<Value>>>>> {
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
  root_scripts: Scripts<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  linereader: Option<Box<dyn LineReader>>,
}

impl<Info> Shell<Info>
where
  Info: Clone,
{
  pub fn new(info: Info) -> Self {
    Shell {
      root_scripts: Scripts::new(info),
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
    self.root_scripts = self.root_scripts.add_command(command);
    self
  }

  pub fn add_subcommand<SubcommandInfo>(
    mut self,
    namespace: &str,
    commands: Scripts<SubcommandInfo>,
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
      self.root_scripts,
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
    _prompt_mode: reedline::PromptEditMode,
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
