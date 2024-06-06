use mysh::{command, CommandArg, Shell, SubcommandList};
use serde::{Deserialize, Serialize};
use tokio;

mod commands;

#[derive(Clone)]
pub struct UserInfo {}

#[derive(Clone)]
pub struct StatusInfo {}

#[tokio::main]
async fn main() {
  Shell::new(UserInfo {})
    .add_command(commands::hello::hello)
    .add_command(commands::pwd::pwd)
    .add_command(commands::ls::ls)
    .add_subcommand(
      "status",
      SubcommandList::new(StatusInfo {})
        .add_command(status_print)
        .add_command(status_log),
    )
    .run()
    .await;
}

#[derive(CommandArg, Serialize, Deserialize, Debug, Clone)]
pub struct StatusArgs {}

#[command(
  name = "print",
  description = "Prints the status",
  long_description = "Prints the status"
)]
pub async fn status_print(_: StatusInfo, _: StatusArgs) -> mysh::Result<()> {
  println!("status_print");
  Ok(())
}

#[command(
  name = "log",
  description = "Logs the status",
  long_description = "Prints the status"
)]
pub async fn status_log(_: StatusInfo, _: StatusArgs) -> mysh::Result<()> {
  println!("status_log");
  Ok(())
}
