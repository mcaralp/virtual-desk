pub enum CommandType
{
    ShellCmd,
    ReadConfigCmd,
    WatchConfigCmd,
    CancelCmd
}

pub fn convert_command_type(cmd: u32) -> Option<CommandType>
{
    match cmd
    {
        1 => Some(CommandType::ShellCmd),
        2 => Some(CommandType::ReadConfigCmd),
        3 => Some(CommandType::WatchConfigCmd),
        4 => Some(CommandType::CancelCmd),
        _ => None,
    }
}
