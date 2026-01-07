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

/// Parse a string value into the appropriate JSON type (bool, i64, or string)
fn parse_value(arg: &str) -> serde_json::Value {
  if arg == "true" || arg == "1" {
    serde_json::Value::Bool(true)
  } else if arg == "false" || arg == "0" {
    serde_json::Value::Bool(false)
  } else if let Ok(n) = arg.parse::<i64>() {
    serde_json::Value::Number(serde_json::Number::from(n))
  } else {
    serde_json::Value::String(arg.to_string())
  }
}

pub fn parse_arguments<T>(argv: Vec<String>) -> crate::Result<T>
where
  T: de::DeserializeOwned + CommandArg,
{
  // println!("--argv {:#?}", argv);

  if argv.len() == 1 {
    return Ok(serde_json::from_str("null").map_err(|_| {
      let expected_fields = T::display_help();
      if expected_fields.is_empty() {
        Error::ArgParseError("No arguments expected, but command failed".to_string())
      } else {
        let fields_list = expected_fields.join(", ");
        Error::ArgParseError(format!("Missing required arguments: {}", fields_list))
      }
    })?);
  }

  if argv.len() == 2 {
    let only = argv.get(1).expect("");

    // If it's a flag (starts with --), fall through to the map parsing logic
    if !only.starts_with("--") {
      // Deserializing directly implies primitive (i32, i64, etc)
      //
      if let Ok(only) = serde_json::from_str(&only) {
        return Ok(only);
      };

      let ser = serde_json::to_string(&only).map_err(|e| Error::Other(e.into()))?;
      return Ok(serde_json::from_str(&ser).map_err(|e| Error::Other(e.into()))?);
    }
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
      // If there's a pending key without a value, treat it as a boolean flag
      if let Some(pending_key) = key.take() {
        map.insert(pending_key, serde_json::Value::Bool(true));
      }
      // Check for --key=value syntax
      if let Some((k, v)) = arg.split_once('=') {
        let value = parse_value(v);
        map.insert(k.to_owned(), value);
      } else {
        key = Some(arg.to_owned());
      }
    } else {
      let Some(unwrapped_key) = key else {
        return Err(Error::ArgParseError("param without option".to_string()));
      };
      let value = parse_value(arg);
      map.insert(unwrapped_key, value);
      key = None;
    }
  }
  // If there's a trailing key without a value, treat it as a boolean flag
  if let Some(pending_key) = key {
    map.insert(pending_key, serde_json::Value::Bool(true));
  }
  let ser = serde_json::to_string(&map).map_err(|e| Error::Other(e.into()))?;
  Ok(serde_json::from_str(&ser).map_err(|e| {
    let expected_fields = T::display_help();
    let provided_fields: Vec<String> = map.keys().map(|k| format!("--{}", k)).collect();

    if expected_fields.is_empty() {
      Error::Other(e.into())
    } else {
      let expected_list = expected_fields.join(", ");
      let provided_list = provided_fields.join(", ");
      Error::ArgParseError(format!(
        "Failed to parse arguments.\nExpected: {}\nProvided: {}\nError: {}",
        expected_list, provided_list, e
      ))
    }
  })?)
}
