/// RAII guard that keeps the hidden main window alive and closes it on drop.
pub struct WindowGuard {
    window: tauri::WebviewWindow,
}

impl WindowGuard
{
    pub fn new(app: tauri::AppHandle) -> tauri::Result<Self>
    {
        let window = tauri::WebviewWindowBuilder::new(
        &app,
            "main",
            tauri::WebviewUrl::App("index.html".into())
        ).title("widgets")
            .visible(false)
            .build()?;

        Ok(Self { window })
    }
}

impl Drop for WindowGuard {
    fn drop(&mut self) {
        let _ = self.window.close();
    }
}
