use crate::error::Error;
use crate::tokenizer::IntoArgs;
use anyhow::anyhow;
use colored::Colorize;
use command::CommandList;
use ctrlc;
use std::env;
use std::io::Write;
use std::process::exit;

pub mod command;
pub mod command_arg;
pub mod error;
pub mod shell;
mod tokenizer;

pub extern crate mysh_derive;
pub use mysh_derive::command;
pub use mysh_derive::*;

pub async fn run<Info>(info: Info, commands: CommandList<Info>)
where
  Info: Clone,
{
  if let Err(e) = run_once_or_loop(&info, commands).await {
    println!("{} {}", "[Error]".red(), e);
  }
}

async fn run_once_or_loop<Info>(info: &Info, commands: CommandList<Info>) -> Result<(), Error>
where
  Info: Clone,
{
  let mut argv: Vec<String> = env::args().collect();
  if argv.len() > 2 {
    argv.remove(0);
    commands.exec(info, argv).await?;
    return Ok(());
  }

  ctrlc::set_handler(|| exit(0)).expect("couldn't set ctrlc handler");
  loop {
    print!(">> ");
    if let Err(e) = loop_once(info, &commands).await {
      println!("{} {}", "[Error]".red(), e);
    }
  }
}

async fn loop_once<Info>(info: &Info, commands: &CommandList<Info>) -> Result<(), Error>
where
  Info: Clone,
{
  std::io::stdout()
    .flush()
    .map_err(|e| anyhow!("Could not flush stdout >> {e}"))?;

  let mut line = String::new();
  std::io::stdin()
    .read_line(&mut line)
    .map_err(|e| anyhow!("unable to read from stdin >> {e}"))?;

  let argv = line
    .try_into_args()
    .map_err(|e| anyhow!("arg parse error >> {e}"))?;

  commands.exec(info, argv).await?;
  Ok(())
}
