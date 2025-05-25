use crate::error::Error;
use serde::de;
use uuid::Uuid;

pub trait CommandArg {
  fn display_help() -> Vec<String>;
}

impl CommandArg for String {
  fn display_help() -> Vec<String> {
    vec!["[string]".to_string()]
  }
}

impl CommandArg for i64 {
  fn display_help() -> Vec<String> {
    vec!["[i64]".to_string()]
  }
}

impl CommandArg for i32 {
  fn display_help() -> Vec<String> {
    vec!["[i32]".to_string()]
  }
}

impl CommandArg for f32 {
  fn display_help() -> Vec<String> {
    vec!["[f32]".to_string()]
  }
}

impl CommandArg for Uuid {
  fn display_help() -> Vec<String> {
    vec!["[uuid]".to_string()]
  }
}

impl<T> CommandArg for Option<T>
where
  T: CommandArg,
{
  fn display_help() -> Vec<String> {
    // @todo: properly format
    // let inner = T::display_help();
    // if inner.len() == 1 && let Some(first) = inner.get(0) && !first.starts_with("--") {
    //   return vec![format]
    // }
    let help = &mut T::display_help();
    if help.len() == 0 {
      return vec![];
    }
    if help.len() == 1 {
      let opt = help.get(0).unwrap();
      return vec![format!("Option<{opt}>")];
    }

    let mut opt = vec!["<Optional>".to_string()];
    opt.append(help);
    opt
  }
}

pub fn parse_arguments<T>(argv: Vec<String>) -> crate::Result<T>
where
  T: de::DeserializeOwned + CommandArg,
{
  // println!("--argv {:#?}", argv);

  if argv.len() == 1 {
    return Ok(
      serde_json::from_str("null")
        .map_err(|_| Error::ArgParseError("Missing arguments".to_string()))?,
    );
  }

  if argv.len() == 2 {
    let only = argv.get(1).expect("");

    // Deserializing directly implies primitive (i32, i64, etc)
    //
    if let Ok(only) = serde_json::from_str(&only) {
      return Ok(only);
    };

    let ser = serde_json::to_string(&only).map_err(|e| Error::Other(e.into()))?;
    return Ok(serde_json::from_str(&ser).map_err(|e| Error::Other(e.into()))?);
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
        return Err(Error::ArgParseError("option without param".to_string()));
      }
      key = Some(arg.to_owned());
    } else {
      let Some(unwrapped_key) = key else {
        return Err(Error::ArgParseError("param without option".to_string()));
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
  let ser = serde_json::to_string(&map).map_err(|e| Error::Other(e.into()))?;
  Ok(serde_json::from_str(&ser).map_err(|e| Error::Other(e.into()))?)
}
