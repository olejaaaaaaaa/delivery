use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static VFS: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();

fn vfs() -> &'static Mutex<Vec<PathBuf>> {
    VFS.get_or_init(|| Mutex::new(vec![]))
}

pub fn mount_vfs(path: impl AsRef<Path>) {
    vfs().lock().unwrap().push(path.as_ref().to_path_buf());
}

pub fn open_vfs(path: impl AsRef<Path>) -> PathBuf {
    for mount in vfs().lock().unwrap().iter() {
        let full = mount.join(path.as_ref());
        if full.exists() {
            return full;
        }
    }
    panic!("Not found file: {:?}", path.as_ref().as_os_str())
}