//! Child-process-only configuration isolation; never mutates the test runner's environment.
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct TestHome(PathBuf);

pub fn temp_home(name: &str) -> TestHome {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    loop {
        let path = std::env::temp_dir().join(format!(
            "solunatus-cli-{}-{}-{}",
            name,
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return TestHome(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("cannot create isolated config directory: {error}"),
        }
    }
}

impl TestHome {
    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_solunatus"));
        command
            .env("HOME", &self.0)
            .env("SOLUNATUS_CONFIG_DIR", &self.0)
            .env("SOLUNATUS_SKIP_TIME_SYNC", "1");
        command
    }
}

impl std::ops::Deref for TestHome {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TestHome {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
