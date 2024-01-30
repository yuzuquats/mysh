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
    .add_command(commands::ls::ls)
    .run()
    .await;
}
