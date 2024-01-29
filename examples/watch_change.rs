use clipboard_rs::{Clipboard, ClipboardContext};

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();

    ctx.add_listener(Box::new(|_ctx| {
        println!("Clipboard changed!");
        _ctx.available_formats().unwrap().iter().for_each(|f| {
            println!("{}", f);
        });
    }));

    ctx.start_listen_change();
}
