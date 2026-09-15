import { invoke } from '@tauri-apps/api/core'
import { v4 as uuidv4 } from 'uuid'
import { subscribe, unsubscribe, type CommandResponse } from './CommandChannel'
import { AsyncQueue } from './AsyncQueue'
import { CommandType, type CommandRequest, type CommandParamsMapping, type CommandResponseMapping } from './CommandTypes'

export class Command<T extends CommandType>
{
    public readonly uuid: string
    private readonly queue = new AsyncQueue<CommandResponseMapping[T]>()

    private constructor()
    {
        this.uuid = uuidv4()
    }

    static async init<T extends CommandType>(command: T, params: CommandParamsMapping[T]): Promise<Command<T>>
    {
        const com = new Command<T>()
        await com.start(command, params)
        return com
    }

    async read(): Promise<CommandResponseMapping[T]>
    {
        return this.queue.next()
    }

    isFinished(): boolean
    {
        return this.queue.isDone()
    }

    async cancel(): Promise<void>
    {
        if(this.queue.isClosed()) return

        try
        {
            console.log('cancel')
            await callCommand(CommandType.CancelCommand, {uuid: this.uuid})
        }
        catch
        {
            // Best-effort: the command may already have finished server-side.
        }
    }

    private finish(): void
    {
        if (this.queue.isClosed()) return
        unsubscribe(this.uuid)
        this.queue.close()
    }

    private async start(command: T, params: CommandParamsMapping[T]): Promise<void>
    {
        await subscribe(this.uuid, event => this.handle(event))

        const request: CommandRequest<T> = { cmd: command, uuid: this.uuid, params }

        try
        {
            await invoke('exec_command', { command: request })
        }
        catch (error)
        {
            this.finish()
            throw error
        }
    }

    private handle(event: CommandResponse): void
    {
        console.log(event)
        if ('Ok' in event.result)
        {
            this.queue.push(event.result.Ok.data)
            if (event.result.Ok.last)
            {
                this.finish()
            }
        }
        else
        {
            this.queue.push(new Error(event.result.Err))
            this.finish()
        }
    }
}

async function callCommand<T extends CommandType>(command: T, params: CommandParamsMapping[T]): Promise<CommandResponseMapping[T][]>
{
    const com = await Command.init(command, params)
    const results: CommandResponseMapping[T][] = []
    while (!com.isFinished())
    {
        const result = await com.read()
        results.push(result)
    }
    return results
}
