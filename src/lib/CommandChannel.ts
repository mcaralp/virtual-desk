import { listen } from '@tauri-apps/api/event'

export interface CommandResponseData
{
    last: boolean;
    data: any;
}

export type CommandResponseResult = { Ok: CommandResponseData; } | { Err: string; };

export interface CommandResponse
{
    uuid: string;
    result: CommandResponseResult;
}

type ResponseHandler = (event: CommandResponse) => void;

const handlers = new Map<string, ResponseHandler>();
let listening: Promise<void> | undefined;

// A single shared listener dispatches by uuid, instead of one listen()/unlisten() per command,
// which avoids races between one command's unlisten() and another command's in-flight emit.
function ensureListening(): Promise<void>
{
    if (!listening)
    {
        // Cache the in-flight promise, not just the resolved value, so concurrent
        // callers await the same registration instead of racing past it.
        listening = listen<CommandResponse>('command-response', event =>
        {
            handlers.get(event.payload.uuid)?.(event.payload);
        }).then(() => {});
    }
    return listening;
}

export async function subscribe(uuid: string, handler: ResponseHandler): Promise<void>
{
    await ensureListening();
    handlers.set(uuid, handler);
}

export function unsubscribe(uuid: string): void
{
    handlers.delete(uuid);
}
