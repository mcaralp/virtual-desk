<script setup lang="ts">
import { ref, onMounted, computed } from "vue"
import { availableMonitors, getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { type AppConfig } from './lib/Config.ts'
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

const config = ref<AppConfig | null>(null)

async function updateConfig(newConfig: AppConfig)
{
    console.log(newConfig)
    config.value = newConfig

    if (newConfig.mode !== 'local') return

    const monitors = await availableMonitors()

    const windowConfig = newConfig.settings.window
    if(windowConfig.screen >= monitors.length)
    {
        throw new Error(`Invalid screen index: ${windowConfig.screen}. Only ${monitors.length} monitors available.`)
    }
    const monitor = monitors[windowConfig.screen]
    if (!monitor) throw new Error("No monitor available")

    const monitorW = monitor.size.width / monitor.scaleFactor
    const monitorH = monitor.size.height / monitor.scaleFactor
    const monitorX = monitor.position.x / monitor.scaleFactor
    const monitorY = monitor.position.y / monitor.scaleFactor

    const width  = parseValue(windowConfig.width,  monitorW)
    const height = parseValue(windowConfig.height, monitorH)

    const originX = parseValue(windowConfig.origin.x, width)
    const originY = parseValue(windowConfig.origin.y, height)

    const posX = monitorX + parseValue(windowConfig.position.x, monitorW) - originX
    const posY = monitorY + parseValue(windowConfig.position.y, monitorH) - originY

    const win = getCurrentWindow()
    await win.setSize(new LogicalSize(width, height))
    await win.setPosition(new LogicalPosition(posX, posY))
    await win.setDecorations(windowConfig.decorations)
    await win.setAlwaysOnTop(windowConfig.pinned)
    await win.show()
}

const mainStyle = computed(() => {
    const settings = config.value?.mode === 'local' ? config.value.settings : null
    const transparent = settings?.window.transparent ?? false
    const color = `rgba(247, 248, 250, ${transparent ? 0 : 1})`
    const opacity = transparent ? 0.8 : 1
    return {
        background: color,
        opacity: opacity,
    }
})

onMounted(async () => {
    const configRequest = await Command.init(CommandType.ConfigCommand, null)

    try
    {
        while(!configRequest.isFinished())
        {
            const newConfig = await configRequest.read();
            updateConfig(newConfig);
        }
    }
    catch (e) {
        console.error(e)
    }
})

</script>

<template>
    <main class="container" :style="mainStyle">
        <Grid v-if="config?.mode === 'local'" :config="config.settings.pages[0]" :widgets="config.settings.widgets" />
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
    width: 100vh;
    height: 100vw;
    transform: rotate(90deg);
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
    display: flex;
    justify-content: center;
    align-items: center;
}
</style>
