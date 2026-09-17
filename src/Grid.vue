<script setup lang="ts">
import { computed } from "vue";
import { type PageConfig, type WidgetConfig, type CellConfig } from "./lib/Config";

const props = defineProps<{ config: PageConfig, widgets: WidgetConfig[] }>()

const gridStyle = computed(() => ({
    gridTemplateColumns: `repeat(${props.config.grid.columns}, 1fr)`,
    gridTemplateRows: `repeat(${props.config.grid.rows}, 1fr)`,
}))

const cellId = (cell: CellConfig): string => `${cell.x}-${cell.y}-${cell.widget}`;
const cellStyle = (cell: CellConfig): Record<string, string> => ({
    gridColumn: `${cell.x + 1} / span ${cell.width}`,
    gridRow: `${cell.y + 1} / span ${cell.height}`,
});

// function getWidget(id: string)
// {
//     return props.widgets.find(widget => widget.id === id);
// }
</script>

<template>
    <div class="grid" :style="gridStyle">
        <div v-for="cell in props.config.grid.cells" :key="cellId(cell)" class="cell" :style="cellStyle(cell)">
            test
        </div>
    </div>
</template>

<style scoped>
.grid
{
    display: grid;
    width: 100%;
    height: 100%;
    gap: 8px;
    padding: 8px;
    box-sizing: border-box;
}

.cell
{
    min-width: 0;
    min-height: 0;
    box-sizing: border-box;
    border: 1px solid #dfe3e8;
    border-radius: 8px;
    background: #ffffff;
    padding: 10px;
}

.widget
{
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    border: 1px solid #444;
    padding: 8px;
}
</style>
