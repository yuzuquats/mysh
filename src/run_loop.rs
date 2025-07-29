use colored::Colorize;
use futures::FutureExt;
use reedline::{ExternalPrinter, Signal};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::panic::AssertUnwindSafe;
use tracing::{error, info};

use crate::Scripts;
use crate::command_list::CommandList;
use crate::error::{Error, ToTrace};
use crate::shell::Callable;
use crate::tokenizer::IntoArgs;

pub trait LineReader {
  fn read_line(&mut self) -> anyhow::Result<Signal>;
  fn external_printer(&self) -> Option<ExternalPrinter<String>>;
}

pub async fn run<Info>(
  scripts: Scripts<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  line_reader: &mut (impl LineReader + ?Sized),
) where
  Info: Clone,
{
  if let Err(e) = run_once_or_loop(&scripts, subcommands, line_reader).await {
    let panic_with_trace_ser =
      serde_json::to_string(&e.to_trace()).expect("trace couldn't serialize");
    error!(
      exception.trace_json = panic_with_trace_ser,
      "Command Failed"
    );
  }
}

async fn run_once_or_loop<Info>(
  scripts: &Scripts<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  line_reader: &mut (impl LineReader + ?Sized),
) -> Result<(), Error>
where
  Info: Clone,
{
  let mut argv: Vec<String> = env::args().collect();
  if argv.len() > 1 {
    // manually set up the printer because we're not using reedline
    if let Some(external_printer) = line_reader.external_printer() {
      let rx = external_printer.receiver().clone();
      tokio::spawn(async move {
        while let Ok(s) = rx.recv() {
          print!("{}", s);
        }
      });
    }

    argv.remove(0);

    // In CLI mode, catch panics and exit with error code
    let result = AssertUnwindSafe(exec(&scripts, &subcommands, argv))
      .catch_unwind()
      .await;

    match result {
      Ok(Ok(_)) => return Ok(()),
      Ok(Err(e)) => return Err(e),
      Err(_) => {
        // A panic occurred - exit with error code
        std::process::exit(1);
      }
    }
  }

  // println!("argv: {:?}", argv);

  loop {
    let sig = line_reader.read_line();

    match sig {
      Ok(Signal::Success(buffer)) => {
        let line = buffer.to_string();
        if line.len() == 0 {
          continue;
        }

        let argv = line
          .try_into_args()
          .map_err(|e| Error::ArgParseError(e.to_string()))?;

        // In interactive mode, catch panics and log them but continue
        let result = AssertUnwindSafe(exec(&scripts, &subcommands, argv))
          .catch_unwind()
          .await;

        match result {
          Ok(Ok(_)) => {}
          Ok(Err(e)) => {
            let panic_with_trace_ser =
              serde_json::to_string(&e.to_trace()).expect("trace couldn't serialize");
            error!(
              exception.trace_json = panic_with_trace_ser,
              "Command Failed"
            );
          }
          Err(_) => {
            error!("Command panicked!");
          }
        }
      }
      Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
        info!("Exiting");
        break;
      }
      x => {
        error!("Event: {:?}", x);
      }
    }
  }

  return Ok(());
}

async fn exec<Info: Clone>(
  scripts: &Scripts<Info>,
  subcommands: &HashMap<String, Box<dyn Callable>>,
  argv: Vec<String>,
) -> crate::Result<Value> {
  let name = &argv.get(0).expect("").clone();
  if name == "help" {
    let include_args = argv.iter().any(|s| s == "--args");
    let command = argv.iter().find(|a| *a != "--args" && *a != "help");
    let Some(help_arg) = command else {
      print_help(&scripts.commands, subcommands, include_args);
      return Ok(().into());
    };

    let command = scripts
      .commands
      .find_command(help_arg)
      .ok_or(Error::CommandNotFound(help_arg.clone()))?;

    command.print_help();
    return Ok(().into());
  }

  if let Some(command) = scripts.commands.find_command(&name) {
    return command.call_with_argv(scripts.info.clone(), argv)?.await;
  }

  if let Some(subcommand) = subcommands.get(name) {
    return subcommand.call_with_argv(argv)?.await;
  }

  let include_args = argv.iter().any(|s| s == "--args");
  print_help(&scripts.commands, subcommands, include_args);
  Ok(().into())
}

pub fn print_help<Info: Clone>(
  commands: &CommandList<Info>,
  subcommands: &HashMap<String, Box<dyn Callable>>,
  include_args: bool,
) {
  println!("\nUsage: [name] [command]\n");
  println!("Commands:");
  commands.print_help(0, include_args);
  for (subcommand_name, subcommand) in subcommands {
    println!("    {}", subcommand_name.bold());
    subcommand.print_help(include_args);
  }
  println!("");
}
