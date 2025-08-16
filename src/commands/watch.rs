use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Context;
use async_watcher::AsyncDebouncer;
use colored::Colorize;
use console::Term;
use tokio::{
    process::{Child, Command},
    sync::broadcast::{channel, Sender},
};

use crate::{commands::build, toml_conf::Config, utils::relative};

#[derive(Clone)]
struct WatchDaemon<'a> {
    path: &'a PathBuf,
    base_path: Option<PathBuf>,
    love_path: PathBuf,
    love_executable: PathBuf,
}

impl<'a> WatchDaemon<'a> {
    pub fn new(path: &'a PathBuf, love_path: PathBuf, base_path: Option<PathBuf>) -> Self {
        Self {
            path,
            love_executable: love_path.join("love.exe"),
            love_path,
            base_path,
        }
    }
    pub async fn build(&self) -> anyhow::Result<()> {
        build::build(self.base_path.clone(), build::Strategy::BuildDev, false)
            .await
            .context("Building")?;
        Ok(())
    }
    pub async fn run(&self) -> anyhow::Result<Child> {
        if !self.path.join(".build").exists() {
            anyhow::bail!("No bundle found."); // maybe we forgot to build it
        }
        Ok(Command::new(&self.love_executable)
            .current_dir(&self.love_path)
            .arg(self.path.join(".build"))
            .spawn()
            .context("Spawning the process")?)
    }
}

async fn spawn_file_reader(watching: Arc<RwLock<bool>>, local: &PathBuf, sender: Sender<Message>) {
    let local = local.clone();
    tokio::spawn(async move {
        let (mut c, mut r) =
            AsyncDebouncer::new_with_channel(Duration::from_secs(1), Some(Duration::from_secs(1)))
                .await
                .unwrap();

        c.watcher()
            .watch(&local, async_watcher::notify::RecursiveMode::Recursive)
            .unwrap();
        while let Some(Ok(data)) = r.recv().await {
            {
                if !*watching.read().unwrap() {
                    continue;
                }
            }
            if data
                .iter()
                .filter(|x| !x.path.starts_with(local.join(".build")))
                .collect::<Vec<_>>()
                .len()
                < 1
            {
                continue;
            }
            sender.send(Message::BuildProject).unwrap();
        }
    });
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Message {
    CloseLove,
    BuildProject,
    CloseDev,
}

async fn spawn_keyboard_handler(watching: Arc<RwLock<bool>>, sender: Sender<Message>) {
    tokio::task::spawn_blocking(move || {
        let term = Term::stdout();
        loop {
            let key = term.read_key().unwrap_or(console::Key::Unknown);
            match key {
                console::Key::Char('a') | console::Key::Char('A') => {
                    let mut auto_save = watching.write().unwrap();
                    if !*auto_save {
                        println!("{} {}", "[+]".blue(), "Auto Save enabled");
                    } else {
                        println!("{} {}", "[-]".blue(), "Auto Save disabled");
                    }
                    *auto_save = !*auto_save;
                }
                console::Key::Char('L') | console::Key::Char('l') => {
                    sender.send(Message::BuildProject).unwrap();
                }
                console::Key::Char('Q') | console::Key::Char('q') => {
                    sender.send(Message::CloseDev).unwrap();
                    break;
                }
                console::Key::Escape => {
                    print!("[-] Closing...\r");
                    sender.send(Message::CloseLove).unwrap();
                }
                _ => {}
            }
        }
    });
}

pub async fn watch(base_path: Option<PathBuf>) {
    let local = relative(base_path.clone());
    println!("Watching...");
    println!("Press [L] if you want to build manually");
    println!("Press [A] if you want to toggle between auto build and manual mode.");
    println!("Press [Q] if you want to close the dev server.");
    println!("Press [Esc] if you want to close Love.");

    if !local.join("kaledis.toml").exists() {
        eprintln!("{}", "No project found!".red());
        return;
    }

    let configs = Config::from_toml_file(local.join("kaledis.toml")).unwrap();
    let love_path = configs.project.love_path;

    let daemon = WatchDaemon::new(&local, love_path, base_path);

    let watching = Arc::new(RwLock::new(false));
    let (sender, mut receiver) = channel::<Message>(2);

    spawn_keyboard_handler(Arc::clone(&watching), sender.clone()).await;
    spawn_file_reader(watching, &local, sender.clone()).await;

    let mut child: Option<Child> = None;
    while let Ok(message) = receiver.recv().await {
        if let Some(mut child) = child.take() {
            if let Err(err) = child.kill().await {
                eprintln!("{}\n{}", err, "Failed to kill love2d process.".red());
            } else if let Message::CloseLove = message {
                println!("{} Closed love.", "[+]".blue());
            };
        }
        if let Message::CloseDev = message {
            break;
        }
        if let Message::BuildProject = message {
            child = daemon.build().await.and(daemon.run().await).ok();
        }
    }
}
