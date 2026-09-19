mod command_request;
mod command_response;
mod command_bus;
mod command_bus_receiver;
mod handler_context;
mod command_dispatch;
mod command_type;
mod handler_command_cancel;
mod handler_command_read_config;
mod handler_command_shell;
mod handler_command_watch_config;
mod util;

use tauri::{AppHandle, Manager};

pub use command_request::CommandRequest;
pub use command_response::{CommandResponse, CommandResponseData};
pub use command_bus::CommandBus;
pub use command_bus_receiver::CommandBusReceiver;
pub use handler_context::HandlerContext;
pub use command_dispatch::CommandDispatch;

#[tauri::command]
pub fn exec_command(command: CommandRequest, app: AppHandle)
    -> Result<(), String>
{
    let state = app.state::<CommandBus>();
    // no error if there are currently no subscribers
    let _ = state.send(command);
    Ok(())
}
