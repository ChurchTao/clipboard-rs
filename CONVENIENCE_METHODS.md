# 剪贴板便利方法 (Clipboard Convenience Methods)

此文档描述了为解决 [Issue #67](https://github.com/ChurchTao/clipboard-rs/issues/67) 而添加的新便利方法。

## 问题背景

在原有的API中，开发者需要在剪贴板变化回调中手动调用 `available_formats()` 来检测格式类型，然后解析平台相关的格式字符串。这种做法存在以下问题：

1. **使用繁琐**：每次都需要手动解析格式
2. **跨平台差异**：不同平台的格式字符串不一致  
3. **代码重复**：需要重复编写相同的检测逻辑

## 解决方案

我们在 `ClipboardContext` 中添加了以下便利方法：

### 新增方法

#### 格式检测方法

```rust
impl ClipboardContext {
    /// 获取剪贴板中当前可用的所有内容格式
    pub fn get_current_formats(&self) -> Result<Vec<ContentFormat>>;
    
    /// 检查剪贴板是否包含文本内容
    pub fn has_text(&self) -> bool;
    
    /// 检查剪贴板是否包含图像内容  
    pub fn has_image(&self) -> bool;
    
    /// 检查剪贴板是否包含文件列表
    pub fn has_files(&self) -> bool;
    
    /// 检查剪贴板是否包含HTML内容
    pub fn has_html(&self) -> bool;
    
    /// 检查剪贴板是否包含RTF内容
    pub fn has_rtf(&self) -> bool;
}
```

### 向前兼容性保证

- ✅ **完全向后兼容**：现有代码无需任何修改
- ✅ **完全向前兼容**：新功能只是添加方法，不破坏现有API
- ✅ **一致性**：新方法与现有 `has()` 方法返回一致的结果

## 使用示例

### 在剪贴板监听器中使用

```rust
impl ClipboardHandler for Manager {
    fn on_clipboard_change(&mut self) {
        // 新的便利方法 - 简单直接
        if self.ctx.has_text() {
            let text = self.ctx.get_text().unwrap();
            println!("检测到文本: {}", text);
        }
        
        if self.ctx.has_image() {
            let image = self.ctx.get_image().unwrap();
            let (w, h) = image.get_size();
            println!("检测到图像: {}x{}", w, h);
        }
        
        if self.ctx.has_files() {
            let files = self.ctx.get_files().unwrap();
            println!("检测到 {} 个文件", files.len());
        }
        
        // 获取所有格式
        let formats = self.ctx.get_current_formats().unwrap();
        println!("共检测到 {} 种格式", formats.len());
    }
}
```

### 对比新旧API

```rust
// 旧方式 - 繁琐
let types = ctx.available_formats().unwrap();
if types.contains(&"text/plain".to_string()) {
    // 处理文本
} else if types.contains(&"image/png".to_string()) {
    // 处理图像  
}

// 新方式 - 简洁
if ctx.has_text() {
    // 处理文本
} else if ctx.has_image() {
    // 处理图像
}

// 获取类型化的格式列表
let formats = ctx.get_current_formats().unwrap();
for format in formats {
    match format {
        ContentFormat::Text => println!("文本格式"),
        ContentFormat::Image => println!("图像格式"),
        ContentFormat::Html => println!("HTML格式"),
        ContentFormat::Files => println!("文件格式"),
        ContentFormat::Rtf => println!("RTF格式"),
        ContentFormat::Other(name) => println!("其他格式: {}", name),
    }
}
```

## 实现细节

### 平台支持

所有便利方法已在以下平台上实现：
- ✅ Windows
- ✅ macOS  
- ✅ Linux (X11)
- ✅ iOS

### 性能考虑

- 便利方法内部调用现有的 `has()` 方法，性能开销微乎其微
- `get_current_formats()` 方法会检测所有支持的格式，比单独调用稍慢，但提供了完整的格式信息

### 错误处理

- 便利的 `has_*()` 方法返回 `bool`，内部处理错误并返回 `false`
- `get_current_formats()` 返回 `Result<Vec<ContentFormat>>`，允许调用者处理错误

## 测试

新功能已通过完整的测试套件验证：

```bash
cargo test                          # 运行所有测试
cargo test --test convenience_methods_test  # 运行便利方法专项测试
cargo run --example convenience_methods     # 运行示例程序
```

## 后续扩展

这种设计为未来扩展奠定了基础：

1. 可以轻松添加更多便利方法（如 `has_custom_format()`）
2. 可以在 `ClipboardChangeEvent` 中包含格式信息（未来版本）
3. 可以添加格式优先级检测等高级功能

## 总结

这次改进显著提升了 clipboard-rs 的易用性，让开发者能够更优雅地处理不同类型的剪贴板内容，同时保持了完美的向前向后兼容性。