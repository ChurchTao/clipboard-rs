use thiserror::Error;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[derive(Debug, Error)]
pub enum ClipboardError {
	#[error("clipboard content is empty")]
	EmptyContent,
	#[error("clipboard watcher task failed: {0}")]
	TaskJoin(String),
	#[error("clipboard access failed: {0}")]
	Message(String),
}
