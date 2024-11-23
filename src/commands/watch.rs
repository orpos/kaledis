use std::{ path::PathBuf, time::Duration };

use async_watcher::AsyncDebouncer;
use colored::Colorize;
use tokio::process::{ Child, Command };

use crate::{ commands::build, toml_conf::Config, utils::relative };

async fn run(
    local: &PathBuf,
    love_path: &PathBuf,
    love_executable: &PathBuf,
    path: Option<PathBuf>
) -> Child {
    build::build(path, build::Strategy::Build).await.unwrap();
    Command::new(love_executable)
        .current_dir(love_path)
        .arg(local.join(".build").join("final.love"))
        .spawn()
        .unwrap()
}

pub async fn watch_folder(path: Option<PathBuf>) {
    let local = relative(path.clone());

    if !local.join("kaledis.toml").exists() {
        println!("{}", "No project found!".red());
        return;
    }

    let configs = Config::from_toml_file(local.join("kaledis.toml")).unwrap();
    let love_path = configs.project.love_path;
    let love_executable = love_path.join("love.exe");

    let (mut debouncer, mut file_events) = AsyncDebouncer::new_with_channel(
        Duration::from_secs(1),
        Some(Duration::from_secs(1))
    ).await.unwrap();

    debouncer.watcher().watch(&local, async_watcher::notify::RecursiveMode::Recursive).unwrap();

    let mut child: Child = run(&local, &love_path, &love_executable, path.clone()).await;
    while let Some(Ok(data)) = file_events.recv().await {
        if
            data
                .iter()
                .find(|x| {
                    return !x.path.starts_with(local.join(".build"));
                })
                .is_none()
        {
            continue;
        }
        // Killing the child process
        child.kill().await.expect("child not dead?");

        println!("Building project...");
        child = run(&local, &love_path, &love_executable, path.clone()).await;
    }
}
