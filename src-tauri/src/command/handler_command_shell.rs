use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::{HandlerContext, util};
use crate::error::Error;

#[derive(Debug, Deserialize)]
struct ShellCommandParams
{
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ShellCommandResponse
{
    pub data: String,
}

pub async fn handler_shell(com: &HandlerContext, id: &str, params: &serde_json::Value)
    -> Result<(), Error>
{
    let params: ShellCommandParams = serde_json::from_value(params.clone())?;

    let mut child = Command::new(&params.command)
        .args(&params.args)
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = child.stdout.take()
        .ok_or_else(|| Error::ChildProcessNoStdout)?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    loop
    {
        let read = util::cancellable(com, id, reader.read_line(&mut line))
            .await??;
        if read == 0
        {
            break;
        }

        let data = line.clone();
        com.emit(id, false, &serde_json::to_value(ShellCommandResponse { data })?).await?;
        line.clear();
    }

    com.emit(id, true, &serde_json::Value::Null).await?;
    Ok(())
}
