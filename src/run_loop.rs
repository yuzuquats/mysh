use anyhow::anyhow;
use colored::Colorize;
use reedline::Signal;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use tracing::{error, info};

use crate::command_list::CommandList;
use crate::error::Error;
use crate::shell::Callable;
use crate::tokenizer::IntoArgs;
use crate::Scripts;

pub trait LineReader {
  fn read_line(&mut self) -> anyhow::Result<Signal>;
}

pub async fn run<Info>(
  scripts: Scripts<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  line_reader: &mut (impl LineReader + ?Sized),
) where
  Info: Clone,
{
  if let Err(e) = run_once_or_loop(&scripts, subcommands, line_reader).await {
    error!("{}", e);
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
    argv.remove(0);
    exec(&scripts, &subcommands, argv).await?;
    return Ok(());
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
          .map_err(|e| anyhow!("arg parse error >> {e}"))?;

        match exec(&scripts, &subcommands, argv).await {
          Ok(_) => {}
          Err(e) => error!("{}", e),
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
    let Some(help_arg) = argv.get(1) else {
      print_help(&scripts.commands, subcommands);
      return Ok(().into());
    };

    let command = scripts
      .commands
      .find_command(help_arg)
      .ok_or(anyhow!("Command not found: {help_arg}"))?;

    command.print_help();
    return Ok(().into());
  }

  if let Some(command) = scripts.commands.find_command(&name) {
    return command.call_with_argv(scripts.info.clone(), argv)?.await;
  }

  if let Some(subcommand) = subcommands.get(name) {
    return subcommand.call_with_argv(argv)?.await;
  }

  print_help(&scripts.commands, subcommands);
  Ok(().into())
}

pub fn print_help<Info: Clone>(
  commands: &CommandList<Info>,
  subcommands: &HashMap<String, Box<dyn Callable>>,
) {
  println!("\nUsage: [name] [command]\n");
  println!("Commands:");
  commands.print_help(0);
  for (subcommand_name, subcommand) in subcommands {
    println!("    {}", subcommand_name.bold());
    subcommand.print_help();
  }
  println!("");
}
