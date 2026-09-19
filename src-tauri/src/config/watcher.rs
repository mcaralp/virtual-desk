use std::sync::Mutex;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use tokio::sync::broadcast;

use super::{Error, AppConfig, util};

pub struct ConfigReceiver
{
    rx: broadcast::Receiver<()>,
    config_path: String,
}

impl Clone for ConfigReceiver
{
    fn clone(&self) -> Self
    {
        Self { rx: self.rx.resubscribe(), config_path: self.config_path.clone() }
    }
}

impl ConfigReceiver
{
    pub async fn recv(&mut self) -> Result<(), Error>
    {
        match self.rx.recv().await
        {
            Ok(()) => Ok(()),
            // A lag means change notifications were dropped; treat it as a change.
            Err(broadcast::error::RecvError::Lagged(_)) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn read_config(&self) -> Result<AppConfig, Error>
    {
        util::read_config(&self.config_path)
    }
}

pub struct ConfigWatcher
{
    debouncer: Mutex<Option<Debouncer<RecommendedWatcher, RecommendedCache>>>,
    tx: broadcast::Sender<()>,
    config_path: String,
}

impl Drop for ConfigWatcher
{
    fn drop(&mut self)
    {
        self.stop();
    }
}

impl ConfigWatcher
{
    pub fn new(config_path: &str) -> Self
    {
        let (tx, _) = broadcast::channel(16);
        Self { debouncer: Mutex::new(None), tx, config_path: config_path.to_string() }
    }

    pub fn read_config(&self) -> Result<crate::config::app::AppConfig, Error>
    {
        util::read_config(&self.config_path)
    }

    pub fn start(&self) -> Result<(), Error>
    {
        let tx = self.tx.clone();
        // Absolute path so the parent directory is always well-defined, even for a bare filename.
        let config_abs = std::path::absolute(&self.config_path)?;
        // Match events by file name so atomic saves (rename into place) are detected.
        let config_file_name = config_abs.file_name().map(std::ffi::OsString::from);

        let mut debouncer = new_debouncer(
            Duration::from_millis(100),
            None,
            move |result: DebounceEventResult|
        {
            if let Ok(events) = result
            {
                let changed = events.iter().any(|event| {
                    matches!(event.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_) | notify::EventKind::Remove(_))
                        && event.paths.iter().any(|p| {
                            p.file_name() == config_file_name.as_deref()
                        })
                });

                if changed
                {
                    let _ = tx.send(());
                }
            }
        })?;

        // Watch the parent directory so atomic saves that replace the file are still tracked.
        let watch_dir = config_abs.parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        debouncer.watch(&watch_dir, RecursiveMode::NonRecursive)?;

        *self.debouncer.lock().unwrap() = Some(debouncer);
        Ok(())
    }

    pub fn subscribe(&self)
        -> ConfigReceiver
    {
        ConfigReceiver {
            rx: self.tx.subscribe(),
            config_path: self.config_path.clone(),
        }
    }

    pub fn stop(&self)
    {
        if let Some(debouncer) = self.debouncer.lock().unwrap().take()
        {
            debouncer.stop();
        }
    }
}

