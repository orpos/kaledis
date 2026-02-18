use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use async_watcher::AsyncDebouncer;
use colored::Colorize;
use console::Term;
use tokio::{
    process::Child,
    sync::broadcast::{Sender, channel},
};

use crate::{
    android::DevServer,
    commands::build::{Builder, Strategy},
    utils::relative,
};

async fn spawn_file_reader(
    watching: Arc<RwLock<bool>>,
    local: &PathBuf,
    assets_paths: Vec<PathBuf>,
    sender: Sender<Message>,
) {
    let root = local.clone();
    tokio::spawn(async move {
        let (mut c, mut r) = AsyncDebouncer::new_with_channel(
            Duration::from_millis(20),
            Some(Duration::from_millis(20)),
        )
        .await
        .unwrap();

        c.watcher()
            .watch(&root, async_watcher::notify::RecursiveMode::Recursive)
            .unwrap();
        while let Some(Ok(data)) = r.recv().await {
            {
                if !*watching.read().unwrap() {
                    continue;
                }
            }
            if data
                .iter()
                .filter(|x| !x.path.starts_with(root.join(".build")))
                .collect::<Vec<_>>()
                .len()
                < 1
            {
                continue;
            }
            let assets: Vec<_> = {
                data.iter()
                    .map(|x| x.path.clone())
                    .filter(|x| assets_paths.iter().any(|y| x.starts_with(y)))
                    .collect()
            };
            let len = assets.len();
            if assets.len() > 0 {
                sender.send(Message::SendAssets(assets)).unwrap();
            }
            if data.len() - len > 0 {
                sender.send(Message::BuildProject).unwrap();
            }
        }
    });
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Message {
    CloseLove,
    BuildProject,
    SendAssets(Vec<PathBuf>),
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

pub async fn watch(base_path: Option<PathBuf>, ip: String) {
    let con = ip + ":9532";
    let mut android_dev_server = DevServer::new(con).await.unwrap();
    let root = relative(base_path.clone());
    println!("Watching...");
    println!("Press [L] if you want to build manually");
    println!("Press [A] if you want to toggle between auto build and manual mode.");
    println!("Press [Q] if you want to close the dev server.");
    println!("Press [Esc] if you want to close Love.");

    if !root.join("kaledis.toml").exists() {
        eprintln!("{}", "No project found!".red());
        return;
    }

    // This is currently not available for android since we use a custom hmr implementation
    let mut builder = Builder::new(root.clone(), Strategy::BuildDev, true).await;
    builder.config.hmr = false;


    let watching = Arc::new(RwLock::new(false));
    let (sender, mut receiver) = channel::<Message>(2);

    spawn_keyboard_handler(Arc::clone(&watching), sender.clone()).await;
    spawn_file_reader(
        watching,
        &root,
        builder
            .config
            .layout
            .bundle
            .iter()
            .chain(builder.config.layout.external.iter())
            .map(|x| PathBuf::from(x))
            .collect(),
        sender.clone(),
    )
    .await;

    let mut child: Option<Child> = None;
    builder.clean_build_folder().await.unwrap();

    android_dev_server.clean_assets().await.unwrap();

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
            break;
        }
        if let Message::BuildProject = message {
            android_dev_server.report_loading().await.unwrap();
            builder.transpile().await;
            let file_contents = tokio::fs::read_to_string(builder.paths.build.join("main.lua"))
                .await
                .unwrap();
            android_dev_server
                .send_code(file_contents.as_bytes().to_vec())
                .await
                .unwrap();
        }
        if let Message::SendAssets(assets) = message {
            for path in assets {
                let contents = tokio::fs::read(&path).await.unwrap();
                println!("{:?}", path);
                android_dev_server
                    .send_asset(
                        &path
                            .strip_prefix(&builder.paths.root)
                            .unwrap()
                            .to_path_buf(),
                        contents,
                    )
                    .await
                    .unwrap();
            }
        }
    }
}
