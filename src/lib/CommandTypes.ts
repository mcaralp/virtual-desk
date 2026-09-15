import { type AppConfig } from './Config'

export enum CommandType {
    ShellCommand = 1,
    ConfigCommand = 2,
    CancelCommand = 3
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
    [CommandType.ConfigCommand]: null
    [CommandType.CancelCommand]: CancelCommandParams
}

export type CommandResponseMapping = {
    [CommandType.ShellCommand]: ShellCommandResponse | null
    [CommandType.ConfigCommand]: AppConfig
    [CommandType.CancelCommand]: null
}

export interface CommandRequest<T extends CommandType>
{
    cmd: T
    uuid: string
    params: CommandParamsMapping[T]
}
