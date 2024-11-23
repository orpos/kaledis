use std::{ env, path::PathBuf };

pub fn relative(path: Option<PathBuf>) -> PathBuf {
    let ma = env::current_dir().unwrap();
    path.map(|x|ma.join(x)).unwrap_or(ma)
}
