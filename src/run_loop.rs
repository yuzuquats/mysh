use crate::command_list::CommandList;
use crate::error::Error;
use crate::shell::Callable;
use crate::tokenizer::IntoArgs;
use anyhow::anyhow;
use colored::Colorize;
use reedline::{Prompt, Reedline, Signal};
use std::collections::HashMap;
use std::env;

pub async fn run<Info>(
  info: Info,
  commands: CommandList<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  prompt: &dyn Prompt,
) where
  Info: Clone,
{
  if let Err(e) = run_once_or_loop(&info, commands, subcommands, prompt).await {
    println!("{} {}", "[Error]".red(), e);
  }
}

async fn run_once_or_loop<Info>(
  info: &Info,
  commands: CommandList<Info>,
  subcommands: HashMap<String, Box<dyn Callable>>,
  prompt: &dyn Prompt,
) -> Result<(), Error>
where
  Info: Clone,
{
  let mut argv: Vec<String> = env::args().collect();
  if argv.len() > 1 {
    argv.remove(0);
    exec(info, &commands, &subcommands, argv).await?;
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

        match exec(info, &commands, &subcommands, argv).await {
          Ok(_) => {}
          Err(e) => println!("{} {}", "[Error]".red(), e),
        }
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

async fn exec<Info: Clone>(
  info: &Info,
  commands: &CommandList<Info>,
  subcommands: &HashMap<String, Box<dyn Callable>>,
  argv: Vec<String>,
) -> crate::Result<()> {
  let name = &argv.get(0).expect("").clone();
  if name == "help" {
    let Some(help_arg) = argv.get(1) else {
      print_help(commands, subcommands);
      return Ok(());
    };

    let command = commands
      .find_command(help_arg)
      .ok_or(anyhow!("Command not found: {help_arg}"))?;

    command.print_help();
    return Ok(());
  }

  if let Some(command) = commands.find_command(&name) {
    return command.call_with_argv(info.clone(), argv)?.await;
  }

  if let Some(subcommand) = subcommands.get(name) {
    return subcommand.call_with_argv(argv)?.await;
  }

  print_help(commands, subcommands);
  Ok(())
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
