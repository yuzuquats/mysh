#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  #[error("{}: {}", .0, if let Some(source) = .0.source() { source.to_string() } else { "".to_string() })]
  Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
