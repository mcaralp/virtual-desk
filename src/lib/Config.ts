export interface CellConfig
{
    x: number
    y: number
    width: number
    height: number
    widget: string
}

export interface GridConfig
{
    columns: number
    rows: number
    cells: CellConfig[]
}

export interface PageConfig
{
    id: string
    grid: GridConfig
}

export interface WidgetConfig
{
    id: string
    type: string
    params: any
}

export interface PositionConfig
{
    x: string
    y: string
}

export interface WindowConfig
{
    screen: number
    width: string
    height: string
    position: PositionConfig
    origin: PositionConfig
    rotation: number
    transparent: boolean
    decorations: boolean
    pinned: boolean
}

export interface Host
{
    address: string
    port: number
}

export interface LocalConfig
{
    window: WindowConfig
    pages: PageConfig[]
    widgets: WidgetConfig[]
}

export interface TcpClientConfig
{
    host: Host
    window: WindowConfig
    pages: PageConfig[]
    widgets: WidgetConfig[]
}

export interface SshClientConfig
{
    host: Host
    private_key_path: string
    server_public_key_path: string
    window: WindowConfig
    pages: PageConfig[]
    widgets: WidgetConfig[]
}

export type AppConfig = { version: number } & (
    | { mode: 'local', settings: LocalConfig }
    | { mode: 'sshclient', settings: SshClientConfig }
    | { mode: 'tcpclient', settings: TcpClientConfig }
)

export function defaultConfig(): AppConfig
{
    return {
        version: 1,
        mode: 'local',
        settings: {
            window: {
                screen: 0,
                width: '400px',
                height: '400px',
                position: { x: '50%', y: '50%' },
                origin: { x: '50%', y: '50%' },
                rotation: 0,
                transparent: false,
                decorations: true,
                pinned: false
            },
            pages: [],
            widgets: []
        }
    }
}
