pub mod common;
pub mod macos;
pub use crate::common::Clipboard;
pub type ClipboardContext = macos::MacOSClipboardContext;
