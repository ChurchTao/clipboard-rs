# 测试套件说明

这个目录包含了clipboard-rs库的完整测试套件，按照功能和调用方式进行了组织。

## 测试文件组织

### 文本相关测试
- `string_tests.rs` - 同步文本操作测试
- `async_string_tests.rs` - 异步文本操作测试

### 图像相关测试
- `image_tests.rs` - 同步图像操作测试
- `async_image_tests.rs` - 异步图像操作测试

### 文件相关测试
- `file_tests.rs` - 同步文件操作测试
- `async_file_tests.rs` - 异步文件操作测试

### 原始数据测试
- `raw_data_tests.rs` - 同步原始数据操作测试
- `async_raw_data_tests.rs` - 异步原始数据操作测试

### 构建器模式测试
- `builder_tests.rs` - 同步构建器模式测试
- `async_builder_tests.rs` - 异步构建器模式测试

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定功能测试
```bash
# 运行文本相关测试
cargo test string

# 运行图像相关测试（需要启用image功能）
cargo test image --features=image

# 运行异步相关测试（需要启用async功能）
cargo test async --features=async

# 运行文件相关测试
cargo test file
```

### 运行特定平台测试
```bash
# 运行macOS特定测试
cargo test --target=x86_64-apple-darwin

# 运行Windows特定测试
cargo test --target=x86_64-pc-windows-msvc

# 运行Linux特定测试
cargo test --target=x86_64-unknown-linux-gnu
```

## 测试特点

1. **API一致性**: 所有测试都使用新的`SyncClipboardManager`和`AsyncClipboardManager` API
2. **功能标记**: 充分利用了`text`、`image`、`async`等feature标记
3. **同步异步覆盖**: 为每个功能提供了同步和异步两种测试方式
4. **构建器模式**: 包含了对构建器模式的完整测试
5. **平台特定测试**: 保留了平台特定的测试用例（如macOS多格式项目测试）
6. **错误处理**: 包含了错误处理和边界条件测试

## 注意事项

1. 图像测试需要`tests/test.png`文件存在
2. 某些测试会修改系统剪贴板，建议在隔离环境中运行
3. 平台特定测试可能需要特定的运行环境