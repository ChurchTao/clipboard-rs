use clipboard_rs::{AsyncClipboardWatcher, ClipboardEvent, AsyncClipboardManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new async clipboard manager
    let clipboard = AsyncClipboardManager::new().await?;

    // Start watching clipboard changes
    let mut event_stream = clipboard.watch().await?;

    println!("Clipboard watcher started. Try copying some text to see events!");
    println!("Stopping in 10 seconds...");

    // Handle events for 10 seconds
    let start_time = std::time::Instant::now();
    loop {
        // Check if 10 seconds have passed
        if start_time.elapsed() >= std::time::Duration::from_secs(10) {
            println!("10 seconds elapsed, stopping watcher...");
            break;
        }

        // Use a timeout to check for events
        match tokio::time::timeout(std::time::Duration::from_secs(1), event_stream.next()).await {
            Ok(Some(event)) => {
                match event {
                    ClipboardEvent::Changed { formats } => {
                        println!("Clipboard changed! Available formats: {:?}", formats);
                    }
                    ClipboardEvent::Cleared => {
                        println!("Clipboard cleared!");
                    }
                    ClipboardEvent::Error(e) => {
                        eprintln!("Clipboard error: {:?}", e);
                    }
                }
            }
            Ok(None) => {
                println!("Event stream ended");
                break;
            }
            Err(_) => {
                // Timeout, continue loop to check elapsed time
                continue;
            }
        }
    }

    Ok(())
}
