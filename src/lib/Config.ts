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

export interface LocalConfig
{
    window: WindowConfig
    pages: PageConfig[]
    widgets: WidgetConfig[]
}

export interface Host
{
    address: string
    port: number
}

export interface TcpServerConfig
{
    host: Host
    window: WindowConfig
    pages: PageConfig[]
    widgets: WidgetConfig[]
}

export interface TcpClientConfig
{
    host: Host
}

export type AppConfig = { version: number } & (
    | { mode: 'local', settings: LocalConfig }
    | { mode: 'tcpserver', settings: TcpServerConfig }
    | { mode: 'tcpclient', settings: TcpClientConfig }
)
