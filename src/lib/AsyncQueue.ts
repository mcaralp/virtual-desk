type WaiterCallbacks<T> = {
    resolve: (data: T) => void,
    reject: (error: Error) => void
}

// Buffers pushed values/errors until read, and tracks whether more will come.
export class AsyncQueue<T>
{
    private readonly buffer: (T | Error)[] = []
    private readonly waiters: WaiterCallbacks<T>[] = []
    private closed = false

    push(value: T | Error): void
    {
        const waiter = this.waiters.shift()
        if (waiter)
        {
            value instanceof Error ? waiter.reject(value) : waiter.resolve(value)
            return
        }
        this.buffer.push(value)
    }

    close(): void
    {
        this.closed = true
    }

    isClosed(): boolean
    {
        return this.closed
    }

    // True once closed and every buffered value has been consumed.
    isDone(): boolean
    {
        return this.closed && this.buffer.length === 0
    }

    next(): Promise<T>
    {
        const value = this.buffer.shift()
        if (value !== undefined)
        {
            return value instanceof Error ? Promise.reject(value) : Promise.resolve(value)
        }
        if (this.closed)
        {
            return Promise.reject(new Error('Queue is closed'))
        }
        return new Promise((resolve, reject) => this.waiters.push({ resolve, reject }))
    }
}
