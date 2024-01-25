# mysh-rs

mysh, short for "My Shell", is a rust library for quickly building small interactive shells.

## Usage

```toml
[dependencies]
mysh = "0.1.0"
futures = "0.3"
```

```rust
use mysh::command_arg::CommandArg;
use mysh::{command, CommandArg};
use serde::Deserialize;

#[derive(CommandArg, Deserialize, Clone)]
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
```

```rust
use mysh::shell::Shell;
use tokio;

#[tokio::main]
async fn main() {
  Shell::new(UserInfo {})
    .add_command(hello)
    .run()
    .await;
}
```

```bash
cargo run

>> hello --name World
hello World
```

## Run Examples

```bash
cargo run -p simple
```
