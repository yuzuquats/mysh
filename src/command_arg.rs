use anyhow::anyhow;
use serde::de;

pub trait CommandArg {
  fn display_help() -> &'static str;
}

impl CommandArg for String {
  fn display_help() -> &'static str {
    "[string]"
  }
}

impl CommandArg for i64 {
  fn display_help() -> &'static str {
    "[i64]"
  }
}

impl CommandArg for i32 {
  fn display_help() -> &'static str {
    "[i32]"
  }
}

impl CommandArg for f32 {
  fn display_help() -> &'static str {
    "[f32]"
  }
}

pub fn parse_arguments<T>(argv: Vec<String>) -> Result<T, anyhow::Error>
where
  T: de::DeserializeOwned,
{
  if argv.len() == 2 {
    let only = argv.get(1).expect("");
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
  return Ok(serde_json::from_str(&ser)?);
}
