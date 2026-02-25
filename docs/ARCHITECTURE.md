# Architecture Notes (Refactor)

## Design goals

1. Keep existing clipboard capabilities unchanged (text, html, rtf, image, files, watcher).
2. Separate **platform implementation** from **public workflow API**.
3. Provide both sync and async entry points.
4. Make watcher usage ergonomic and thread-lifecycle-safe.

## Layered architecture

- `platform/*`:
  OS-specific clipboard read/write/watch implementations.
- `ClipboardContext` / `ClipboardWatcherContext<T>`:
  low-level, compatibility-oriented API.
- `ClipboardService`:
  sync high-level facade for application code.
- `AsyncClipboardService` (`feature = "async"`):
  async facade for Tokio applications.
- `ClipboardWatcherBuilder`:
  closure-based watcher API with explicit running handle.

## API flow recommendations

### Write flow

Application -> `ClipboardService` / `AsyncClipboardService` -> `ClipboardContext` -> platform backend.

### Read flow

Application -> `ClipboardService` / `AsyncClipboardService` -> `ClipboardContext` -> platform backend -> normalized result.

### Watch flow

Application -> `ClipboardWatcherBuilder::on_change(...)`
-> `.spawn()` (returns `RunningClipboardWatcher`)
-> `RunningClipboardWatcher::stop()` for graceful shutdown.

This gives explicit ownership for start/stop lifecycle and avoids exposing generic handler types to callers.

## Why watcher builder is better

- Eliminates user-side generic type complexity (`ClipboardWatcherContext<T>`).
- Supports multiple callbacks naturally.
- Provides explicit `stop()` + thread `join()` lifecycle.
- Keeps old watcher API for compatibility.

