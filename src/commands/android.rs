use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::Context;
use async_watcher::AsyncDebouncer;
use colored::Colorize;
use console::Term;
use itertools::Itertools;
use tokio::{
    process::{Child, Command},
    sync::broadcast::{Sender, channel},
};

use crate::{
    android::AndroidServer,
    commands::build::{self, Builder},
    toml_conf::Config,
    utils::relative,
};

async fn spawn_file_reader(watching: Arc<RwLock<bool>>, local: &PathBuf, sender: Sender<Message>) {
    let local = local.clone();
    tokio::spawn(async move {
        let (mut c, mut r) = AsyncDebouncer::new_with_channel(
            Duration::from_millis(20),
            Some(Duration::from_millis(20)),
        )
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
            sender
                .send(Message::BuildProject(Some(
                    data.iter()
                        .map(|x| x.path.clone())
                        .filter(|x| {
                            if let Some(ext) = x.extension() {
                                if ext == "luau" {
                                    return true;
                                }
                            };
                            false
                        })
                        .unique()
                        .collect(),
                )))
                .unwrap();
        }
    });
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Message {
    CloseLove,
    BuildProject(Option<Vec<PathBuf>>),
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
                    sender.send(Message::BuildProject(None)).unwrap();
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
    let mut android_dev_server = AndroidServer::new(
        inquire::Text::new("Put the address here: ")
            .prompt()
            .unwrap(),
    )
    .await
    .unwrap();
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

    let mut configs = Config::from_toml_file(local.join("kaledis.toml")).unwrap();

    // This is currently not available for android since we use a custom hmr implementation
    configs.experimental_hmr = false;
    let mut builder = Builder::new(local.clone(), configs, build::Strategy::BuildDev, true)
        .await
        .unwrap();

    let watching = Arc::new(RwLock::new(false));
    let (sender, mut receiver) = channel::<Message>(2);

    spawn_keyboard_handler(Arc::clone(&watching), sender.clone()).await;
    spawn_file_reader(watching, &local, sender.clone()).await;

    let mut child: Option<Child> = None;
    builder.clean_build_folder().await.unwrap();
    while let Ok(message) = receiver.recv().await {
        if !builder.config.experimental_hmr {
            if let Some(mut child) = child.take() {
                if let Err(err) = child.kill().await {
                    eprintln!("{}\n{}", err, "Failed to kill love2d process.".red());
                } else if let Message::CloseLove = message {
                    println!("{} Closed love.", "[+]".blue());
                };
            }
        }
        if let Message::CloseDev = message {
            break;
        }
        // TODO: handle assets and make this use the file system on the android server
        if let Message::BuildProject(_) = message {
            android_dev_server.report_loading().await.unwrap();
            builder.add_luau_files().await.unwrap();
            let file_contents = tokio::fs::read_to_string(builder.paths.build.join("main.lua"))
                .await
                .unwrap();
            android_dev_server
                .send_code(file_contents.as_bytes().to_vec())
                .await
                .unwrap();
            // I am still doing the android dev command
        }
    }
}
