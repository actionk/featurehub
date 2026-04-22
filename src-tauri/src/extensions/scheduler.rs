use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::extensions::manifest::ScheduleDecl;
use crate::extensions::script_runner::{self, ScriptInput};

pub struct ScheduleHandle {
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
    pub join: Option<std::thread::JoinHandle<()>>,
}

impl ScheduleHandle {
    /// Cancel and wait for the thread to exit. Equivalent to dropping the handle,
    /// but explicit when you need to be sure cleanup completed before continuing.
    pub fn cancel_and_join(mut self) {
        // Drop will handle the actual cancel + join; this just consumes self.
        self.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for ScheduleHandle {
    fn drop(&mut self) {
        self.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

struct InFlightGuard(Arc<Mutex<bool>>);
impl Drop for InFlightGuard {
    fn drop(&mut self) {
        if let Ok(mut g) = self.0.lock() {
            *g = false;
        }
    }
}

pub fn spawn_schedule(
    extension_id: String,
    extension_dir: PathBuf,
    schedule: ScheduleDecl,
    db_path: String,
    storage_path: String,
) -> ScheduleHandle {
    let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_flag = cancel.clone();
    let in_flight = Arc::new(Mutex::new(false));
    let interval = Duration::from_secs(schedule.interval_secs.max(60));

    let join = std::thread::spawn(move || {
        let mut next_tick = Instant::now() + interval;
        while !cancel_flag.load(std::sync::atomic::Ordering::Relaxed) {
            let now = Instant::now();
            if now >= next_tick {
                // Try to acquire in-flight lock; skip tick if busy.
                // _guard resets the flag on drop, so a handler panic won't stall the schedule.
                let _guard = {
                    let mut g = match in_flight.try_lock() {
                        Ok(g) => g,
                        Err(_) => {
                            next_tick = now + interval;
                            std::thread::sleep(Duration::from_millis(500));
                            continue;
                        }
                    };
                    if *g {
                        next_tick = now + interval;
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                    *g = true;
                    drop(g);
                    InFlightGuard(in_flight.clone())
                };

                let handler_path = extension_dir.join(&schedule.handler);
                let input = ScriptInput {
                    params: Default::default(),
                    db_path: db_path.clone(),
                    storage_path: storage_path.clone(),
                    feature_id: None,
                };
                let ext_id = extension_id.clone();
                match script_runner::run_blocking_with_notifications(&handler_path, &input, 60) {
                    Ok(r) => script_runner::forward_notifications(&ext_id, r.notifications),
                    Err(e) => eprintln!("[ext:{}] schedule '{}' failed: {}", ext_id, schedule.id, e),
                }
                // No explicit reset — _guard drops at end of iteration.
                next_tick = Instant::now() + interval;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    ScheduleHandle { cancel, join: Some(join) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_then_cancel_does_not_hang() {
        // Use a non-existent handler — script_runner returns an error, scheduler logs and continues.
        let schedule = ScheduleDecl {
            id: "t".into(),
            handler: "nonexistent.js".into(),
            interval_secs: 60,
            enabled_setting: None,
        };
        let handle = spawn_schedule(
            "test".into(),
            std::env::temp_dir(),
            schedule,
            String::new(),
            std::env::temp_dir().to_string_lossy().to_string(),
        );
        std::thread::sleep(Duration::from_millis(100));
        handle.cancel_and_join();
    }
}
