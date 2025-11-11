use clipboard_rs::{ClipboardManager, ContentFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建新的剪贴板管理器
    let clipboard = ClipboardManager::new().await?;

    // 获取所有可用格式
    let formats = clipboard.available_formats().await?;
    println!("Available formats: {:?}", formats);

    // 检查是否包含 RTF 格式
    let has_rtf = clipboard.has(ContentFormat::Rtf).await?;
    println!("Has RTF: {}", has_rtf);

    // 获取 RTF 内容
    let rtf = clipboard.get_rtf().await.unwrap_or_default();
    println!("RTF: {}", rtf);

    // 获取 HTML 内容
    let has_html = clipboard.has(ContentFormat::Html).await?;
    println!("Has HTML: {}", has_html);

    let html = clipboard.get_html().await.unwrap_or_default();
    println!("HTML: {}", html);

    // 获取文本内容
    let text = clipboard.get_text().await.unwrap_or_default();
    println!("Text: {}", text);

    // 使用构建器设置多种内容
    clipboard
        .set_with_builder(
            clipboard
                .build_content()
                .with_text("Hello, World! (Modern API)")
                .with_html("<h1>Hello, World! (Modern API)</h1>")
                .with_rtf(r"{\rtf1\ansi\b Hello, World! (Modern API)}")
        )
        .await?;

    println!("Content set successfully!");

    Ok(())
}