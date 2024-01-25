use core::mem;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError;

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    f.write_str("missing closing quote")
  }
}

impl std::error::Error for ParseError {}

enum State {
  /// Within a delimiter.
  Delimiter,
  /// After backslash, but before starting word.
  Backslash,
  /// Within an unquoted word.
  Unquoted,
  /// After backslash in an unquoted word.
  UnquotedBackslash,
  /// Within a single quoted word.
  SingleQuoted,
  /// Within a double quoted word.
  DoubleQuoted,
  /// After backslash inside a double quoted word.
  DoubleQuotedBackslash,
}

pub(crate) trait IntoArgs {
  fn try_into_args(&self) -> Result<Vec<String>, ParseError>;
}

impl<S: std::ops::Deref<Target = str>> IntoArgs for S {
  fn try_into_args(&self) -> Result<Vec<String>, ParseError> {
    use State::*;

    let mut words = Vec::new();
    let mut word = String::new();
    let mut chars = self.chars();
    let mut state = Delimiter;

    loop {
      let c = chars.next();
      state = match state {
        Delimiter => match c {
          None => break,
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => Backslash,
          Some('\t') | Some(' ') | Some('\n') => Delimiter,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Backslash => match c {
          None => {
            word.push('\\');
            words.push(mem::take(&mut word));
            break;
          }
          Some('\n') => Delimiter,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        Unquoted => match c {
          None => {
            words.push(mem::take(&mut word));
            break;
          }
          Some('\'') => SingleQuoted,
          Some('\"') => DoubleQuoted,
          Some('\\') => UnquotedBackslash,
          Some('\t') | Some(' ') | Some('\n') => {
            words.push(mem::take(&mut word));
            Delimiter
          }
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        UnquotedBackslash => match c {
          None => {
            word.push('\\');
            words.push(mem::take(&mut word));
            break;
          }
          Some('\n') => Unquoted,
          Some(c) => {
            word.push(c);
            Unquoted
          }
        },
        SingleQuoted => match c {
          None => return Err(ParseError),
          Some('\'') => Unquoted,
          Some(c) => {
            word.push(c);
            SingleQuoted
          }
        },
        DoubleQuoted => match c {
          None => return Err(ParseError),
          Some('\"') => Unquoted,
          Some('\\') => DoubleQuotedBackslash,
          Some(c) => {
            word.push(c);
            DoubleQuoted
          }
        },
        DoubleQuotedBackslash => match c {
          None => return Err(ParseError),
          Some('\n') => DoubleQuoted,
          Some(c @ '$') | Some(c @ '`') | Some(c @ '"') | Some(c @ '\\') => {
            word.push(c);
            DoubleQuoted
          }
          Some(c) => {
            word.push('\\');
            word.push(c);
            DoubleQuoted
          }
        },
      }
    }

    Ok(words)
  }
}
