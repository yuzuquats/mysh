use mysh::{command, CommandArg};
use serde::Deserialize;

use crate::UserInfo;

#[derive(CommandArg, Deserialize, Clone)]
pub struct Args {
  name: String,
}

#[command(
  name = "hello",
  description = "Prints hello world",
  long_description = "Prints hello world"
)]
pub async fn hello(_: UserInfo, args: Args) -> mysh::Result<()> {
  println!("Hello {}", args.name);
  Ok(())
}
