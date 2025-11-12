use clipboard_rs::{ClipboardManager, common::ContentFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建新的剪贴板管理器
    let clipboard = ClipboardManager::new().await?;

    // 清空剪贴板
    clipboard.clear().await?;

    // 设置原始数据
    let custom_data = b"Hello, Raw Data!".to_vec();
    clipboard.set_raw("custom/format", &custom_data).await?;

    // 检查是否包含自定义格式
    let formats = clipboard.available_formats().await?;
    println!("Available formats: {:?}", formats);

    // 获取原始数据
    if formats.contains(&"custom/format".to_string()) {
        let retrieved_data = clipboard.get_raw("custom/format").await?;
        println!("Retrieved raw data: {:?}", String::from_utf8_lossy(&retrieved_data));
    }

    // 设置多种原始数据格式
    clipboard.set_raw("application/json", br#"{"message": "Hello JSON"}"#).await?;
    clipboard.set_raw("text/csv", b"name,age\nAlice,30\nBob,25").await?;

    // 获取所有可用格式
    let all_formats = clipboard.available_formats().await?;
    println!("All available formats: {:?}", all_formats);

    // 获取JSON数据
    if all_formats.contains(&"application/json".to_string()) {
        let json_data = clipboard.get_raw("application/json").await?;
        println!("JSON data: {}", String::from_utf8_lossy(&json_data));
    }

    // 获取CSV数据
    if all_formats.contains(&"text/csv".to_string()) {
        let csv_data = clipboard.get_raw("text/csv").await?;
        println!("CSV data: {}", String::from_utf8_lossy(&csv_data));
    }

    println!("Raw data operations completed successfully!");

    Ok(())
}