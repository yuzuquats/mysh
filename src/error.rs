#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  #[error("{}", .0)]
  Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
