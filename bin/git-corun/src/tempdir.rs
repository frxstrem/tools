use std::path::{Path, PathBuf};
use std::{io,env,fs};

pub struct TempDir {
    path: PathBuf,
    autoclean: bool,
}

impl TempDir {
    pub fn new(path: impl AsRef<Path>, autoclean: bool) -> io::Result<TempDir> {
        fs::create_dir_all(&path)?;
        let path = path.as_ref().to_path_buf();
        Ok(TempDir { path, autoclean })
    }

    pub fn new_temp(name: impl AsRef<Path>) -> io::Result<TempDir> {
        let path = env::temp_dir().join(name);
        TempDir::new(path, true)
    }

    fn clean(&mut self) -> io::Result<()> {
        if self.autoclean {
            fs::remove_dir_all(&self.path)
        } else {
            Ok(())
        }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        self.clean().expect("TempDir::clean")
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path { &self.path }
}
