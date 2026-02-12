use mysh::{CommandArg, Scripts, Shell, command};
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
      Scripts::new(StatusInfo {})
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
pub async fn status_log(_: StatusInfo, _: Option<()>) -> mysh::Result<()> {
  println!("status_log");
  Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct MyJson {
  hi: String,
}

// pub struct status_log;
// #[allow(non_camel_case_types, missing_docs)]
// pub struct __status_log_future {
//   inner: std::pin::Pin<Box<dyn mysh::futures::Future<Output = mysh::Result<mysh::json::Value>>>>,
// }
// impl std::future::Future for __status_log_future {
//   type Output = mysh::Result<mysh::json::Value>;
//   fn poll(
//     mut self: std::pin::Pin<&mut Self>,
//     cx: &mut std::task::Context<'_>,
//   ) -> std::task::Poll<Self::Output> {
//     self.inner.as_mut().poll(cx)
//   }
// }
// impl status_log {
//   pub async fn call(_: StatusInfo, _: Option<()>) -> mysh::Result<MyJson> {
//     {
//       println!("status_log");
//       // ::std::io::_print(format_args!("status_log\n"));
//     };
//     Ok(MyJson {
//       hi: "blah".to_owned(),
//     })
//   }
//   fn future(info: StatusInfo, args: Option<()>) -> __status_log_future {
//     use anyhow::Context;
//     let inner = Box::pin(async move {
//       let r = status_log::call(info, args).await?;
//       Ok(mysh::json::to_value(r).context("Return value not json-able")?)
//     });
//     __status_log_future { inner }
//   }
// }
// impl std::ops::Deref for status_log {
//   type Target = (dyn Fn(StatusInfo, Option<()>) -> __status_log_future);
//   fn deref(&self) -> &Self::Target {
//     &Self::future
//   }
// }
// impl mysh::CommandMetadata<StatusInfo> for status_log {
//   fn name(&self) -> &'static str {
//     "log"
//   }
//   fn description(&self) -> &'static str {
//     "Logs the status"
//   }
//   fn long_description(&self) -> Option<&'static str> {
//     Some("Prints the status")
//   }
//   fn call_with_argv(
//     &self,
//     info: StatusInfo,
//     argv: Vec<String>,
//   ) -> mysh::Result<
//     std::pin::Pin<Box<dyn mysh::futures::Future<Output = mysh::Result<mysh::json::Value>>>>,
//   > {
//     use anyhow::Context;
//     let args = mysh::parse_arguments(argv)?;
//     Ok(Box::pin(async move {
//       let r = status_log::call(info, args).await?;
//       Ok(mysh::json::to_value(r).context("Return value not json-able")?)
//     }))
//   }
//   fn help(&self) -> Vec<&'static str> {
//     use mysh::CommandArg;
//     StatusArgs::display_help()
//   }
// }
