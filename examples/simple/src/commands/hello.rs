use mysh::{command, CommandArg};
use serde::Deserialize;

use crate::UserInfo;

#[derive(CommandArg, Deserialize, Clone, Debug)]
pub struct Args {
  name: String,
}

#[command(
  name = "hello",
  description = "Prints hello world",
  long_description = "Prints hello world"
)]
pub async fn hello(_: UserInfo, args: Option<Args>) -> mysh::Result<()> {
  println!("Hello {:#?}", args);

  Option::<Args>::display_help();
  Ok(())
}
