use clipboard_rs::{
	Clipboard, ClipboardContent, ClipboardContext, ClipboardHandler, ClipboardWatcher, 
	ClipboardWatcherContext, ContentFormat, RustImageData,
	common::RustImage,
};
use std::{thread, time::Duration};

struct FormatManager {
	ctx: ClipboardContext,
}

impl FormatManager {
	pub fn new() -> Self {
		let ctx = ClipboardContext::new().unwrap();
		FormatManager { ctx }
	}
}

impl ClipboardHandler for FormatManager {
	fn on_clipboard_change(&mut self) {
		println!("剪贴板内容发生变化！");
		
		// 使用新的便利方法快速检查格式
		println!("格式检测:");
		println!("  📝 包含文本: {}", self.ctx.has_text());
		println!("  🖼️  包含图像: {}", self.ctx.has_image());
		println!("  📁 包含文件: {}", self.ctx.has_files());
		println!("  🌐 包含HTML: {}", self.ctx.has_html());
		println!("  📄 包含RTF: {}", self.ctx.has_rtf());
		
		// 获取所有当前格式
		match self.ctx.get_current_formats() {
			Ok(formats) => {
				println!("  📋 当前格式数量: {}", formats.len());
				for (i, format) in formats.iter().enumerate() {
					match format {
						ContentFormat::Text => println!("    {}. 纯文本格式", i + 1),
						ContentFormat::Image => println!("    {}. 图像格式", i + 1),
						ContentFormat::Html => println!("    {}. HTML格式", i + 1),
						ContentFormat::Files => println!("    {}. 文件列表格式", i + 1),
						ContentFormat::Rtf => println!("    {}. RTF格式", i + 1),
						ContentFormat::Other(name) => println!("    {}. 其他格式: {}", i + 1, name),
					}
				}
			}
			Err(e) => println!("  ❌ 获取格式失败: {}", e),
		}
		
		// 根据格式获取内容
		if self.ctx.has_text() {
			match self.ctx.get_text() {
				Ok(text) => {
					let preview = if text.len() > 50 {
						format!("{}...", &text[..47])
					} else {
						text
					};
					println!("  📝 文本内容预览: \"{}\"", preview);
				}
				Err(e) => println!("  ❌ 获取文本失败: {}", e),
			}
		}
		
		if self.ctx.has_image() {
			match self.ctx.get_image() {
				Ok(image) => {
					let (width, height) = image.get_size();
					println!("  🖼️  图像尺寸: {}x{} 像素", width, height);
				}
				Err(e) => println!("  ❌ 获取图像失败: {}", e),
			}
		}
		
		if self.ctx.has_files() {
			match self.ctx.get_files() {
				Ok(files) => {
					println!("  📁 文件数量: {}", files.len());
					for (i, file) in files.iter().take(3).enumerate() {
						println!("    {}. {}", i + 1, file);
					}
					if files.len() > 3 {
						println!("    ... 还有 {} 个文件", files.len() - 3);
					}
				}
				Err(e) => println!("  ❌ 获取文件列表失败: {}", e),
			}
		}
		
		println!("─────────────────────────────");
	}
}

fn main() {
	println!("🦀 Clipboard-rs 便利方法演示");
	println!("本示例展示了新的便利方法如何简化剪贴板格式检测");
	println!("═════════════════════════════");
	
	// 演示基本用法
	demonstrate_basic_usage();
	
	// 演示剪贴板监听
	demonstrate_clipboard_watching();
}

fn demonstrate_basic_usage() {
	println!("\n📋 基本用法演示:");
	let ctx = ClipboardContext::new().unwrap();
	
	// 测试文本
	println!("\n1. 测试文本内容:");
	let test_text = "Hello, World! 你好世界！🦀";
	ctx.set_text(test_text.to_string()).unwrap();
	
	// 使用便利方法
	if ctx.has_text() {
		println!("   ✓ 检测到文本内容");
		let formats = ctx.get_current_formats().unwrap();
		println!("   ✓ 当前格式数量: {}", formats.len());
	}
	
	// 测试HTML
	println!("\n2. 测试HTML内容:");
	let test_html = "<html><body><h1>Hello HTML!</h1><p>这是一个HTML测试</p></body></html>";
	ctx.set_html(test_html.to_string()).unwrap();
	
	if ctx.has_html() {
		println!("   ✓ 检测到HTML内容");
	}
	
	// 测试RTF
	println!("\n3. 测试RTF内容:");
	let test_rtf = r"{\rtf1\ansi\deff0 {\fonttbl {\f0 Times New Roman;}} \f0\fs60 Hello RTF!}";
	ctx.set_rich_text(test_rtf.to_string()).unwrap();
	
	if ctx.has_rtf() {
		println!("   ✓ 检测到RTF内容");
	}
	
	// 测试图像（如果测试图像存在）
	println!("\n4. 测试图像内容:");
	if let Ok(test_image) = RustImageData::from_path("tests/test.png") {
		ctx.set_image(test_image).unwrap();
		if ctx.has_image() {
			let (width, height) = ctx.get_image().unwrap().get_size();
			println!("   ✓ 检测到图像内容 ({}x{})", width, height);
		}
	} else {
		println!("   ⚠️ 跳过图像测试（测试图像不存在）");
	}
	
	// 测试多格式内容
	println!("\n5. 测试多格式内容:");
	let contents: Vec<ClipboardContent> = vec![
		ClipboardContent::Text(test_text.to_string()),
		ClipboardContent::Html(test_html.to_string()),
		ClipboardContent::Rtf(test_rtf.to_string()),
	];
	ctx.set(contents).unwrap();
	
	let formats = ctx.get_current_formats().unwrap();
	println!("   ✓ 同时设置多种格式，检测到 {} 种格式", formats.len());
	
	// 使用便利方法快速检查
	println!("   快速格式检查:");
	println!("     📝 文本: {}", if ctx.has_text() { "✓" } else { "✗" });
	println!("     🌐 HTML: {}", if ctx.has_html() { "✓" } else { "✗" });
	println!("     📄 RTF:  {}", if ctx.has_rtf() { "✓" } else { "✗" });
	println!("     🖼️ 图像: {}", if ctx.has_image() { "✓" } else { "✗" });
	println!("     📁 文件: {}", if ctx.has_files() { "✓" } else { "✗" });
	
	// 对比新旧API
	println!("\n6. API对比:");
	println!("   旧方式: ctx.has(ContentFormat::Text) = {}", ctx.has(ContentFormat::Text));
	println!("   新方式: ctx.has_text() = {}", ctx.has_text());
	println!("   结果一致: {}", ctx.has(ContentFormat::Text) == ctx.has_text());
}

fn demonstrate_clipboard_watching() {
	println!("\n🔍 剪贴板监听演示:");
	println!("启动剪贴板监听器，复制一些内容到剪贴板来测试...");
	println!("程序将在10秒后自动停止");
	
	let manager = FormatManager::new();
	let mut watcher = ClipboardWatcherContext::new().unwrap();
	let watcher_shutdown = watcher.add_handler(manager).get_shutdown_channel();
	
	// 启动监听线程
	thread::spawn(move || {
		watcher.start_watch();
	});
	
	// 10秒后停止监听
	thread::spawn(move || {
		thread::sleep(Duration::from_secs(10));
		println!("\n⏰ 10秒已到，停止监听...");
		watcher_shutdown.stop();
	});
	
	// 主线程等待
	thread::sleep(Duration::from_secs(11));
	println!("✅ 演示完成！");
}