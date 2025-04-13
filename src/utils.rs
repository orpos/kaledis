use std::{ env, path::PathBuf };

pub fn relative(path: Option<PathBuf>) -> PathBuf {
    let ma = env::current_dir().unwrap();
    path.map(|x| ma.join(x)).unwrap_or(ma)
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
