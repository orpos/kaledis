use std::{
    io::Write,
    sync::{Arc, Weak},
    time::Duration,
};

use console::{style, Term};
use tokio::{sync::RwLock, time::sleep};

pub struct LoadingStatusBar {
    status: Arc<RwLock<String>>,
}

async fn loading_animation(text: Weak<RwLock<String>>) {
    let mut term = Term::stdout();
    let mut frame_index = 0;
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    loop {
        // As we have a Weak reference when the variable holding LoadingStatusBar dies
        // it will be deleted and as such the upgrade will return None
        // So we stop showing the animation
        if let Some(text) = text.upgrade() {
            let data = &*text.read().await;
            term.write(
                format!("{} {}\r", style(frames[frame_index]).dim().blue(), data).as_bytes(),
            )
            .unwrap();
            frame_index += 1;
            frame_index %= frames.len();
            sleep(Duration::from_millis(100)).await;
            continue;
        }
        break;
    }
}

impl LoadingStatusBar {
    pub fn new(text: String) -> LoadingStatusBar {
        Self {
            status: Arc::new(RwLock::new(text)),
        }
    }
    pub async fn start_animation(&self) {
        let status = Arc::downgrade(&self.status);
        tokio::spawn(async move {
            loading_animation(status).await;
        });
    }
    pub async fn change_status(&self, data: String) {
        // Clear the line before because there maybe some chars from the previous text
        let _ = Term::stdout().clear_line();

        *self.status.write().await = data;
    }
}
