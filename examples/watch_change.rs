use clipboard_rs::{Clipboard, ClipboardContext};

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();

    ctx.on_change(Box::new(|| {
        println!("Clipboard changed!");
    }))
    .unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
