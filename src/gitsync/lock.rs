//! The write lock: a file in `.git/` with an operating system file lock.
//! The lock goes away when the process ends, also after a crash, so it is never stale.

use crate::error::{Error, Result};
use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct Lock {
    file: File,
}

impl Lock {
    /// Take the lock. Waits up to `wait`, then fails with `sync_failed`.
    pub fn acquire(git_dir: &Path, wait: Duration) -> Result<Lock> {
        let path = git_dir.join("waymark.lock");
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| {
                Error::setup(format!("Cannot open the lock file {}: {e}", path.display()))
            })?;
        let start = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Lock { file }),
                Err(TryLockError::WouldBlock) => {
                    if start.elapsed() >= wait {
                        return Err(Error::sync(
                            "Another waymark call holds the lock on this memory repository.",
                        )
                        .with_next("Wait a moment and try again."));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(TryLockError::Error(e)) => {
                    return Err(Error::setup(format!("Cannot lock {}: {e}", path.display())))
                }
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_lock_waits_and_then_fails() {
        let dir = tempfile::tempdir().unwrap();
        let first = Lock::acquire(dir.path(), Duration::from_secs(1)).unwrap();
        let start = Instant::now();
        let e = Lock::acquire(dir.path(), Duration::from_millis(150))
            .err()
            .expect("must fail");
        assert!(start.elapsed() >= Duration::from_millis(150));
        assert_eq!(e.code, crate::error::Code::SyncFailed);
        drop(first);
        assert!(Lock::acquire(dir.path(), Duration::from_millis(150)).is_ok());
    }

    #[test]
    fn a_waiting_lock_succeeds_when_the_first_is_released() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let first = Lock::acquire(&path, Duration::from_secs(1)).unwrap();
        let p2 = path.clone();
        let h = std::thread::spawn(move || Lock::acquire(&p2, Duration::from_secs(3)).is_ok());
        std::thread::sleep(Duration::from_millis(200));
        drop(first);
        assert!(h.join().unwrap());
    }
}
