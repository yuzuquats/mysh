use mysh::command_arg::CommandArg;
use mysh::{command, CommandArg};
use serde::{Deserialize, Serialize};

use crate::UserInfo;

#[derive(CommandArg, Serialize, Deserialize, Debug, Clone)]
pub struct Args {
  name: String,
}

#[command(
  name = "hello",
  description = "Prints hello world",
  long_description = "Prints hello world"
)]
pub async fn hello(_: &UserInfo, args: &Args) -> Result<(), mysh::error::Error> {
  println!("Hello {}", args.name);
  Ok(())
}
