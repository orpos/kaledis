use std::{
    path::PathBuf,
    process::exit,
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
    signal,
    sync::broadcast::{Sender, channel},
};

use crate::{
    android::DevServer,
    commands::build::{Builder, Strategy},
    home_manager::CURRENT_PLATFORM,
    utils::relative,
};

async fn spawn_file_reader(watching: Arc<RwLock<bool>>, local: &PathBuf, sender: Sender<Message>) {
    let local = local.clone();
    tokio::spawn(async move {
        let (mut c, mut r) = AsyncDebouncer::new_with_channel(
            Duration::from_millis(1),
            Some(Duration::from_millis(1)),
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

    // let configs = KConfig::from_toml_file(local.join("kaledis.toml")).unwrap();

    // let daemon = WatchDaemon::new(&local, love_path, base_path);
    let builder = Builder::new(local.clone(), Strategy::BuildDev, false).await;

    let watching = Arc::new(RwLock::new(false));
    let (sender, mut receiver) = channel::<Message>(2);

    spawn_keyboard_handler(Arc::clone(&watching), sender.clone()).await;
    spawn_file_reader(watching, &local, sender.clone()).await;

    let mut child: Option<Child> = None;
    builder.clean_build_folder().await.unwrap();
    builder.transpile().await;
    let mut path = builder
        .home
        .get_path(&builder.config.love, CURRENT_PLATFORM.clone())
        .await;

    #[cfg(windows)]
    path.push("love.exe");

    #[cfg(target_os = "linux")]
    path.push("love2d.AppImage");

    let mut server = None;

    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("failed to listen to control+c");
        println!("\n\n{} Ctrl+C detected. Closing...\n\n", "[!]".red());
        sender.send(Message::CloseDev).unwrap();
    });

    while let Ok(message) = receiver.recv().await {
        if !builder.config.hmr {
            if let Some(mut child) = child.take() {
                if let Err(err) = child.kill().await {
                    eprintln!("{}\n{}", err, "Failed to kill love2d process.".red());
                } else if let Message::CloseLove = message {
                    println!("{} Closed love.", "[+]".blue());
                };
            }
        }
        if let Message::CloseDev = message {
            exit(0);
        }
        if let Message::BuildProject(change) = message {
            if builder.config.hmr
                && let Some(files) = &change
            {
                builder.add_assets(None).await;
                for file in files {
                    builder._transpile_files(&file, &builder.paths.build).await;
                }
            } else {
                builder.clean_build_folder().await.unwrap();
                let modules = builder.transpile().await;
                builder.add_assets(None).await;
                builder.handle_conf_file(modules).await;
            }

            if !builder.config.hmr {
                if let Some(mut child) = child.take() {
                    child.kill().await.unwrap();
                }

                child = Some(
                    Command::new(&path)
                        .current_dir(&path.parent().unwrap())
                        .arg(&builder.paths.build)
                        .spawn()
                        .context("Spawning the process")
                        .unwrap(),
                );
            } else if let None = child {
                child = Some(
                    Command::new(&path)
                        .current_dir(&path.parent().unwrap())
                        .arg(&builder.paths.build)
                        .spawn()
                        .context("Spawning the process")
                        .unwrap(),
                );
                // In here we already assumed that there is a child and there is hmr
            } else if let Some(files) = &change {
                if server.is_none() {
                    server = Some(
                        DevServer::new("127.0.0.1:9532".to_owned())
                            .await
                            .expect("Failed to start dev server"),
                    );
                }
                if let Some(server) = &mut server {
                    server
                        .dispatch(
                            "update",
                            files
                                .iter()
                                .map(|x| {
                                    x.strip_prefix(&builder.paths.src)
                                        .expect("Invalid prefix path, report this error on github.")
                                        .with_extension("lua")
                                        .to_string_lossy()
                                        .to_string()
                                })
                                .join(",")
                                .as_bytes()
                                .to_vec(),
                        )
                        .await
                        .expect("Failed to dispatch update");
                }
            } else {
                if server.is_none() {
                    server = Some(
                        DevServer::new("127.0.0.1:9532".to_owned())
                            .await
                            .expect("Failed to start dev server"),
                    );
                }
                if let Some(server) = &mut server {
                    server
                        .dispatch("update", b"main.lua".to_vec())
                        .await
                        .expect("Failed to dispatch update");
                }
            }
        }
    }
}
