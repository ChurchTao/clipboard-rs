use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat};

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

#[test]
fn test_file() {
	let ctx = ClipboardContext::new().unwrap();

	let file_list = get_files();

	ctx.set_files(file_list.clone()).unwrap();

	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let has = ctx.has(ContentFormat::Files);
	assert!(has);

	let files = ctx.get_files().unwrap();
	assert_eq!(files.len(), 2);

	for file in files {
		println!("{:?}", file);
	}

	ctx.clear().unwrap();

	let has = ctx.has(ContentFormat::Files);
	assert!(!has);

	ctx.set(vec![
		ClipboardContent::Text(file_list.clone().join("\n").to_string()),
		ClipboardContent::Files(file_list.clone()),
	])
	.unwrap();

	let has = ctx.has(ContentFormat::Files);
	assert!(has);

	let types = ctx.available_formats().unwrap();
	println!("{:?}", types);

	let contents = ctx
		.get(&[ContentFormat::Text, ContentFormat::Files])
		.unwrap();

	assert_eq!(contents.len(), 2);

	for c in contents {
		match c {
			ClipboardContent::Text(data) => {
				assert_eq!(data, file_list.clone().join("\n"));
				println!("ClipboardContent::Text = {}", data);
			}
			ClipboardContent::Files(files) => {
				assert_eq!(files.len(), 2);
				for file in files {
					println!("ClipboardContent::Files = {:?}", file);
				}
			}
			_ => panic!("unexpected format"),
		}
	}
}

fn get_files() -> Vec<String> {
	let test_file1 = format!("{}clipboard_rs_test_file1.txt", TMP_PATH);
	let test_file2 = format!("{}clipboard_rs_test_file2.txt", TMP_PATH);
	std::fs::write(&test_file1, "hello world").unwrap();
	std::fs::write(&test_file2, "hello world").unwrap();
	vec![test_file1, test_file2]
}
