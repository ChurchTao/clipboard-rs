# Changelog

## v0.3.0 (2025-07-01) [released]

- Add: Support iOS, but only for test, not for production.
- Fix: Merge [pr#62](https://github.com/ChurchTao/clipboard-rs/pull/62)

## v0.2.4 (2025-02-07) [released]

- Fix: [pr#56](https://github.com/ChurchTao/clipboard-rs/pull/56)

## v0.2.3 (2025-02-07) [released]

- Fix: [issues#57](https://github.com/ChurchTao/clipboard-rs/issues/57)
- Fix: [pr#56](https://github.com/ChurchTao/clipboard-rs/pull/56)

## v0.2.2 (2024-11-19) [released]

- Convergence dep: `image` to `jpeg/png/tiff/bmp` [pr#54](https://github.com/ChurchTao/clipboard-rs/pull/54)

## v0.2.1 (2024-08-26) [released]

### zh

- 增加 X11 启动参数，自定义读取的超时时间 [issues#45](https://github.com/ChurchTao/clipboard-rs/issues/45)

### en

- Add X11 startup parameters to customize the timeout for reading [issues#45](https://github.com/ChurchTao/clipboard-rs/issues/45)

## v0.2.0 (2024-08-25) [released]

### zh

- macOS 性能优化，增强 test 类，替换使用 objc2
- 修复 windows 读取 rtf 可能失败的情况
- 修复读取 html 少<的情况，【因为 wps 写入的 StartFragment 有问题】

### en

- macOS performance optimization, enhanced test class, replaced with objc2
- Fixed the case where reading rtf on windows may fail
- Fixed the case of reading html less than <, [because the StartFragment written by wps is problematic]

## v0.1.9 (2024-07-22) [released]

- Fix: Bug: `set` on windows without clear [issues#32](https://github.com/ChurchTao/clipboard-rs/issues/32)

## v0.1.8 (2024-07-18) [released]

- Fix: Bug: When read rimeout on Linux there is throw error but not
  loop [issues#30](https://github.com/ChurchTao/clipboard-rs/issues/30)

## v0.1.7 (2024-04-30) [released]

- Fix: Bug: Cannot write all content when writing to html on
  Windows [issues#23](https://github.com/ChurchTao/clipboard-rs/issues/23)

## v0.1.6 (2024-04-12) [released]

- Fix: Bug: Cannot paste after writing image to clipboard (on Windows)
  #17 [issues#17](https://github.com/ChurchTao/clipboard-rs/issues/17)
- Fix: Bug: No transparent background for clipboard image read on Windows
  #18 [issues#18](https://github.com/ChurchTao/clipboard-rs/issues/18)
- Fix: Bug: Cannot read clipboard image on MacOS for screenshots taken by certain apps
  #19 [issues#19](https://github.com/ChurchTao/clipboard-rs/issues/19)

## v0.1.5 (2024-04-11) [released]

- Fix: Fix the bug `fn get_image()` where image type is `CF_DIBV5`
  in `win11`. [issues#14](https://github.com/ChurchTao/clipboard-rs/issues/14)

## v0.1.4 (2024-03-18) [released]

- Fix: Fix the bug `fn read_files()` where no files in
  clipboard. [issues#11](https://github.com/ChurchTao/clipboard-rs/issues/11)

## v0.1.3 (2024-03-14) [released]

- Fix: Fix the bug on `Windows` can't read DIBV5 format image from
  clipboard [issues#8](https://github.com/ChurchTao/clipboard-rs/issues/8)
- Fix: Fix the bug on `Windows` can't move `WatcherContext` to another
  thread [issues#4](https://github.com/ChurchTao/clipboard-rs/issues/4)
- Change: Demo `watch_change.rs` The callback function for monitoring changes in the clipboard is changed to implement a
  trait. [pr#6](https://github.com/ChurchTao/clipboard-rs/pull/6)

## v0.1.2 (2024-03-08) [released]

- Change `rust-version = "1.75.0"` to `rust-version = "1.63.0"` [pr#3](https://github.com/ChurchTao/clipboard-rs/pull/3)
- Clean up the code and add some comments

## v0.1.1 (2024-03-04) [released]

- Feature: Add a option to `getFiles` or `setFiles`
- Feature: Add a option to `get` or `set` multi items
- Fix: make `WatcherShutdown` public
