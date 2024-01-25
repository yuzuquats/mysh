#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  #[error("Other Error >> {}", .0)]
  Other(#[from] anyhow::Error),
}
