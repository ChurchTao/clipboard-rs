use clipboard_rs::{ClipboardWatcher, ClipboardWatcherContext};
use std::{thread, time::Duration};

#[test]
fn should_shutdown_successfully() {
    let mut watcher = ClipboardWatcherContext::new().unwrap();

    watcher.add_handler(Box::new(move || {
        println!("changed");
    }));

    let watcher_shutdown = watcher.get_shutdown_channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        println!("stop watch!");
        watcher_shutdown.stop();
    });

    println!("start watch!");
    watcher.start_watch();
}
