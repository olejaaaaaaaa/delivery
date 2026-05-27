use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static VFS: OnceLock<Mutex<Vec<(PathBuf, PathBuf)>>> = OnceLock::new();

fn vfs() -> &'static Mutex<Vec<(PathBuf, PathBuf)>> {
    VFS.get_or_init(|| Mutex::new(vec![]))
}

pub fn mount_vfs(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
    vfs().lock().unwrap().push((
        src.as_ref().to_path_buf(),
        dst.as_ref().to_path_buf(),
    ));
}

pub fn open_vfs(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();

    for (src, dst) in vfs().lock().unwrap().iter() {
        if let Ok(relative) = path.strip_prefix(src) {
            let candidate = dst.join(relative);
            if candidate.exists() {
                return candidate;
            }
        } 
    }

    path.to_path_buf()
}