use std::thread::{self, JoinHandle};

use crate::{ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext, Result, WatcherShutdown};

struct CallbackHandler {
	callback: Box<dyn FnMut() + Send + 'static>,
}

impl ClipboardHandler for CallbackHandler {
	fn on_clipboard_change(&mut self) {
		(self.callback)();
	}
}

/// Preferred watcher API for new code.
///
/// This builder-style API hides the generic `ClipboardWatcherContext<T>` and
/// allows users to register closures directly.
pub struct ClipboardWatcherBuilder {
	handlers: Vec<CallbackHandler>,
}

impl ClipboardWatcherBuilder {
	pub fn new() -> Self {
		Self {
			handlers: Vec::new(),
		}
	}

	pub fn on_change<F>(mut self, callback: F) -> Self
	where
		F: FnMut() + Send + 'static,
	{
		self.handlers.push(CallbackHandler {
			callback: Box::new(callback),
		});
		self
	}

	pub fn run_blocking(self) -> Result<()> {
		let mut watcher = ClipboardWatcherContext::new()?;
		for handler in self.handlers {
			watcher.add_handler(handler);
		}
		watcher.start_watch();
		Ok(())
	}

	pub fn spawn(self) -> Result<RunningClipboardWatcher> {
		let mut watcher = ClipboardWatcherContext::new()?;
		for handler in self.handlers {
			watcher.add_handler(handler);
		}
		let shutdown = watcher.get_shutdown_channel();
		let join_handle = thread::spawn(move || {
			watcher.start_watch();
		});
		Ok(RunningClipboardWatcher {
			shutdown: Some(shutdown),
			join_handle: Some(join_handle),
		})
	}
}

impl Default for ClipboardWatcherBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Handle to a running watcher thread created by [`ClipboardWatcherBuilder::spawn`].
pub struct RunningClipboardWatcher {
	shutdown: Option<WatcherShutdown>,
	join_handle: Option<JoinHandle<()>>,
}

impl RunningClipboardWatcher {
	pub fn stop(mut self) -> Result<()> {
		if let Some(shutdown) = self.shutdown.take() {
			shutdown.stop();
		}
		if let Some(handle) = self.join_handle.take() {
			handle
				.join()
				.map_err(|_| std::io::Error::other("watcher thread panicked"))?;
		}
		Ok(())
	}
}
