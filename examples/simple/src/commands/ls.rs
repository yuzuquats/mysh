use std::fs;

use anyhow::anyhow;
use mysh::command_arg::CommandArg;
use mysh::{command, CommandArg};
use serde::{Deserialize, Serialize};

use crate::UserInfo;

#[derive(CommandArg, Serialize, Deserialize, Debug, Clone)]
pub struct Args {}

#[command(
  name = "ls",
  description = "list directory contents",
  long_description = "list directory contents"
)]
pub async fn ls(_: &UserInfo, _: &Args) -> Result<(), mysh::error::Error> {
  for path in fs::read_dir(".").map_err(|e| anyhow!("read_dir failed >> {e}"))? {
    match path {
      Ok(p) => {
        println!("{}", p.path().display())
      }
      Err(e) => {
        println!("Err: {}", e)
      }
    };
  }
  Ok(())
}
