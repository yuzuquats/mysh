use anyhow::anyhow;
use serde::de;

pub trait CommandArg {
  fn display_help() -> &'static str;
}

pub fn parse_command_arg<T>(argv: Vec<String>) -> Result<T, anyhow::Error>
where
  T: de::DeserializeOwned,
{
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
