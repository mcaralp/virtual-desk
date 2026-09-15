mod com;
mod config;
mod command;

pub fn get_config_path() -> String
{
    let args: Vec<String> = std::env::args().collect();
    if let Some(path) = args.get(1)
    {
        return path.clone();
    }
    dirs::home_dir()
        .map(|h| h.join(".virtualdeck/config.yaml").to_string_lossy().into_owned())
        .expect("Unable to determine home directory")
}

fn setup(app: &mut tauri::App)
    -> Result<(), Box<dyn std::error::Error>>
{
    let config_watcher = config::ConfigWatcher::new(&get_config_path());
    config_watcher.start()?;
    let command_state = command::CommandState::new(app.handle().clone());
    command_state.start();

    com::setup(config_watcher, command_state)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run()
{
    tauri::Builder::default()
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            command::exec_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
