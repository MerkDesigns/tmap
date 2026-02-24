<script setup lang="ts">
import { computed } from 'vue'
type Theme = {
  appBg: string
  appText: string
  appTextMuted: string
  appTextSubtle: string
  canvasGrid: string
  canvasGridSize: string
  canvasGridDot: string
  canvasGridVisible: boolean
  canvasBorder: string
  panelBg: string
  panelBorder: string
  panelText: string
  panelTextMuted: string
  menuBg: string
  menuBorder: string
  menuHover: string
  inputBg: string
  inputBorder: string
  inputText: string
  inputPlaceholder: string
  buttonBg: string
  buttonBorder: string
  buttonText: string
  buttonHover: string
  buttonPrimaryBg: string
  buttonPrimaryText: string
  buttonPrimaryHover: string
  buttonDangerBg: string
  buttonDangerText: string
  buttonDangerHover: string
  buttonGhostBg: string
  buttonGhostBorder: string
  buttonGhostText: string
  buttonGhostHover: string
  dayBg: string
  dayBorder: string
  dayText: string
  dayHoverBg: string
  daySelectedBg: string
  daySelectedBorder: string
  daySelectedText: string
  dayTodayRing: string
  divider: string
  shadowStrong: string
  overlayBg: string
  clockSolid: string
  clockOutline: string
  clockTransparent: string
  clockSize: number
  clockOpacity: number
  containerBorderBrightness: number
  containerHeaderBrightness: number
  containerBorderSaturation: number
  containerHeaderSaturation: number
}

type ThemePreset = { id: string; name: string }

const props = defineProps<{ theme: Theme; basePresets: ThemePreset[]; userPresets: ThemePreset[]; selectedPresetId: string; themeName: string }>()
const emit = defineEmits<{
  (e: 'reset'): void
  (e: 'save', name: string): void
  (e: 'export'): void
  (e: 'import'): void
  (e: 'selectPreset', id: string): void
  (e: 'changed'): void
  (e: 'updateName', name: string): void
}>()

const isCustomContext = computed(() => (
  props.selectedPresetId === 'custom'
  || props.userPresets.some((p) => p.id === props.selectedPresetId)
))

const sections: Array<{ title: string; keys: Array<{ key: keyof Theme; label: string }> }> = [
  {
    title: 'Global',
    keys: [
      { key: 'appBg', label: 'App background' },
      { key: 'appText', label: 'Text primary' },
      { key: 'appTextMuted', label: 'Text muted' },
      { key: 'appTextSubtle', label: 'Text subtle' },
      { key: 'divider', label: 'Divider' },
      { key: 'shadowStrong', label: 'Shadow strong' },
      { key: 'overlayBg', label: 'Overlay' }
    ]
  },
  {
    title: 'Canvas',
    keys: [
      { key: 'canvasBorder', label: 'Canvas border' },
      { key: 'canvasGrid', label: 'Canvas grid color' },
      { key: 'canvasGridSize', label: 'Grid spacing' },
      { key: 'canvasGridDot', label: 'Dot size' },
      { key: 'canvasGridVisible', label: 'Grid visible' }
    ]
  },
  {
    title: 'Panels & Menus',
    keys: [
      { key: 'panelBg', label: 'Panel background' },
      { key: 'panelBorder', label: 'Panel border' },
      { key: 'panelText', label: 'Panel text' },
      { key: 'panelTextMuted', label: 'Panel text muted' },
      { key: 'menuBg', label: 'Menu background' },
      { key: 'menuBorder', label: 'Menu border' },
      { key: 'menuHover', label: 'Menu hover' }
    ]
  },
  {
    title: 'Inputs',
    keys: [
      { key: 'inputBg', label: 'Input background' },
      { key: 'inputBorder', label: 'Input border' },
      { key: 'inputText', label: 'Input text' },
      { key: 'inputPlaceholder', label: 'Input placeholder' }
    ]
  },
  {
    title: 'Buttons',
    keys: [
      { key: 'buttonBg', label: 'Button background' },
      { key: 'buttonBorder', label: 'Button border' },
      { key: 'buttonText', label: 'Button text' },
      { key: 'buttonHover', label: 'Button hover' },
      { key: 'buttonPrimaryBg', label: 'Primary background' },
      { key: 'buttonPrimaryText', label: 'Primary text' },
      { key: 'buttonPrimaryHover', label: 'Primary hover' },
      { key: 'buttonDangerBg', label: 'Danger background' },
      { key: 'buttonDangerText', label: 'Danger text' },
      { key: 'buttonDangerHover', label: 'Danger hover' },
      { key: 'buttonGhostBg', label: 'Ghost background' },
      { key: 'buttonGhostBorder', label: 'Ghost border' },
      { key: 'buttonGhostText', label: 'Ghost text' },
      { key: 'buttonGhostHover', label: 'Ghost hover' }
    ]
  },
  {
    title: 'Calendar',
    keys: [
      { key: 'dayBg', label: 'Day background' },
      { key: 'dayBorder', label: 'Day border' },
      { key: 'dayText', label: 'Day text' },
      { key: 'dayHoverBg', label: 'Day hover' },
      { key: 'daySelectedBg', label: 'Day selected bg' },
      { key: 'daySelectedBorder', label: 'Day selected border' },
      { key: 'daySelectedText', label: 'Day selected text' },
      { key: 'dayTodayRing', label: 'Today ring' }
    ]
  },
  {
    title: 'Clock',
    keys: [
      { key: 'clockSolid', label: 'Clock solid' },
      { key: 'clockOutline', label: 'Clock outline' },
      { key: 'clockTransparent', label: 'Clock transparent' },
      { key: 'clockSize', label: 'Clock size' },
      { key: 'clockOpacity', label: 'Clock opacity' }
    ]
  },
  {
    title: 'Containers',
    keys: [
      { key: 'containerHeaderBrightness', label: 'Topbar brightness' },
      { key: 'containerBorderBrightness', label: 'Border brightness' },
      { key: 'containerHeaderSaturation', label: 'Topbar saturation' },
      { key: 'containerBorderSaturation', label: 'Border saturation' }
    ]
  }
]
const colorKeys = new Set<keyof Theme>([
  'appBg',
  'appText',
  'appTextMuted',
  'appTextSubtle',
  'canvasGrid',
  'canvasBorder',
  'panelBg',
  'panelBorder',
  'panelText',
  'panelTextMuted',
  'menuBg',
  'menuBorder',
  'menuHover',
  'inputBg',
  'inputBorder',
  'inputText',
  'inputPlaceholder',
  'buttonBg',
  'buttonBorder',
  'buttonText',
  'buttonHover',
  'buttonPrimaryBg',
  'buttonPrimaryText',
  'buttonPrimaryHover',
  'buttonDangerBg',
  'buttonDangerText',
  'buttonDangerHover',
  'buttonGhostBg',
  'buttonGhostBorder',
  'buttonGhostText',
  'buttonGhostHover',
  'dayBg',
  'dayBorder',
  'dayText',
  'dayHoverBg',
  'daySelectedBg',
  'daySelectedBorder',
  'daySelectedText',
  'dayTodayRing',
  'divider',
  'shadowStrong',
  'overlayBg',
  'clockSolid',
  'clockOutline',
  'clockTransparent'
])
const sliderKeys = new Set<keyof Theme>([
  'clockSize',
  'clockOpacity',
  'containerHeaderBrightness',
  'containerBorderBrightness',
  'containerHeaderSaturation',
  'containerBorderSaturation'
])

function toHex(input: string) {
  const raw = input.trim()
  if (raw.startsWith('#')) {
    const hex = raw.slice(1)
    if (hex.length === 3) return `#${hex[0]}${hex[0]}${hex[1]}${hex[1]}${hex[2]}${hex[2]}`
    if (hex.length >= 6) return `#${hex.slice(0, 6)}`
  }
  const rgbMatch = raw.match(/rgba?\(([^)]+)\)/i)
  if (rgbMatch) {
    const parts = rgbMatch[1].split(',').map((p) => Number(p.trim()))
    if (parts.length >= 3 && parts.every((n) => Number.isFinite(n))) {
      const [r, g, b] = parts
      const toByte = (v: number) => Math.max(0, Math.min(255, Math.round(v)))
      return `#${toByte(r).toString(16).padStart(2, '0')}${toByte(g).toString(16).padStart(2, '0')}${toByte(b).toString(16).padStart(2, '0')}`
    }
  }
  return '#000000'
}

function onPickColor(key: keyof Theme, value: string) {
  props.theme[key] = value
  emit('changed')
}
</script>

<template>
  <div class="theme-panel theme-panel-shadow w-[440px] max-w-[96vw] rounded-2xl border p-4">
    <div class="mb-3 flex items-center justify-between">
      <div class="text-sm font-semibold">Design Studio</div>
      <div v-if="isCustomContext" class="flex items-center gap-2">
        <button class="theme-btn theme-btn-ghost rounded-md px-2 py-1 text-xs font-semibold" @click="emit('reset')">Reset</button>
        <button class="theme-btn theme-btn-primary rounded-md px-2 py-1 text-xs font-semibold" @click="emit('save', props.themeName)">Save</button>
      </div>
    </div>
    <div v-if="isCustomContext" class="mb-3 flex items-center gap-2">
      <label class="text-xs font-semibold theme-text-muted">Theme name</label>
      <input
        :value="props.themeName"
        type="text"
        class="theme-input flex-1 rounded-md border px-2 py-1 text-xs"
        spellcheck="false"
        @input="emit('updateName', ($event.target as HTMLInputElement).value)"
      >
    </div>
    <div v-if="isCustomContext" class="mb-3 flex items-center gap-2">
      <button class="theme-btn theme-hover rounded-md px-2 py-1 text-xs font-semibold" @click="emit('export')">Export</button>
      <button class="theme-btn theme-hover rounded-md px-2 py-1 text-xs font-semibold" @click="emit('import')">Import</button>
      <span class="text-[11px] theme-text-muted">Exports a portable theme JSON.</span>
    </div>
    <div class="mb-3 flex items-center gap-2">
      <label class="text-xs font-semibold theme-text-muted">Theme</label>
      <select
        class="theme-input flex-1 rounded-md border px-2 py-1 text-xs"
        :value="selectedPresetId"
        @change="emit('selectPreset', ($event.target as HTMLSelectElement).value)"
      >
        <optgroup label="Presets">
          <option v-for="preset in basePresets" :key="preset.id" :value="preset.id">
            {{ preset.name }}
          </option>
        </optgroup>
        <optgroup label="Custom Themes">
          <option value="custom-new">Create new…</option>
          <option v-if="selectedPresetId === 'custom'" value="custom">Unsaved</option>
          <option v-for="preset in userPresets" :key="preset.id" :value="preset.id">
            {{ preset.name }}
          </option>
        </optgroup>
      </select>
    </div>
    <div class="max-h-[70vh] space-y-4 overflow-auto pr-4 [scrollbar-gutter:stable]">
      <div v-for="section in sections" :key="section.title" class="space-y-2">
        <div class="text-xs font-bold uppercase tracking-wide theme-text-muted">{{ section.title }}</div>
        <div class="space-y-2">
          <label v-for="item in section.keys" :key="item.key" class="flex items-center gap-2 text-xs theme-text">
            <span class="w-40 shrink-0">{{ item.label }}</span>
            <template v-if="item.key === 'canvasGridVisible'">
              <input
                v-model="theme.canvasGridVisible"
                type="checkbox"
                class="h-4 w-4"
                @change="emit('changed')"
              >
              <span class="text-xs theme-text-muted">Show dots</span>
            </template>
            <template v-else-if="sliderKeys.has(item.key)">
              <input
                v-model.number="theme[item.key]"
                type="range"
                :min="item.key === 'clockSize' ? 32 : item.key === 'clockOpacity' ? 0.3 : 0.6"
                :max="item.key === 'clockSize' ? 120 : item.key === 'clockOpacity' ? 1 : 1.4"
                :step="item.key === 'clockSize' ? 1 : item.key === 'clockOpacity' ? 0.05 : 0.02"
                class="h-2 w-full cursor-pointer appearance-none rounded-full border bg-transparent theme-border"
                @input="emit('changed')"
              >
              <span class="w-12 text-right text-[11px] theme-text-muted">
                {{ item.key === 'clockSize' ? `${Math.round(Number(theme[item.key]))}px` : Number(theme[item.key]).toFixed(2) }}
              </span>
            </template>
            <template v-else>
              <input
                v-model="theme[item.key]"
                type="text"
                class="theme-input flex-1 rounded-md border px-2 py-1 text-xs"
                spellcheck="false"
                @input="emit('changed')"
              >
              <input
                v-if="colorKeys.has(item.key)"
                type="color"
                class="h-8 w-8 cursor-pointer rounded border theme-border"
                :value="toHex(theme[item.key] as string)"
                @input="onPickColor(item.key, ($event.target as HTMLInputElement).value)"
              >
            </template>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>
