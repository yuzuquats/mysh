use std::{backtrace::Backtrace, ops::Coroutine, panic::PanicHookInfo};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static FRAME_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r#"\{\s*fn:\s*"([^"]+)",\s*file:\s*"([^"]+)",\s*line:\s*(\d+)\s*\},"#).unwrap()
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExceptionWithTrace {
  pub message: Option<String>,
  pub sources: Vec<String>,
  pub frames: Vec<FrameSymbol>,
  pub filtered_range: (Option<String>, Option<String>),
}

impl ExceptionWithTrace {
  pub fn new<'a>(message: Option<String>, trace: Option<&Backtrace>) -> Self {
    ExceptionWithTrace {
      message,
      sources: Vec::new(),
      frames: trace.map_or_else(Vec::new, |t| ExceptionWithTrace::parse_frames(t)),
      filtered_range: (None, None),
    }
  }

  pub fn with_sources<'a>(
    message: Option<String>,
    sources: Vec<String>,
    trace: Option<&Backtrace>,
  ) -> Self {
    ExceptionWithTrace {
      message,
      sources,
      frames: trace.map_or_else(Vec::new, |t| ExceptionWithTrace::parse_frames(t)),
      filtered_range: (None, None),
    }
  }

  pub fn parse_message(info: &PanicHookInfo<'_>) -> Option<String> {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
      println!("{s}");
      s.splitn(2, "\n").next().map(|s| s.to_string())
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
      s.splitn(2, "\n").next().map(|s| s.to_string())
    } else {
      None
    }
  }

  fn parse_frames<'a>(trace: &'a Backtrace) -> Vec<FrameSymbol> {
    // NOTE:
    // backtrace is not stabilized and provides no methods to query the
    // underlying file, line, lineo. The debug format is "json-like" but
    // not quite...
    //
    let trace_dbg = format!("{:#?}", &trace);
    trace_dbg
      .trim_start_matches("Backtrace")
      .split("\n")
      .into_iter()
      .flat_map(|l| {
        let Some(captures) = FRAME_REGEX.captures(l) else {
          return None;
        };
        let func = captures
          .get(1)
          .map(|m| m.as_str().to_owned())
          .expect("no line");
        let file = captures.get(2).map(|m| m.as_str().to_owned());
        let line = captures
          .get(3)
          .map(|m| m.as_str().parse::<u32>().ok())
          .flatten();

        Some(FrameSymbol { func, file, line })
      })
      .collect::<Vec<FrameSymbol>>()
  }

  pub fn filtered_frames<'a>(
    &'a self,
  ) -> impl Coroutine<Yield = (usize, &'a FrameSymbol), Return = ()> + use<'a> {
    #[coroutine]
    static move || {
      let mut saw_start = self.filtered_range.0.is_none();

      let t = self;

      for (idx, line) in t.frames.iter().enumerate() {
        if !saw_start {
          if let Some(start) = &t.filtered_range.0 {
            if line.func == *start {
              saw_start = true;
            }
          }
          continue;
        }

        if let Some(end) = &t.filtered_range.1 {
          if line.func.starts_with(end) {
            break;
          }
        }

        match line.func.as_ref() {
          "core::panicking::panic_fmt"
          | "core::result::unwrap_failed"
          | "core::result::Result<T,E>::expect" => continue,
          _ => {
            yield (idx, line);
          }
        }
      }
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameSymbol {
  pub func: String,
  pub file: Option<String>,
  pub line: Option<u32>,
}

impl FrameSymbol {
  pub fn is_core_or_stdlib(&self) -> bool {
    self.func.starts_with("core::") || self.func.starts_with("std::")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Add a helper method for testing the message splitting logic
  // without mocking PanicHookInfo
  fn test_split_message(input: &str) -> String {
    input.splitn(2, "\n").next().unwrap().to_string()
  }

  #[test]
  fn test_message_splitting() {
    // Test with single-line message
    let single_line = "test message";
    assert_eq!(test_split_message(single_line), "test message");

    // Test with multi-line message
    let multi_line = "first line\nsecond line\nthird line";
    assert_eq!(test_split_message(multi_line), "first line");
  }

  #[test]
  fn test_frame_symbol_core_or_stdlib() {
    // Test core module detection
    let core_symbol = FrameSymbol {
      func: "core::result::Result<T,E>::expect".to_string(),
      file: None,
      line: None,
    };
    assert!(core_symbol.is_core_or_stdlib());

    // Test std module detection
    let std_symbol = FrameSymbol {
      func: "std::panic::panic_fmt".to_string(),
      file: None,
      line: None,
    };
    assert!(std_symbol.is_core_or_stdlib());

    // Test non-stdlib module
    let custom_symbol = FrameSymbol {
      func: "mysh::exception::tests".to_string(),
      file: None,
      line: None,
    };
    assert!(!custom_symbol.is_core_or_stdlib());
  }

  #[test]
  fn test_parse_frames_regex() {
    // Test the regex pattern against sample backtrace output
    let sample_line =
      r#"{ fn: "mysh::exception::tests::test_function", file: "/path/to/source.rs", line: 42 },"#;

    let captures = FRAME_REGEX.captures(sample_line).unwrap();
    assert_eq!(
      captures.get(1).unwrap().as_str(),
      "mysh::exception::tests::test_function"
    );
    assert_eq!(captures.get(2).unwrap().as_str(), "/path/to/source.rs");
    assert_eq!(captures.get(3).unwrap().as_str(), "42");
  }

  #[test]
  fn test_filtered_range_defaults() {
    // Test that the default range values are None
    let backtrace = Backtrace::capture();
    let exception = ExceptionWithTrace::new(Some("test error".to_string()), Some(&backtrace));

    assert_eq!(exception.filtered_range.0, None);
    assert_eq!(exception.filtered_range.1, None);
  }

  #[test]
  fn test_filtered_range_defaults_no_backtrace() {
    // Test with no backtrace
    let exception = ExceptionWithTrace::new(Some("test error".to_string()), None);

    assert_eq!(exception.filtered_range.0, None);
    assert_eq!(exception.filtered_range.1, None);
    assert!(exception.frames.is_empty(), "Frames should be empty with no backtrace");
  }

  #[test]
  fn test_filtered_range_custom() {
    // Test setting custom range values
    let backtrace = Backtrace::capture();
    let mut exception = ExceptionWithTrace::new(Some("test error".to_string()), Some(&backtrace));

    // Set custom range values
    exception.filtered_range.0 = Some("start_function".to_string());
    exception.filtered_range.1 = Some("end_function".to_string());

    assert_eq!(
      exception.filtered_range.0,
      Some("start_function".to_string())
    );
    assert_eq!(exception.filtered_range.1, Some("end_function".to_string()));
  }

  #[test]
  fn test_filtered_frames_behavior() {
    // Test case 1: With both start and end
    let frames = create_test_frames();
    let exception = ExceptionWithTrace {
      message: Some("test error".to_string()),
      sources: Vec::new(),
      frames: frames.clone(),
      filtered_range: (
        Some("start_function".to_string()),
        Some("end_function".to_string()),
      ),
    };

    // Collect the filtered frames
    let frame_co = std::pin::pin!(exception.filtered_frames());
    let filtered = std::iter::from_coroutine(frame_co).collect::<Vec<_>>();

    // We should get only the frames between start and end (excluding core functions)
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].1.func, "middle_function");

    // Test case 2: With no start, only end
    let exception = ExceptionWithTrace {
      message: Some("test error".to_string()),
      sources: Vec::new(),
      frames: frames.clone(),
      filtered_range: (None, Some("end_function".to_string())),
    };

    let frame_co = std::pin::pin!(exception.filtered_frames());
    let filtered = std::iter::from_coroutine(frame_co).collect::<Vec<_>>();

    assert_eq!(filtered.len(), 3);
    assert_eq!(filtered[0].1.func, "pre_start_function");
    assert_eq!(filtered[1].1.func, "start_function");
    assert_eq!(filtered[2].1.func, "middle_function");

    // Test case 3: With start, no end
    let exception = ExceptionWithTrace {
      message: Some("test error".to_string()),
      sources: Vec::new(),
      frames: frames.clone(),
      filtered_range: (Some("start_function".to_string()), None),
    };

    let frame_co = std::pin::pin!(exception.filtered_frames());
    let filtered = std::iter::from_coroutine(frame_co).collect::<Vec<_>>();

    assert_eq!(filtered.len(), 3);
    assert_eq!(filtered[0].1.func, "middle_function");
    // The test includes post_end_function and end_function when there's no end filter
    assert!(filtered
      .iter()
      .any(|(_, frame)| frame.func == "post_end_function"));
    assert!(filtered
      .iter()
      .any(|(_, frame)| frame.func == "end_function"));

    // Test case 4: With no start and no end
    let exception = ExceptionWithTrace {
      message: Some("test error".to_string()),
      sources: Vec::new(),
      frames: frames.clone(),
      filtered_range: (None, None),
    };

    let frame_co = std::pin::pin!(exception.filtered_frames());
    let filtered = std::iter::from_coroutine(frame_co).collect::<Vec<_>>();

    assert_eq!(filtered.len(), 5);
  }

  // Helper function to create test frames
  fn create_test_frames() -> Vec<FrameSymbol> {
    vec![
      FrameSymbol {
        func: "pre_start_function".to_string(),
        file: Some("/path/to/file1.rs".to_string()),
        line: Some(10),
      },
      FrameSymbol {
        func: "start_function".to_string(),
        file: Some("/path/to/file2.rs".to_string()),
        line: Some(20),
      },
      FrameSymbol {
        func: "middle_function".to_string(),
        file: Some("/path/to/file3.rs".to_string()),
        line: Some(30),
      },
      FrameSymbol {
        func: "core::panicking::panic_fmt".to_string(),
        file: Some("/path/to/core.rs".to_string()),
        line: Some(40),
      },
      FrameSymbol {
        func: "end_function".to_string(),
        file: Some("/path/to/file4.rs".to_string()),
        line: Some(50),
      },
      FrameSymbol {
        func: "post_end_function".to_string(),
        file: Some("/path/to/file5.rs".to_string()),
        line: Some(60),
      },
    ]
  }
}
