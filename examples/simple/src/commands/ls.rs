use std::fs;

use anyhow::anyhow;
use mysh::{CommandArg, command};
use serde::{Deserialize, Serialize};

use crate::UserInfo;

#[derive(CommandArg, Serialize, Deserialize, Debug, Clone)]
pub struct Args {}

#[command(
  name = "ls",
  description = "list directory contents",
  long_description = "list directory contents"
)]
pub async fn ls(_: UserInfo, _: Args) -> mysh::Result<()> {
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
