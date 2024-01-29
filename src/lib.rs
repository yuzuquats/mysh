use crate::error::Error;
use crate::tokenizer::IntoArgs;
use anyhow::anyhow;
use colored::Colorize;
use command::CommandList;
use reedline::{Prompt, Reedline, Signal};
use std::env;

pub mod command;
pub mod command_arg;
pub mod error;
pub mod shell;
mod tokenizer;

pub extern crate mysh_derive;
pub use futures;
pub use mysh_derive::command;
pub use mysh_derive::*;

pub async fn run<Info>(info: Info, commands: CommandList<Info>, prompt: &dyn Prompt)
where
  Info: Clone,
{
  if let Err(e) = run_once_or_loop(&info, commands, prompt).await {
    println!("{} {}", "[Error]".red(), e);
  }
}

async fn run_once_or_loop<Info>(
  info: &Info,
  commands: CommandList<Info>,
  prompt: &dyn Prompt,
) -> Result<(), Error>
where
  Info: Clone,
{
  let mut argv: Vec<String> = env::args().collect();
  if argv.len() > 1 {
    argv.remove(0);
    commands.exec(info.clone(), argv).await?;
    return Ok(());
  }

  println!("argv: {:?}", argv);

  let mut line_editor = Reedline::create();
  loop {
    let sig = line_editor.read_line(prompt);

    match sig {
      Ok(Signal::Success(buffer)) => {
        let line = buffer.to_string();
        if line.len() == 0 {
          continue;
        }

        let argv = line
          .try_into_args()
          .map_err(|e| anyhow!("arg parse error >> {e}"))?;

        let r = commands.exec(info.clone(), argv).await;
        println!("command result: {r:?}");
      }
      Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
        println!("\nAborted!");
        break;
      }
      x => {
        println!("Event: {:?}", x);
      }
    }
  }

  return Ok(());
}
