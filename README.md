# mysh-rs

mysh, short for "My Shell", is a rust library for quickly building small
interactive shells.

## Usage

```toml
[dependencies]
mysh = "0.1.1"
futures = "0.3"
```

```rust
use mysh::{command, CommandArg};
use serde::Deserialize;

#[derive(CommandArg, Deserialize, Clone)]
pub struct Args {
  name: String,
}

#[command(
  name = "hello",
  description = "Prints hello world",
  long_description = "Prints hello world" // optional
)]
pub async fn hello(_: UserInfo, args: Args) -> mysh::Result<()> {
  println!("Hello {}", args.name);
  Ok(())
}
```

```rust
use mysh::Shell;
use tokio;

#[derive(Clone)]
pub struct UserInfo {}

// #[tokio::main] // or
#[actix_rt::main]
async fn main() {
  Shell::new(UserInfo {})
    .add_command(hello)
    .run()
    .await;
}
```

### Trigger read-eval-print-loop

```bash
cargo run

>> hello --name World
Hello World
```

### Run single command

```bash
cargo run -- hello --name World
Hello World
```

## Run Examples

```bash
cargo run -p simple
```
