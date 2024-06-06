use std::env;

use anyhow::anyhow;
use mysh::{command, CommandArg};
use serde::{Deserialize, Serialize};

use crate::UserInfo;

#[derive(CommandArg, Serialize, Deserialize, Debug, Clone)]
pub struct Args {}

#[command(
  name = "pwd",
  description = "return working directory name",
  long_description = "The pwd utility writes the absolute pathname of the current working directory to the standard output.

Some shells may provide a builtin pwd command which is similar or identical to this utility. Consult the builtin(1) manual page."
)]
pub async fn pwd(_: UserInfo, _: Args) -> mysh::Result<()> {
  let current_dir = env::current_dir().map_err(|e| anyhow!("current_dir >> {e}"))?;
  println!("{}", current_dir.to_str().expect(""));
  Ok(())
}
