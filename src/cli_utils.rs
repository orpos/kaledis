use std::{ io::Write, sync::{ Arc, Weak }, time::Duration };

use console::{ style, Term };
use tokio::{ sync::RwLock, time::sleep };

pub struct LoadingStatusBar {
    status: Arc<RwLock<String>>,
}

async fn loading_animation(text: Weak<RwLock<String>>) {
    let mut term = Term::stdout();
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut frame_index = 0;
    loop {
        // tentando resolver um bug
        let result = text.upgrade();
        match result {
            None => {
                break;
            }
            Some(text) => {
                let data = &*text.read().await;
                term.write(
                    format!("{} {}\r", style(frames[frame_index]).dim().blue(), data).as_bytes()
                ).unwrap();
                frame_index += 1;
                frame_index %= frames.len();
                sleep(Duration::from_millis(100)).await;
            }
        }
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
        let mut lock = self.status.write().await;
        *lock = data;
    }
}

#[macro_export]
macro_rules! allow {
    ($target:expr, $equal:expr) => {
        {
            $target == $equal
        }
    };
    ($target:expr, $equal:expr, $($eq:expr),+) => {
        {
            allow!($target, $equal) ||
            allow!($target, $($eq),+)
        }
    };
}
