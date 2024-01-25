#![feature(async_closure)]
#![feature(associated_type_defaults)]

use mysh::shell::Shell;
use tokio;

mod commands;

#[derive(Clone)]
pub struct UserInfo {}

#[tokio::main]
async fn main() {
  Shell::new(UserInfo {})
    .add_command(commands::hello::hello)
    .add_command(commands::pwd::pwd)
    .run()
    .await;
}
