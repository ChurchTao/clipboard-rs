# Changelog

## v0.1.5 (2024-04-11) [released]

- Fix: Fix the bug `fn get_image()` where image type is `CF_DIBV5` in `win11`. [issues#14](https://github.com/ChurchTao/clipboard-rs/issues/14)

## v0.1.4 (2024-03-18) [released]

- Fix: Fix the bug `fn read_files()` where no files in clipboard. [issues#11](https://github.com/ChurchTao/clipboard-rs/issues/11)

## v0.1.3 (2024-03-14) [released]

- Fix: Fix the bug on `Windows` can't read DIBV5 format image from clipboard [issues#8](https://github.com/ChurchTao/clipboard-rs/issues/8)
- Fix: Fix the bug on `Windows` can't move `WatcherContext` to another thread [issues#4](https://github.com/ChurchTao/clipboard-rs/issues/4)
- Change: Demo `watch_change.rs` The callback function for monitoring changes in the clipboard is changed to implement a trait. [pr#6](https://github.com/ChurchTao/clipboard-rs/pull/6)

## v0.1.2 (2024-03-08) [released]

- Change `rust-version = "1.75.0"` to `rust-version = "1.63.0"` [pr#3](https://github.com/ChurchTao/clipboard-rs/pull/3)
- Clean up the code and add some comments

## v0.1.1 (2024-03-04) [released]

- Feature: Add a option to `getFiles` or `setFiles`
- Feature: Add a option to `get` or `set` multi items
- Fix: make `WatcherShutdown` public
