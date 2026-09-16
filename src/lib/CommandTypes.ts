import { type AppConfig } from './Config'

export enum CommandType {
    ShellCommand = 1,
    ReadConfigCommand = 2,
    WatchConfigCommand = 3,
    CancelCommand = 4
}

interface ShellCommandParams
{
    command: string
    args: string[]
}

interface ShellCommandResponse
{
    data: string
}

interface CancelCommandParams
{
    uuid: string
}

export type CommandParamsMapping = {
    [CommandType.ShellCommand]: ShellCommandParams
    [CommandType.ReadConfigCommand]: null
    [CommandType.WatchConfigCommand]: null
    [CommandType.CancelCommand]: CancelCommandParams
}

export type CommandResponseMapping = {
    [CommandType.ShellCommand]: ShellCommandResponse | null
    [CommandType.ReadConfigCommand]: AppConfig
    [CommandType.WatchConfigCommand]: AppConfig
    [CommandType.CancelCommand]: null
}

export interface CommandRequest<T extends CommandType>
{
    cmd: T
    uuid: string
    params: CommandParamsMapping[T]
}
