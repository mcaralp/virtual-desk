use std::sync::Mutex;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use tokio::sync::broadcast;

use crate::config::Error;
use crate::config::util;

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
        let _ = self.rx.recv().await?;
        Ok(())
    }

    pub fn read_config(&self) -> Result<crate::config::app::AppConfig, Error>
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

impl ConfigWatcher
{
    pub fn new(config_path: &str) -> Self
    {
        let (tx, _) = broadcast::channel(16);
        Self { debouncer: Mutex::new(None), tx, config_path: config_path.to_string() }
    }

    pub fn start(&self) -> Result<(), Error>
    {
        let tx = self.tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(100),
            None,
            move |result: DebounceEventResult|
        {
            if let Ok(events) = result
            {
                let changed = events.iter().any(|event| {
                    matches!(event.kind, notify::EventKind::Create(_) | notify::EventKind::Modify(_))
                });

                if changed
                {

                    let _ = tx.send(());
                }
            }
        })?;

        debouncer.watch(&self.config_path, RecursiveMode::NonRecursive)?;

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
