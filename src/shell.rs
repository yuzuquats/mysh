use std::{
  borrow::Cow,
  collections::HashMap,
  sync::{Arc, RwLock},
};

use crate::error::Error;
use anyhow::Context;
use futures::Future;
use reedline::{
  DefaultPrompt, DefaultPromptSegment, ExternalPrinter, FileBackedHistory, Prompt,
  PromptHistorySearchStatus, Reedline,
};
use serde_json::Value;

use crate::{command_list::CommandList, command_metadata::CommandMetadata, run_loop::LineReader};

pub trait Callable {
  fn call_with_argv(
    &self,
    argv: Vec<String>,
  ) -> crate::Result<std::pin::Pin<Box<dyn Future<Output = crate::Result<Value>>>>>;

  fn print_help(&self, include_args: bool);
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

  pub fn to_shell(self) -> Shell<Info> {
    Shell::new_with_root_scripts(self)
  }

  pub async fn run(self) {
    crate::run_loop::run(self, HashMap::new(), &mut DefaultLineReader::new()).await;
  }

  pub async fn run_with(self) {
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

    let subcommand_name = argv
      .get(0)
      .ok_or_else(|| Error::MissingSubcommand(self.commands.names().join(", ")))?;
    let subcommand = self
      .commands
      .find_command(&subcommand_name)
      .ok_or(Error::NoSuchSubcommand)?;

    Ok(subcommand.call_with_argv(self.info.clone(), argv)?.await?)
  }
}

impl<T: Clone> Callable for Scripts<T> {
  fn call_with_argv(
    &self,
    argv: Vec<String>,
  ) -> crate::Result<std::pin::Pin<Box<dyn Future<Output = crate::Result<Value>>>>> {
    let subcommand_name = argv
      .get(1)
      .ok_or_else(|| Error::MissingSubcommand(self.commands.names().join(", ")))?;
    let subcommand = self
      .commands
      .find_command(&subcommand_name)
      .ok_or(Error::NoSuchSubcommand)?;
    let mut argv = argv.clone();
    argv.remove(0);
    subcommand.call_with_argv(self.info.clone(), argv)
  }

  fn print_help(&self, include_args: bool) {
    self.commands.print_help(1, include_args);
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

  pub fn new_with_root_scripts(root_scripts: Scripts<Info>) -> Self {
    Shell {
      root_scripts,
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

  pub async fn run_with(self, external_printer: ExternalPrinter<String>) {
    crate::run_loop::run(
      self.root_scripts,
      self.subcommands,
      &mut (*self
        .linereader
        .unwrap_or_else(|| Box::new(DefaultLineReader::new_with(Some(external_printer))))),
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
  fn render_prompt_left(&'_ self) -> std::borrow::Cow<'_, str> {
    Cow::Owned(self.0.read().expect("").clone().unwrap_or("".to_string()))
  }

  fn render_prompt_right(&'_ self) -> std::borrow::Cow<'_, str> {
    Cow::Borrowed("")
  }

  fn render_prompt_indicator(
    &'_ self,
    _prompt_mode: reedline::PromptEditMode,
  ) -> std::borrow::Cow<'_, str> {
    Cow::Borrowed(" → ")
  }

  fn render_prompt_multiline_indicator(&'_ self) -> std::borrow::Cow<'_, str> {
    Cow::Borrowed("::: ")
  }

  fn render_prompt_history_search_indicator(
    &'_ self,
    history_search: reedline::PromptHistorySearch,
  ) -> std::borrow::Cow<'_, str> {
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
  pub(crate) reedline: Reedline,
  pub(crate) printer: Option<ExternalPrinter<String>>,
  pub prompt: PromptText,
}

impl DefaultLineReader {
  pub fn new() -> Self {
    Self::new_with(None)
  }

  pub fn new_with(external_printer: Option<ExternalPrinter<String>>) -> Self {
    let history = Box::new(
      FileBackedHistory::with_file(100, "history.txt".into())
        .expect("Error configuring history with file"),
    );
    let mut reedline = Reedline::create().with_history(history);
    reedline = if let Some(external_printer) = external_printer.clone() {
      reedline.with_external_printer(external_printer)
    } else {
      reedline
    };
    DefaultLineReader {
      reedline,
      printer: external_printer,
      prompt: PromptText::new(),
    }
  }
}

impl LineReader for DefaultLineReader {
  fn read_line(&mut self) -> anyhow::Result<reedline::Signal> {
    self.reedline.read_line(&self.prompt).context("")
  }

  fn external_printer(&self) -> Option<ExternalPrinter<String>> {
    self.printer.clone()
  }
}
