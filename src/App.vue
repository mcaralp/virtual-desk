<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue"
import { availableMonitors, getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { type AppConfig, defaultConfig } from './lib/Config.ts'
import { Command } from './lib/Command'
import { CommandType } from './lib/CommandTypes'
import Grid from './Grid.vue'


function parseValue(value: string, reference: number): number
{
    const num = parseFloat(value)
    if (value.endsWith('%'))
    {
        return num / 100 * reference
    }

    return num
}

// Rotates a point (x, y) taken from a `width` x `height` rect by `rotation` degrees (clockwise)
function rotatePoint(x: number, y: number, width: number, height: number, rotation: number): { x: number, y: number }
{
    switch (rotation)
    {
        case 90:  return { x: y, y: width - x }
        case 180: return { x: width - x, y: height - y }
        case 270: return { x: height - y, y: x }
        default:  return { x, y }
    }
}

const config = ref<AppConfig | null>(null)
const error = ref<string | null>(null)

const windowConfig = computed(() => {
    if (config.value === null) return null
    return config.value.settings.window
})

const pagesConfig = computed(() => {
    if (config.value === null) return []
    return config.value.settings.pages
})

const widgetsConfig = computed(() => {
    if (config.value === null) return []
    return config.value.settings.widgets
})

const rotation = computed(() => {
    if (!windowConfig.value) return 0
    return ((windowConfig.value.rotation % 360) + 360) % 360  
})

watch(config, async (newConfig: AppConfig | null) => {
    if (newConfig === null) return 
  
    const monitors = await availableMonitors()

    const windowConfig = newConfig.settings.window
    if(windowConfig.screen >= monitors.length)
    {
        throw new Error(`Invalid screen index: ${windowConfig.screen}. Only ${monitors.length} monitors available.`)
    }
    const monitor = monitors[windowConfig.screen]
    if (!monitor) throw new Error("No monitor available")

    const rotation = ((windowConfig.rotation % 360) + 360) % 360
    const swapped = rotation === 90 || rotation === 270

    const monitorW = monitor.size.width / monitor.scaleFactor
    const monitorH = monitor.size.height / monitor.scaleFactor
    const monitorX = monitor.position.x / monitor.scaleFactor
    const monitorY = monitor.position.y / monitor.scaleFactor

    // Monitor and window dimensions as seen in the configured (visual) orientation
    const visualMonitorW = swapped ? monitorH : monitorW
    const visualMonitorH = swapped ? monitorW : monitorH
    const visualWidth = parseValue(windowConfig.width, visualMonitorW)
    const visualHeight = parseValue(windowConfig.height, visualMonitorH)

    // Real window size to apply, axes swap for 90/270
    const width  = swapped ? visualHeight : visualWidth
    const height = swapped ? visualWidth  : visualHeight

    // Origin is a point within the window rect, position a point within the monitor rect;
    // both are configured in the visual orientation and need to be rotated back into real coordinates
    const { x: originX, y: originY } = rotatePoint(
        parseValue(windowConfig.origin.x, visualWidth),
        parseValue(windowConfig.origin.y, visualHeight),
        visualWidth, visualHeight, rotation
    )
    const { x: positionX, y: positionY } = rotatePoint(
        parseValue(windowConfig.position.x, visualMonitorW),
        parseValue(windowConfig.position.y, visualMonitorH),
        visualMonitorW, visualMonitorH, rotation
    )

    const posX = monitorX + positionX - originX
    const posY = monitorY + positionY - originY

    const win = getCurrentWindow()
    await win.setDecorations(windowConfig.decorations)
    await win.setSize(new LogicalSize(width, height))
    await win.setPosition(new LogicalPosition(posX, posY))

    // Check for invisible window borders on Windows and adjust position accordingly
    const scale = await win.scaleFactor()
    const outer = await win.outerPosition()
    const inner = await win.innerPosition()
    const borderX = (inner.x - outer.x) / scale
    const borderY = (inner.y - outer.y) / scale
    if (borderX !== 0 || borderY !== 0)
    {
        await win.setPosition(new LogicalPosition(posX - borderX, posY - borderY))
    }

    await win.setAlwaysOnTop(windowConfig.pinned)
    await win.show()
});

const mainStyle = computed(() => {
    const settings = config.value?.mode === 'local' ? config.value.settings : null
    const transparent = settings?.window.transparent ?? false
    const color = `rgba(247, 248, 250, ${transparent ? 0 : 1})`
    const opacity = transparent ? 0.8 : 1
    const clockwise = 360 - rotation.value
    return {
        background: color,
        opacity: opacity,
        width: clockwise % 180 === 0 ? '100vw' : '100vh',
        height: clockwise % 180 === 0 ? '100vh' : '100vw',
        transform: `rotate(${clockwise}deg)`
    }
})

function sleep(ms: number)
{
    return new Promise(resolve => setTimeout(resolve, ms))
}

async function readConfig(firstLoad: boolean)
{
    while(true)
    {
        try
        {
            const configRequest = await Command.init(CommandType.ReadConfigCommand, null)
            config.value = await configRequest.read()
            error.value = null
            break
        }
        catch (e)
        {
            console.log(e)
            if(e instanceof Error)
            {
                error.value = e.message;
            }
            if(firstLoad)
            {
                config.value = defaultConfig()
                firstLoad = false
            }
        }

        await sleep(1000)
    }
}

onMounted(async () => {

    let firstLoad = true
    while(true)
    {
        await readConfig(firstLoad)

        try
        {
            const watchConfigRequest = await Command.init(CommandType.WatchConfigCommand, null)
            while(!watchConfigRequest.isFinished())
            {
                config.value = await watchConfigRequest.read();
            }
        }
        catch (e)
        {
            if(e instanceof Error)
            {
                error.value = e.message;
            }
        }
    }
})

</script>

<template>
    <main class="container" :style="mainStyle">
        <div v-if="error !== null" class="error">
            <div class="text">Failed to load configuration<br/>{{ error }}</div>
        </div>
        <Grid v-else-if="pagesConfig.length > 0" :config="pagesConfig[0]" :widgets="widgetsConfig" />
    </main>
</template>

<style scoped>
.container
{
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
}

.error
{
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
}

.error .text
{
    box-sizing: border-box;
    border: 1px solid #dfe3e8;
    border-radius: 8px;
    background: #ffffff;
    padding: 20px;
    text-align: center;
    line-height: 30px;
}
</style>

<style>
html, body
{
    margin: 0;
    width: 100%;
    height: 100%;
}

#app
{
    margin: 0;
    width: 100%;
    height: 100%;
    display: grid;
    justify-content: center;
    align-items: center;
}
</style>
