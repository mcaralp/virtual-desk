/// RAII guard that keeps the hidden main window alive and closes it on drop.
use tokio::sync::watch;

pub struct WindowGuard
{
    window: tauri::WebviewWindow,
    closed: watch::Receiver<bool>,
}

impl WindowGuard
{
    pub fn new(app: &tauri::AppHandle) -> tauri::Result<Self>
    {
        let window = tauri::WebviewWindowBuilder::new(
        app,
            "main",
            tauri::WebviewUrl::App("index.html".into())
        ).title("widgets")
            .visible(false)
            .build()?;

        let (closed_tx, closed_rx) = watch::channel(false);

        let closed_for_event = closed_tx.clone();

        window.on_window_event(move |event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                let _ = closed_for_event.send(true);
            }
        });

        Ok(Self { window, closed: closed_rx })
    }

    pub async fn stop(&self) -> tauri::Result<()>
    {
        if !*self.closed.borrow()
        {
            self.window.close()?;
        }

        let mut closed = self.closed.clone();

        while !*closed.borrow()
        {
            if closed.changed().await.is_err()
            {
                break;
            }
        }

        Ok(())
    }
}
