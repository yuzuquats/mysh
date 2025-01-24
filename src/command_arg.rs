use anyhow::{anyhow, Context};
use serde::de;
use uuid::Uuid;

pub trait CommandArg {
  fn display_help() -> Vec<&'static str>;
}

impl CommandArg for String {
  fn display_help() -> Vec<&'static str> {
    vec!["[string]"]
  }
}

impl CommandArg for i64 {
  fn display_help() -> Vec<&'static str> {
    vec!["[i64]"]
  }
}

impl CommandArg for i32 {
  fn display_help() -> Vec<&'static str> {
    vec!["[i32]"]
  }
}

impl CommandArg for f32 {
  fn display_help() -> Vec<&'static str> {
    vec!["[f32]"]
  }
}

impl CommandArg for Uuid {
  fn display_help() -> Vec<&'static str> {
    vec!["[uuid]"]
  }
}

impl<T> CommandArg for Option<T>
where
  T: CommandArg,
{
  fn display_help() -> Vec<&'static str> {
    // @todo: properly format
    // let inner = T::display_help();
    // if inner.len() == 1 && let Some(first) = inner.get(0) && !first.starts_with("--") {
    //   return vec![format]
    // }
    let help = &mut T::display_help();
    if help.len() == 0 {
      return vec![];
    }

    let mut opt = vec!["<Optional>"];
    opt.append(help);
    opt
  }
}

pub fn parse_arguments<T>(argv: Vec<String>) -> Result<T, anyhow::Error>
where
  T: de::DeserializeOwned + CommandArg,
{
  // println!("--argv {:#?}", argv);

  if argv.len() == 1 {
    return Ok(serde_json::from_str("null").context("Missing arguments")?);
  }

  if argv.len() == 2 {
    let only = argv.get(1).expect("");

    // Deserializing directly implies primitive (i32, i64, etc)
    //
    if let Ok(only) = serde_json::from_str(&only) {
      return Ok(only);
    };

    let ser = serde_json::to_string(&only)?;
    return Ok(serde_json::from_str(&ser)?);
  }

  use serde_json::Map;
  let mut map: Map<String, serde_json::Value> = Map::new();
  let mut key: Option<String> = None;
  for (i, arg) in argv.iter().enumerate() {
    // println!("[]: {arg}");
    if i == 0 {
      continue;
    }

    if arg.starts_with("--") {
      let arg = arg.trim_start_matches("--");
      if key.is_some() {
        return Err(anyhow!("option without param"));
      }
      key = Some(arg.to_owned());
    } else {
      let Some(unwrapped_key) = key else {
        return Err(anyhow!("param without option"));
      };
      match arg.parse::<i64>() {
        Ok(n) => {
          map.insert(
            unwrapped_key,
            serde_json::Value::Number(serde_json::Number::from(n as i64)),
          );
        }
        Err(_) => {
          map.insert(unwrapped_key, serde_json::Value::String(arg.to_string()));
        }
      }
      key = None;
    }
  }
  let ser = serde_json::to_string(&map)?;
  Ok(serde_json::from_str(&ser)?)
}
