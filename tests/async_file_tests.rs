//! 异步文件相关功能的测试用例
//! 包括异步API的文件操作测试

#[cfg(feature = "async")]
use clipboard_rs::{AsyncClipboardManager, ContentFormat};

#[cfg(target_os = "macos")]
const TMP_PATH: &str = "/tmp/";
#[cfg(target_os = "windows")]
const TMP_PATH: &str = "C:\\Windows\\Temp\\";
#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "emscripten"
    ))
))]
const TMP_PATH: &str = "/tmp/";

// ios
#[cfg(any(target_os = "ios", target_os = "android"))]
const TMP_PATH: &str = "/tmp/";

/// 异步文件测试
#[tokio::test]
#[cfg(feature = "async")]
async fn test_async_file() {
    let clipboard = AsyncClipboardManager::new().await.unwrap();
    clipboard.clear().await.unwrap();

    let file_list = get_files();

    clipboard.set_files(&file_list).await.unwrap();

    let types = clipboard.available_formats().await.unwrap();
    println!("Available formats: {:?}", types);

    let has = clipboard.has(ContentFormat::Files).await.unwrap();
    assert!(has);

    let files = clipboard.get_files().await.unwrap();
    assert_eq!(files.len(), 2);

    for file in &files {
        println!("File: {:?}", file);
    }

    clipboard.clear().await.unwrap();

    let has = clipboard.has(ContentFormat::Files).await.unwrap();
    assert!(!has);

    let contents = clipboard
        .build_content()
        .with_text(&file_list.join("\n"))
        .with_files(&file_list);

    clipboard.set(contents).await.unwrap();

    let has = clipboard.has(ContentFormat::Files).await.unwrap();
    assert!(has);

    let types = clipboard.available_formats().await.unwrap();
    println!("Available formats after setting: {:?}", types);

    let contents = clipboard
        .get(&[ContentFormat::Text, ContentFormat::Files])
        .await
        .unwrap();

    assert_eq!(contents.len(), 2);

    for c in contents {
        match c {
            clipboard_rs::ClipboardContent::Text(data) => {
                assert_eq!(data, file_list.join("\n"));
                println!("ClipboardContent::Text = {}", data);
            }
            clipboard_rs::ClipboardContent::Files(files) => {
                assert_eq!(files.len(), 2);
                for file in &files {
                    println!("ClipboardContent::Files = {:?}", file);
                }
            }
            _ => panic!("unexpected format"),
        }
    }
}

/// 获取测试文件列表
fn get_files() -> Vec<String> {
    let test_file1 = format!("{}clipboard_rs_test_file1.txt", TMP_PATH);
    let test_file2 = format!("{}clipboard_rs_test_file2.txt", TMP_PATH);
    std::fs::write(&test_file1, "hello world").unwrap();
    std::fs::write(&test_file2, "hello world").unwrap();
    vec![test_file1, test_file2]
}