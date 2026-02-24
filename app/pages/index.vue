<script setup lang="ts">
import { ArrowDownRight, BringToFront, Check, ChevronLeft, ChevronRight, Circle, CircleHelp, ClipboardPaste, Copy, EllipsisVertical, Minus, Plus, Redo2, SendToBack, SquarePen, Tag, Trash2, Undo2, X } from 'lucide-vue-next'

const CANVAS_SIZE = 3000
const ROLL_CONTAINER_W = 340
const ROLL_CONTAINER_H = 240
const ROLL_CONTAINER_GAP = 24
const STORAGE_KEY = 'tmap:state:v1'
const CLOCK_STYLE_KEY = 'tmap:clock-style:v1'
const LAYOUT_KEY = 'tmap:layout:v1'
const THEME_KEY = 'tmap:theme:v1'
const THEME_NAME_KEY = 'tmap:theme:name:v1'
const THEME_PRESET_KEY = 'tmap:theme:preset:v1'
const THEME_PRESETS_KEY = 'tmap:theme:presets:v1'
const THEME_USER_PRESETS_KEY = 'tmap:theme:user:presets:v1'
const CANVAS_PRESETS_KEY = 'tmap:canvas:presets:v1'
const ZOOM_KEY = 'tmap:zoom:v1'
const PWA_INSTALL_DISMISSED_KEY = 'tmap:pwa:install:dismissed:v1'
const SCHEMA_VERSION = 1
const CONTAINER_HEADER_H = 44
const TASK_ROW_H = 36
const MIN_W = 220
const MIN_H = 140

type Repeatability = 'daily' | 'weekly' | 'occasional'
type ContainerRole = 'todo' | 'done' | null
type ClockStyle = 'solid' | 'transparent' | 'outline'
interface BeforeInstallPromptEvent extends Event {
  prompt: () => Promise<void>
  userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>
}

interface PaletteColor { key: string; name: string; bar: string; border: string; chip: string; chipText: string }
interface TaskTemplate { id: string; name: string; color: string; repeatability: Repeatability }
interface TaskInstance { id: string; templateId: string; name: string; color: string; containerId: string }
interface ContainerItem { id: string; name: string; x: number; y: number; width: number; height: number; color: string; role: ContainerRole; z: number; taskIds: string[] }
interface DayCanvas { containers: ContainerItem[]; tasks: TaskInstance[]; nextZ: number }
interface AppState { version: number; templates: TaskTemplate[]; canvases: Record<string, DayCanvas>; meta: { lastRolloverDate: string | null } }
interface TaskDrag { source: 'inventory' | 'container'; templateId?: string; taskId?: string; x: number; y: number; hoverContainerId: string | null; hoverIndex: number; name: string; color: string }
interface LayoutPositions { calendar: { x: number; y: number }; panel: { x: number; y: number }; clock: { x: number; y: number }; minimap: { x: number; y: number } }
interface Theme {
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
interface ThemePreset { id: string; name: string; values: Theme }
interface CanvasPreset { id: string; name: string; canvas: DayCanvas }

const palette: PaletteColor[] = [
  { key: 'slate', name: 'Slate', bar: '#0f172a', border: '#334155', chip: '#334155', chipText: '#f8fafc' },
  { key: 'sky', name: 'Sky', bar: '#075985', border: '#0ea5e9', chip: '#0ea5e9', chipText: '#f0f9ff' },
  { key: 'emerald', name: 'Emerald', bar: '#065f46', border: '#10b981', chip: '#10b981', chipText: '#ecfdf5' },
  { key: 'amber', name: 'Amber', bar: '#92400e', border: '#f59e0b', chip: '#f59e0b', chipText: '#fffbeb' },
  { key: 'rose', name: 'Rose', bar: '#881337', border: '#f43f5e', chip: '#f43f5e', chipText: '#fff1f2' },
  { key: 'violet', name: 'Violet', bar: '#4c1d95', border: '#8b5cf6', chip: '#8b5cf6', chipText: '#f5f3ff' },
  { key: 'teal', name: 'Teal', bar: '#0f766e', border: '#14b8a6', chip: '#14b8a6', chipText: '#f0fdfa' },
  { key: 'stone', name: 'Stone', bar: '#44403c', border: '#78716c', chip: '#78716c', chipText: '#fafaf9' },
  { key: 'dusty-blue', name: 'Dusty Blue', bar: '#1f2f45', border: '#4a6f97', chip: '#5f84ac', chipText: '#e7effa' }
]

const repeatabilityLabels: Record<Repeatability, string> = { daily: 'Daily', weekly: 'Weekly', occasional: 'Occasional' }
const roleLabels = { todo: 'To-Do', done: 'Done' }

const viewportRef = ref<HTMLElement | null>(null)
const calendarBarRef = ref<HTMLElement | null>(null)
const state = ref<AppState>(createInitialState())
const selectedDate = ref(formatDateKey(new Date()))
const weekStart = ref(startOfWeek(new Date()))
const weekDirection = ref<'next' | 'prev'>('next')
const camera = reactive({ x: 0, y: 0 })
const zoom = ref(1)
const persistentCanvas = ref<1 | 2 | 3 | null>(null)
const calendarBarHeight = ref(0)
const hasCentered = ref(false)
const spacePressed = ref(false)
const now = ref(new Date())
const lastDateKey = ref(formatDateKey(now.value))
const clockStyle = ref<ClockStyle>('transparent')
const themePanelOpen = ref(false)
const selectedThemeId = ref('noctis')
const themeName = ref('Noctis')
let applyingTheme = false
const canInstallPwa = ref(false)
const installDismissed = ref(false)
let deferredInstallPrompt: BeforeInstallPromptEvent | null = null
const importThemeInputRef = ref<HTMLInputElement | null>(null)
const importDataInputRef = ref<HTMLInputElement | null>(null)
const defaultTheme: Theme = {
  appBg: '#020617',
  appText: '#e2e8f0',
  appTextMuted: '#94a3b8',
  appTextSubtle: '#64748b',
  canvasGrid: 'rgba(71, 85, 105, 0.3)',
  canvasGridSize: '22px',
  canvasGridDot: '1px',
  canvasGridVisible: true,
  canvasBorder: 'rgba(51, 65, 85, 0.8)',
  panelBg: 'rgba(15, 23, 42, 0.92)',
  panelBorder: 'rgba(51, 65, 85, 0.8)',
  panelText: '#e2e8f0',
  panelTextMuted: '#94a3b8',
  menuBg: '#0b1220',
  menuBorder: 'rgba(51, 65, 85, 0.9)',
  menuHover: 'rgba(30, 41, 59, 0.9)',
  inputBg: 'rgba(2, 6, 23, 0.6)',
  inputBorder: 'rgba(71, 85, 105, 0.6)',
  inputText: '#f1f5f9',
  inputPlaceholder: '#64748b',
  buttonBg: 'rgba(15, 23, 42, 0.9)',
  buttonBorder: 'rgba(71, 85, 105, 0.8)',
  buttonText: '#cbd5f5',
  buttonHover: 'rgba(30, 41, 59, 0.9)',
  buttonPrimaryBg: '#06b6d4',
  buttonPrimaryText: '#0b1220',
  buttonPrimaryHover: '#22d3ee',
  buttonDangerBg: 'rgba(244, 63, 94, 0.2)',
  buttonDangerText: '#fecdd3',
  buttonDangerHover: 'rgba(244, 63, 94, 0.3)',
  buttonGhostBg: 'rgba(2, 6, 23, 0.8)',
  buttonGhostBorder: 'rgba(71, 85, 105, 0.8)',
  buttonGhostText: '#cbd5f5',
  buttonGhostHover: 'rgba(30, 41, 59, 0.9)',
  dayBg: 'rgba(30, 41, 59, 0.8)',
  dayBorder: 'rgba(71, 85, 105, 0.7)',
  dayText: '#e2e8f0',
  dayHoverBg: 'rgba(51, 65, 85, 0.9)',
  daySelectedBg: '#22d3ee',
  daySelectedBorder: 'rgba(125, 211, 252, 0.7)',
  daySelectedText: '#0b1220',
  dayTodayRing: '#7dd3fc',
  divider: 'rgba(51, 65, 85, 0.8)',
  shadowStrong: 'rgba(0, 0, 0, 0.38)',
  overlayBg: 'transparent',
  clockSolid: 'rgba(255, 255, 255, 0.98)',
  clockOutline: 'rgba(255, 255, 255, 0.92)',
  clockTransparent: 'rgba(255, 255, 255, 0.26)',
  clockSize: 64,
  clockOpacity: 1,
  containerBorderBrightness: 1,
  containerHeaderBrightness: 1,
  containerBorderSaturation: 1,
  containerHeaderSaturation: 1
}
const theme = reactive<Theme>({ ...defaultTheme })
const baseThemePresets: ThemePreset[] = [
  {
    id: 'noctis',
    name: 'Noctis',
    values: {
      appBg: '#0b0b0f',
      appText: '#f2f2f7',
      appTextMuted: '#a1a1a6',
      appTextSubtle: '#8e8e93',
      canvasGrid: 'rgba(99, 99, 102, 0.35)',
      canvasGridSize: '22px',
      canvasGridDot: '1px',
      canvasGridVisible: true,
      canvasBorder: 'rgba(72, 72, 74, 0.85)',
      panelBg: 'rgba(28, 28, 30, 0.92)',
      panelBorder: 'rgba(72, 72, 74, 0.9)',
      panelText: '#f2f2f7',
      panelTextMuted: '#a1a1a6',
      menuBg: '#1c1c1e',
      menuBorder: 'rgba(72, 72, 74, 0.9)',
      menuHover: 'rgba(58, 58, 60, 0.85)',
      inputBg: '#1c1c1e',
      inputBorder: 'rgba(72, 72, 74, 0.9)',
      inputText: '#f2f2f7',
      inputPlaceholder: '#8e8e93',
      buttonBg: '#1c1c1e',
      buttonBorder: 'rgba(72, 72, 74, 0.9)',
      buttonText: '#f2f2f7',
      buttonHover: 'rgba(58, 58, 60, 0.85)',
      buttonPrimaryBg: '#0a84ff',
      buttonPrimaryText: '#0b0b0f',
      buttonPrimaryHover: '#3da0ff',
      buttonDangerBg: '#1c1c1e',
      buttonDangerText: '#ff453a',
      buttonDangerHover: 'rgba(255, 69, 58, 0.12)',
      buttonGhostBg: '#1c1c1e',
      buttonGhostBorder: 'rgba(72, 72, 74, 0.9)',
      buttonGhostText: '#f2f2f7',
      buttonGhostHover: 'rgba(58, 58, 60, 0.85)',
      dayBg: 'rgba(28, 28, 30, 0.92)',
      dayBorder: 'rgba(72, 72, 74, 0.9)',
      dayText: '#f2f2f7',
      dayHoverBg: 'rgba(58, 58, 60, 0.85)',
      daySelectedBg: '#086fd6',
      daySelectedBorder: '#086fd6',
      daySelectedText: '#ffffff',
      dayTodayRing: '#50a8cc',
      divider: 'rgba(72, 72, 74, 0.8)',
      shadowStrong: 'rgba(0, 0, 0, 0.6)',
      overlayBg: 'transparent',
      clockSolid: 'rgba(255, 255, 255, 0.98)',
      clockOutline: 'rgba(242, 242, 247, 0.85)',
      clockTransparent: 'rgba(242, 242, 247, 0.22)',
      clockSize: 64,
      clockOpacity: 1,
      containerBorderBrightness: 1,
      containerHeaderBrightness: 1,
      containerBorderSaturation: 1.25,
      containerHeaderSaturation: 1.25
    }
  },
  {
    id: 'lumen',
    name: 'Lumen',
    values: {
      appBg: '#f2f2f7',
      appText: '#1c1c1e',
      appTextMuted: '#6e6e73',
      appTextSubtle: '#8e8e93',
      canvasGrid: 'rgba(199, 199, 204, 0.5)',
      canvasGridSize: '22px',
      canvasGridDot: '1px',
      canvasGridVisible: true,
      canvasBorder: 'rgba(199, 199, 204, 0.8)',
      panelBg: 'rgba(255, 255, 255, 0.94)',
      panelBorder: 'rgba(199, 199, 204, 0.9)',
      panelText: '#1c1c1e',
      panelTextMuted: '#6e6e73',
      menuBg: '#ffffff',
      menuBorder: 'rgba(199, 199, 204, 0.9)',
      menuHover: 'rgba(235, 235, 240, 0.85)',
      inputBg: '#ffffff',
      inputBorder: 'rgba(199, 199, 204, 0.9)',
      inputText: '#1c1c1e',
      inputPlaceholder: '#8e8e93',
      buttonBg: '#ffffff',
      buttonBorder: 'rgba(199, 199, 204, 0.9)',
      buttonText: '#1c1c1e',
      buttonHover: 'rgba(235, 235, 240, 0.85)',
      buttonPrimaryBg: '#007aff',
      buttonPrimaryText: '#ffffff',
      buttonPrimaryHover: '#3395ff',
      buttonDangerBg: '#ffffff',
      buttonDangerText: '#ff3b30',
      buttonDangerHover: 'rgba(255, 59, 48, 0.16)',
      buttonGhostBg: '#ffffff',
      buttonGhostBorder: 'rgba(199, 199, 204, 0.9)',
      buttonGhostText: '#1c1c1e',
      buttonGhostHover: 'rgba(235, 235, 240, 0.85)',
      dayBg: 'rgba(250, 250, 252, 0.95)',
      dayBorder: 'rgba(199, 199, 204, 0.9)',
      dayText: '#1c1c1e',
      dayHoverBg: 'rgba(235, 235, 240, 0.85)',
      daySelectedBg: '#007aff',
      daySelectedBorder: '#0062cc',
      daySelectedText: '#ffffff',
      dayTodayRing: '#5ac8fa',
      divider: 'rgba(199, 199, 204, 0.7)',
      shadowStrong: 'rgba(15, 23, 42, 0.12)',
      overlayBg: 'transparent',
      clockSolid: 'rgba(28, 28, 30, 0.98)',
      clockOutline: 'rgba(28, 28, 30, 0.8)',
      clockTransparent: 'rgba(28, 28, 30, 0.22)',
      clockSize: 64,
      clockOpacity: 1,
      containerBorderBrightness: 1,
      containerHeaderBrightness: 1,
      containerBorderSaturation: 1.15,
      containerHeaderSaturation: 1.15
    }
  }
]
const presetOverrides = ref<Record<string, Theme>>({})
const userThemePresets = ref<ThemePreset[]>([])
const themePresets = computed(() => [
  ...baseThemePresets.map((preset) => ({
    ...preset,
    values: presetOverrides.value[preset.id] ?? preset.values
  })),
  ...userThemePresets.value
])

const canvasMenu = reactive({ visible: false, x: 0, y: 0, worldX: 0, worldY: 0 })
const canvasPresetMenu = reactive({ visible: false, x: 0, y: 0 })
const containerMenu = reactive({ visible: false, x: 0, y: 0, id: null as string | null })
const taskMenu = reactive({ visible: false, x: 0, y: 0, id: null as string | null })
const roleMenu = reactive({ visible: false, x: 0, y: 0, id: null as string | null })
const containerTaskMenu = reactive({ visible: false, x: 0, y: 0, id: null as string | null })

const panelTab = ref<'creator' | 'inventory'>('creator')
const creator = reactive({ name: '', color: 'cmap:0.6000', repeatability: 'daily' as Repeatability })
const creatorColorT = ref(0.6)
const containerColorT = ref(0.6)
const taskColorT = ref(0.6)
const inventoryDeleteMode = ref(false)

const draggingContainerId = ref<string | null>(null)
const resizingId = ref<string | null>(null)
const taskDrag = ref<TaskDrag | null>(null)
const dropChip = ref<{ x: number; y: number; name: string; color: string } | null>(null)
const lastPointer = reactive({ x: 0, y: 0 })
const editingContainerId = ref<string | null>(null)
const editingContainerName = ref('')
const editingTaskId = ref<string | null>(null)
const editingTaskName = ref('')
const infoPanelOpen = ref(false)
const quickMenuOpen = ref(false)
const minimapVisible = ref(false)
const panelOpen = ref(false)
let minimapTimer: ReturnType<typeof setTimeout> | null = null
const devLayoutMode = ref(false)
const layoutPositions = reactive<LayoutPositions>({ calendar: { x: 0, y: 0 }, panel: { x: 0, y: 0 }, clock: { x: 16, y: 16 }, minimap: { x: 16, y: 120 } })
let layoutSnapshot: LayoutPositions | null = null
const inputModal = reactive({
  visible: false,
  title: '',
  label: '',
  value: '',
  confirmText: 'Save'
})
const confirmModal = reactive({
  visible: false,
  title: '',
  message: '',
  confirmText: 'Confirm'
})
const rolloverModal = reactive({
  visible: false,
  title: 'Rollover Complete',
  message: ''
})
let inputModalSubmit: ((value: string) => void) | null = null
let confirmModalSubmit: (() => void) | null = null
const containerClipboard = ref<ContainerItem & { tasks: TaskInstance[] } | null>(null)
const canvasPresets = ref<CanvasPreset[]>([])
const presetDeleteMode = ref(false)
const presetDeleteSelection = ref<Set<string>>(new Set())

const pan = reactive({ active: false, sx: 0, sy: 0, cx: 0, cy: 0 })
let containerDrag: { id: string; sx: number; sy: number; ox: number; oy: number } | null = null
let resizeDrag: { id: string; sx: number; sy: number; ow: number; oh: number } | null = null
let layoutDrag: { target: 'calendar' | 'panel' | 'clock' | 'minimap'; sx: number; sy: number; ox: number; oy: number } | null = null
let saveTimer: ReturnType<typeof setTimeout> | null = null
let clockTimer: ReturnType<typeof setInterval> | null = null
let tauriInvoke: ((command: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null
const hasLoadedPersistence = ref(false)
const undoStack = ref<string[]>([])
const redoStack = ref<string[]>([])
let historyTimer: ReturnType<typeof setTimeout> | null = null
let lastHistorySnapshot = ''
let isRestoringHistory = false

const activeCanvasKey = computed(() => (persistentCanvas.value ? `slot:${persistentCanvas.value}` : selectedDate.value))
const currentCanvas = computed(() => ensureCanvas(activeCanvasKey.value))
const containersSorted = computed(() => [...currentCanvas.value.containers].sort((a, b) => a.z - b.z))
const weekDays = computed(() => Array.from({ length: 7 }, (_, i) => addDays(weekStart.value, i)))
const weekTransitionName = computed(() => `week-slide-${weekDirection.value}`)
const groupedTemplates = computed(() => ({
  daily: state.value.templates.filter((t) => t.repeatability === 'daily'),
  weekly: state.value.templates.filter((t) => t.repeatability === 'weekly'),
  occasional: state.value.templates.filter((t) => t.repeatability === 'occasional')
}))
const time24 = computed(() => now.value.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false }))
const rolloverCount = computed(() => getManualRolloverCount())
const minimapSize = 180
const minimapScale = computed(() => minimapSize / CANVAS_SIZE)
const minimapViewport = computed(() => {
  const rect = viewportRef.value?.getBoundingClientRect()
  const width = rect?.width ?? 0
  const height = rect?.height ?? 0
  const worldX = -camera.x / zoom.value
  const worldY = -camera.y / zoom.value
  return {
    x: worldX * minimapScale.value,
    y: worldY * minimapScale.value,
    width: (width / zoom.value) * minimapScale.value,
    height: (height / zoom.value) * minimapScale.value
  }
})
const mapSliderBg = computed(() => ({
  background: 'linear-gradient(90deg, #ff3b30 0%, #ff9500 16%, #ffcc00 32%, #34c759 48%, #00c7be 64%, #007aff 80%, #af52de 92%, #ff2d55 100%)'
}))
const themeVars = computed(() => ({
  '--app-bg': theme.appBg,
  '--app-text': theme.appText,
  '--app-text-muted': theme.appTextMuted,
  '--app-text-subtle': theme.appTextSubtle,
  '--canvas-grid': theme.canvasGrid,
  '--canvas-grid-color': theme.canvasGridVisible ? theme.canvasGrid : 'transparent',
  '--canvas-grid-size': theme.canvasGridSize,
  '--canvas-grid-dot': theme.canvasGridDot,
  '--canvas-border': theme.canvasBorder,
  '--panel-bg': theme.panelBg,
  '--panel-border': theme.panelBorder,
  '--panel-text': theme.panelText,
  '--panel-text-muted': theme.panelTextMuted,
  '--menu-bg': theme.menuBg,
  '--menu-border': theme.menuBorder,
  '--menu-hover': theme.menuHover,
  '--input-bg': theme.inputBg,
  '--input-border': theme.inputBorder,
  '--input-text': theme.inputText,
  '--input-placeholder': theme.inputPlaceholder,
  '--button-bg': theme.buttonBg,
  '--button-border': theme.buttonBorder,
  '--button-text': theme.buttonText,
  '--button-hover': theme.buttonHover,
  '--button-primary-bg': theme.buttonPrimaryBg,
  '--button-primary-text': theme.buttonPrimaryText,
  '--button-primary-hover': theme.buttonPrimaryHover,
  '--button-danger-bg': theme.buttonDangerBg,
  '--button-danger-text': theme.buttonDangerText,
  '--button-danger-hover': theme.buttonDangerHover,
  '--button-ghost-bg': theme.buttonGhostBg,
  '--button-ghost-border': theme.buttonGhostBorder,
  '--button-ghost-text': theme.buttonGhostText,
  '--button-ghost-hover': theme.buttonGhostHover,
  '--day-bg': theme.dayBg,
  '--day-border': theme.dayBorder,
  '--day-text': theme.dayText,
  '--day-hover-bg': theme.dayHoverBg,
  '--day-selected-bg': theme.daySelectedBg,
  '--day-selected-border': theme.daySelectedBorder,
  '--day-selected-text': theme.daySelectedText,
  '--day-today-ring': theme.dayTodayRing,
  '--divider': theme.divider,
  '--shadow-strong': theme.shadowStrong,
  '--overlay-bg': theme.overlayBg,
  '--clock-solid': theme.clockSolid,
  '--clock-outline': theme.clockOutline,
  '--clock-transparent': theme.clockTransparent
}))

watch(state, () => {
  if (!hasLoadedPersistence.value) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    void saveAppState(JSON.stringify(state.value))
  }, 120)
}, { deep: true })

watch(state, () => {
  if (!hasLoadedPersistence.value || isRestoringHistory) return
  if (historyTimer) clearTimeout(historyTimer)
  historyTimer = setTimeout(() => {
    const snapshot = JSON.stringify(state.value)
    if (snapshot === lastHistorySnapshot) return
    undoStack.value.push(snapshot)
    if (undoStack.value.length > 80) undoStack.value.shift()
    redoStack.value = []
    lastHistorySnapshot = snapshot
  }, 250)
}, { deep: true })

  onMounted(async () => {
    await loadState()
    finalizeAfterLoad()
    loadCanvasPresets()
    const storedZoom = typeof window === 'undefined' ? null : localStorage.getItem(ZOOM_KEY)
    if (storedZoom) {
      const parsed = Number.parseFloat(storedZoom)
      if (Number.isFinite(parsed)) zoom.value = clamp(parsed, 0.7, 1.6)
    }
  const savedClockStyle = await loadClockStyle()
  if (savedClockStyle === 'solid' || savedClockStyle === 'transparent' || savedClockStyle === 'outline') clockStyle.value = savedClockStyle
  loadLayoutPositions()
  loadTheme()
  installDismissed.value = localStorage.getItem(PWA_INSTALL_DISMISSED_KEY) === '1'
  window.addEventListener('beforeinstallprompt', onBeforeInstallPrompt as EventListener)
  window.addEventListener('appinstalled', onAppInstalled)
  clockTimer = setInterval(() => { now.value = new Date() }, 1000)
    window.addEventListener('resize', centerCamera)
    window.addEventListener('resize', updateCalendarBarHeight)
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
    window.addEventListener('keydown', onKeyDown)
    window.addEventListener('keyup', onKeyUp)
    viewportRef.value?.addEventListener('wheel', onViewportWheel, { passive: false })
    document.addEventListener('pointerdown', closeMenus)
    nextTick(updateCalendarBarHeight)
  })

watch(now, (value) => {
  const currentKey = formatDateKey(value)
  if (currentKey === lastDateKey.value) return
  const previousKey = lastDateKey.value
  lastDateKey.value = currentKey
  const rolledCount = rolloverToToday()
  ensureCanvas(currentKey)
  if (selectedDate.value === previousKey) {
    selectedDate.value = currentKey
    weekStart.value = startOfWeek(value)
  }
  if (rolledCount > 0) {
    rolloverModal.message = `${rolledCount} task${rolledCount === 1 ? '' : 's'} rolled over to ${currentKey}.`
    rolloverModal.visible = true
  }
})

watch(clockStyle, (value) => {
  if (!hasLoadedPersistence.value) return
  void saveClockStyle(value)
})
watch(theme, () => {
  if (!hasLoadedPersistence.value) return
  saveTheme()
}, { deep: true })

onBeforeUnmount(() => {
  if (clockTimer) clearInterval(clockTimer)
  window.removeEventListener('beforeinstallprompt', onBeforeInstallPrompt as EventListener)
  window.removeEventListener('appinstalled', onAppInstalled)
    window.removeEventListener('resize', centerCamera)
    window.removeEventListener('resize', updateCalendarBarHeight)
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('keydown', onKeyDown)
    window.removeEventListener('keyup', onKeyUp)
    viewportRef.value?.removeEventListener('wheel', onViewportWheel)
    document.removeEventListener('pointerdown', closeMenus)
  })

function createInitialState(): AppState { return { version: SCHEMA_VERSION, templates: [], canvases: {}, meta: { lastRolloverDate: null } } }
function ensureCanvas(key: string): DayCanvas { if (!state.value.canvases[key]) state.value.canvases[key] = { containers: [], tasks: [], nextZ: 1 }; return state.value.canvases[key] }
async function getTauriInvoke() {
  if (tauriInvoke) return tauriInvoke
  if (typeof window === 'undefined') return null
  const marker = window as Window & { __TAURI__?: unknown; __TAURI_INTERNALS__?: unknown }
  if (!marker.__TAURI__ && !marker.__TAURI_INTERNALS__) return null
  try {
    const mod = await import('@tauri-apps/api/core')
    tauriInvoke = mod.invoke as (command: string, args?: Record<string, unknown>) => Promise<unknown>
    return tauriInvoke
  } catch {
    return null
  }
}
async function loadAppState() {
  return localStorage.getItem(STORAGE_KEY)
}
async function saveAppState(value: string) {
  localStorage.setItem(STORAGE_KEY, value)
}
async function loadClockStyle() {
  const invoke = await getTauriInvoke()
  if (invoke) {
    try { return await invoke('load_clock_style') as string | null } catch { return localStorage.getItem(CLOCK_STYLE_KEY) }
  }
  return localStorage.getItem(CLOCK_STYLE_KEY)
}
async function saveClockStyle(value: string) {
  const invoke = await getTauriInvoke()
  if (invoke) {
    try { await invoke('save_clock_style', { style: value }); return } catch {}
  }
  localStorage.setItem(CLOCK_STYLE_KEY, value)
}
  function loadLayoutPositions() {
    if (typeof window === 'undefined') return
    const raw = localStorage.getItem(LAYOUT_KEY)
    if (raw) {
      try {
        const parsed = JSON.parse(raw) as LayoutPositions
        if (parsed?.calendar && parsed?.panel) {
          layoutPositions.calendar.x = parsed.calendar.x
          layoutPositions.calendar.y = parsed.calendar.y
          layoutPositions.panel.x = parsed.panel.x
          layoutPositions.panel.y = parsed.panel.y
          if (parsed.clock) {
            layoutPositions.clock.x = parsed.clock.x
            layoutPositions.clock.y = parsed.clock.y
          } else {
            layoutPositions.clock.x = 16
            layoutPositions.clock.y = 16
          }
          if (parsed.minimap) {
            layoutPositions.minimap.x = parsed.minimap.x
            layoutPositions.minimap.y = parsed.minimap.y
          } else {
            layoutPositions.minimap.x = 16
            layoutPositions.minimap.y = Math.max(16, window.innerHeight - 16 - minimapSize)
          }
          return
        }
      } catch {}
    }
    layoutPositions.calendar.x = Math.round(window.innerWidth / 2)
    layoutPositions.calendar.y = 16
    layoutPositions.panel.x = Math.max(16, window.innerWidth - 16 - 380)
    layoutPositions.panel.y = 96
    layoutPositions.clock.x = 16
    layoutPositions.clock.y = 16
    layoutPositions.minimap.x = 16
    layoutPositions.minimap.y = Math.max(16, window.innerHeight - 16 - minimapSize)
  }
function saveLayoutPositions() {
  if (typeof window === 'undefined') return
  localStorage.setItem(LAYOUT_KEY, JSON.stringify(layoutPositions))
}
  function loadTheme() {
    if (typeof window === 'undefined') return
    const presetsRaw = localStorage.getItem(THEME_PRESETS_KEY)
    if (presetsRaw) {
      try {
        presetOverrides.value = JSON.parse(presetsRaw) as Record<string, Theme>
      } catch {}
    }
    const userPresetsRaw = localStorage.getItem(THEME_USER_PRESETS_KEY)
    if (userPresetsRaw) {
      try {
        const parsed = JSON.parse(userPresetsRaw) as ThemePreset[]
        if (Array.isArray(parsed)) userThemePresets.value = parsed
      } catch {}
    }
    const presetRaw = localStorage.getItem(THEME_PRESET_KEY)
    if (presetRaw) selectedThemeId.value = presetRaw
    const savedName = localStorage.getItem(THEME_NAME_KEY)
    if (savedName) themeName.value = savedName
    if (selectedThemeId.value && selectedThemeId.value !== 'custom') {
      const preset = themePresets.value.find((p) => p.id === selectedThemeId.value)
      if (preset) {
        applyTheme(preset.values, true)
        themeName.value = preset.name
        return
      }
    }
    const defaultPreset = themePresets.value.find((p) => p.id === 'noctis')
    if (defaultPreset) {
      selectedThemeId.value = 'noctis'
      themeName.value = defaultPreset.name
      applyTheme(defaultPreset.values, true)
      saveThemePreset()
    }
    if (selectedThemeId.value !== 'custom') return
    const raw = localStorage.getItem(THEME_KEY)
    if (!raw) return
    try {
      const parsed = JSON.parse(raw) as Partial<Theme>
      Object.assign(theme, { ...defaultTheme, ...parsed })
    } catch {}
  }
  function saveTheme() {
    if (typeof window === 'undefined') return
    localStorage.setItem(THEME_KEY, JSON.stringify(theme))
    localStorage.setItem(THEME_NAME_KEY, themeName.value)
  }
  function saveThemePreset() {
    if (typeof window === 'undefined') return
    localStorage.setItem(THEME_PRESET_KEY, selectedThemeId.value)
  }
  function savePresetOverrides() {
    if (typeof window === 'undefined') return
    localStorage.setItem(THEME_PRESETS_KEY, JSON.stringify(presetOverrides.value))
  }
  function saveUserPresets() {
    if (typeof window === 'undefined') return
    localStorage.setItem(THEME_USER_PRESETS_KEY, JSON.stringify(userThemePresets.value))
  }
  function makeCustomThemeName() {
    const base = 'Custom'
    const existing = new Set(userThemePresets.value.map((p) => p.name.toLowerCase()))
    if (!existing.has(base.toLowerCase())) return base
    let i = 2
    while (existing.has(`${base} ${i}`.toLowerCase())) i += 1
    return `${base} ${i}`
  }
function applyTheme(values: Theme, silent = false) {
  applyingTheme = true
  Object.assign(theme, values)
  queueMicrotask(() => { applyingTheme = false })
  if (!silent) saveTheme()
}
  function selectTheme(id: string) {
    if (id === 'custom-new') {
      selectedThemeId.value = 'custom'
      themeName.value = makeCustomThemeName()
      saveThemePreset()
      saveTheme()
      return
    }
    if (id === 'custom') {
      selectedThemeId.value = 'custom'
      saveThemePreset()
      return
    }
    const preset = themePresets.value.find((p) => p.id === id)
    if (!preset) return
    selectedThemeId.value = id
    saveThemePreset()
    themeName.value = preset.name
    applyTheme(preset.values, true)
    saveTheme()
  }
  function makePresetId(name: string) {
    const base = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '') || 'theme'
    let id = base
    let i = 2
    const existing = new Set(themePresets.value.map((p) => p.id))
    while (existing.has(id)) {
      id = `${base}-${i}`
      i += 1
    }
    return id
  }
  function saveCurrentTheme(name: string) {
    const trimmed = name.trim()
    if (!trimmed) return
    themeName.value = trimmed
    const existing = userThemePresets.value.find((p) => p.name.toLowerCase() === trimmed.toLowerCase())
    if (existing) {
      existing.values = JSON.parse(JSON.stringify(theme)) as Theme
      selectedThemeId.value = existing.id
      saveThemePreset()
      saveUserPresets()
      saveTheme()
      return
    }
    const id = makePresetId(trimmed)
    const preset: ThemePreset = { id, name: trimmed, values: JSON.parse(JSON.stringify(theme)) as Theme }
    userThemePresets.value = [...userThemePresets.value, preset]
    selectedThemeId.value = id
    saveThemePreset()
    saveUserPresets()
    saveTheme()
  }
function markThemeCustom() {
  selectedThemeId.value = 'custom'
  saveThemePreset()
}
  function resetTheme() {
    Object.assign(theme, defaultTheme)
    markThemeCustom()
  }
  function updateThemeName(name: string) {
    themeName.value = name
    saveTheme()
  }
  function exportThemeConfig() {
    const safeName = themeName.value.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '') || 'theme'
    const payload = {
      version: 1,
      selectedThemeId: selectedThemeId.value,
      themeName: themeName.value,
      theme: theme,
      presetOverrides: presetOverrides.value
    }
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${safeName}.json`
    document.body.appendChild(a)
    a.click()
    a.remove()
    URL.revokeObjectURL(url)
  }
function openThemeImport() { importThemeInputRef.value?.click() }
function onThemeImportChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    try {
      const raw = String(reader.result || '')
        const parsed = JSON.parse(raw) as { selectedThemeId?: string; themeName?: string; theme?: Theme; presetOverrides?: Record<string, Theme> }
        if (parsed.presetOverrides) {
          presetOverrides.value = parsed.presetOverrides
          savePresetOverrides()
        }
        if (parsed.themeName) {
          themeName.value = parsed.themeName
          localStorage.setItem(THEME_NAME_KEY, parsed.themeName)
        }
        if (parsed.theme) {
          Object.assign(theme, parsed.theme)
        }
      if (parsed.selectedThemeId) {
        selectTheme(parsed.selectedThemeId)
      } else {
        saveTheme()
      }
    } catch {}
    input.value = ''
  }
  reader.readAsText(file)
}
function onBeforeInstallPrompt(e: Event) {
  const event = e as BeforeInstallPromptEvent
  event.preventDefault()
  deferredInstallPrompt = event
  if (!installDismissed.value) canInstallPwa.value = true
}
async function promptInstallPwa() {
  if (!deferredInstallPrompt) return
  const prompt = deferredInstallPrompt
  deferredInstallPrompt = null
  canInstallPwa.value = false
  prompt.prompt()
  try { await prompt.userChoice } catch {}
  localStorage.setItem(PWA_INSTALL_DISMISSED_KEY, '1')
  installDismissed.value = true
}
function dismissInstallPwa() {
  canInstallPwa.value = false
  localStorage.setItem(PWA_INSTALL_DISMISSED_KEY, '1')
  installDismissed.value = true
}
function resetInstallPrompt() {
  localStorage.removeItem(PWA_INSTALL_DISMISSED_KEY)
  installDismissed.value = false
  canInstallPwa.value = true
}
function onAppInstalled() {
  canInstallPwa.value = false
  localStorage.setItem(PWA_INSTALL_DISMISSED_KEY, '1')
  installDismissed.value = true
}
  async function loadState() {
    const raw = await loadAppState()
    if (!raw) return
    try {
      const parsed = JSON.parse(raw) as AppState
      if (parsed.version === SCHEMA_VERSION && parsed.canvases && parsed.templates) state.value = parsed
    } catch {
      state.value = createInitialState()
    }
  }
  function finalizeAfterLoad() {
    const initialRolled = rolloverToToday()
    selectedDate.value = formatDateKey(new Date())
    weekStart.value = startOfWeek(new Date())
    ensureCanvas(selectedDate.value)
    centerCamera()
    lastHistorySnapshot = JSON.stringify(state.value)
    undoStack.value = [lastHistorySnapshot]
    redoStack.value = []
    if (initialRolled > 0) {
      rolloverModal.message = `${initialRolled} task${initialRolled === 1 ? '' : 's'} rolled over to ${selectedDate.value}.`
      rolloverModal.visible = true
    }
    hasLoadedPersistence.value = true
  }
  function exportAppData() {
    const payload = JSON.stringify(state.value, null, 2)
    const blob = new Blob([payload], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    const stamp = new Date().toISOString().slice(0, 10)
    a.href = url
    a.download = `tmap-backup-${stamp}.json`
    document.body.appendChild(a)
    a.click()
    a.remove()
    URL.revokeObjectURL(url)
  }
  function requestImportAppData() {
    openConfirmModal('Import Data', 'This will replace your current data. Continue?', 'Import', () => {
      importDataInputRef.value?.click()
    })
  }
  function onImportDataSelected(event: Event) {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]
    input.value = ''
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => {
      try {
        const text = String(reader.result ?? '')
        const parsed = JSON.parse(text) as AppState
        if (!parsed || parsed.version !== SCHEMA_VERSION || !parsed.canvases || !parsed.templates) {
          openConfirmModal('Import Failed', 'That file does not look like a valid tmap backup.', 'OK', () => {})
          return
        }
        state.value = parsed
        lastHistorySnapshot = JSON.stringify(state.value)
        undoStack.value = [lastHistorySnapshot]
        redoStack.value = []
        ensureCanvas(selectedDate.value)
        void saveAppState(JSON.stringify(state.value))
      } catch {
        openConfirmModal('Import Failed', 'That file could not be read.', 'OK', () => {})
      }
    }
    reader.readAsText(file)
  }
  function centerCamera() { if (!viewportRef.value || hasCentered.value) return; const rect = viewportRef.value.getBoundingClientRect(); camera.x = Math.round(rect.width / 2 - (CANVAS_SIZE / 2) * zoom.value); camera.y = Math.round(rect.height / 2 - (CANVAS_SIZE / 2) * zoom.value); hasCentered.value = true }
  function getViewportWorldCenter() {
    if (!viewportRef.value) return { x: CANVAS_SIZE / 2, y: CANVAS_SIZE / 2 }
    const rect = viewportRef.value.getBoundingClientRect()
    return { x: (-camera.x + rect.width / 2) / zoom.value, y: (-camera.y + rect.height / 2) / zoom.value }
  }
  function focusCameraOnContainer(c: ContainerItem) {
    if (!viewportRef.value) return
    const rect = viewportRef.value.getBoundingClientRect()
    const cx = c.x + c.width / 2
    const cy = c.y + c.height / 2
    camera.x = Math.round(rect.width / 2 - cx * zoom.value)
    camera.y = Math.round(rect.height / 2 - cy * zoom.value)
  }
  function focusCameraOnPoint(point: { x: number; y: number }) {
    if (!viewportRef.value) return
    const rect = viewportRef.value.getBoundingClientRect()
    camera.x = Math.round(rect.width / 2 - point.x * zoom.value)
    camera.y = Math.round(rect.height / 2 - point.y * zoom.value)
  }
  function showMinimap() {
    minimapVisible.value = true
    if (minimapTimer) clearTimeout(minimapTimer)
    minimapTimer = setTimeout(() => {
      minimapVisible.value = false
    }, 3000)
  }
  function onKeyUp(e: KeyboardEvent) { if (e.code === 'Space') spacePressed.value = false }
  function onKeyDown(e: KeyboardEvent) {
    if (e.code === 'Space') spacePressed.value = true
    if (e.target && (e.target as HTMLElement).closest('input,textarea,select,[contenteditable="true"]')) return
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
      e.preventDefault()
      if (e.shiftKey) redoHistory()
      else undoHistory()
      return
    }
    if ((e.ctrlKey || e.metaKey) && (e.key.toLowerCase() === 'y')) {
      e.preventDefault()
      redoHistory()
    }
  }
  function closeMenus() { canvasMenu.visible = false; canvasPresetMenu.visible = false; containerMenu.visible = false; taskMenu.visible = false; roleMenu.visible = false; containerTaskMenu.visible = false }
  function toWorld(clientX: number, clientY: number) { const r = viewportRef.value?.getBoundingClientRect(); if (!r) return { x: 0, y: 0 }; return { x: (clientX - r.left - camera.x) / zoom.value, y: (clientY - r.top - camera.y) / zoom.value } }
function getContainer(id: string | null) { return id ? currentCanvas.value.containers.find((c) => c.id === id) ?? null : null }
function getTask(id: string | null) { return id ? currentCanvas.value.tasks.find((t) => t.id === id) ?? null : null }
function uid(prefix: string) { return `${prefix}-${crypto.randomUUID()}` }
function clamp(v: number, min: number, max: number) { return Math.max(min, Math.min(max, v)) }
function formatDateKey(d: Date) { return `${d.getFullYear()}-${`${d.getMonth() + 1}`.padStart(2, '0')}-${`${d.getDate()}`.padStart(2, '0')}` }
function parseDateKey(key: string) { const [y, m, d] = key.split('-').map(Number); return new Date(y, m - 1, d) }
function startOfWeek(d: Date) { const c = new Date(d); c.setHours(0, 0, 0, 0); const diff = (c.getDay() + 6) % 7; c.setDate(c.getDate() - diff); return c }
function addDays(d: Date, n: number) { const c = new Date(d); c.setDate(c.getDate() + n); return c }
function formatDayDDMMYY(d: Date) {
  const dd = `${d.getDate()}`.padStart(2, '0')
  const mm = `${d.getMonth() + 1}`.padStart(2, '0')
  const yy = `${d.getFullYear() % 100}`.padStart(2, '0')
  return `${dd}.${mm}.${yy}`
}
function cycleClockStyle() {
  if (clockStyle.value === 'solid') { clockStyle.value = 'transparent'; return }
  if (clockStyle.value === 'transparent') { clockStyle.value = 'outline'; return }
  clockStyle.value = 'solid'
}

  function onViewportPointerDown(e: PointerEvent) { if (e.button === 1 || (spacePressed.value && e.button === 0)) { e.preventDefault(); closeMenus(); pan.active = true; pan.sx = e.clientX; pan.sy = e.clientY; pan.cx = camera.x; pan.cy = camera.y } }
  function onViewportWheel(e: WheelEvent) {
    if (!e.ctrlKey) return
    e.preventDefault()
    const r = viewportRef.value?.getBoundingClientRect()
    if (!r) return
    const prev = zoom.value
    const next = clamp(prev - e.deltaY * 0.0012, 0.7, 1.6)
    if (next === prev) return
    const worldX = (e.clientX - r.left - camera.x) / prev
    const worldY = (e.clientY - r.top - camera.y) / prev
    camera.x = e.clientX - r.left - worldX * next
    camera.y = e.clientY - r.top - worldY * next
    zoom.value = Number(next.toFixed(3))
    localStorage.setItem(ZOOM_KEY, String(zoom.value))
    showMinimap()
  }
  function resetZoomToOne() {
    if (!viewportRef.value) return
    const r = viewportRef.value.getBoundingClientRect()
    const prev = zoom.value
    const worldX = (r.width / 2 - camera.x) / prev
    const worldY = (r.height / 2 - camera.y) / prev
    zoom.value = 1
    camera.x = r.width / 2 - worldX * zoom.value
    camera.y = r.height / 2 - worldY * zoom.value
    localStorage.setItem(ZOOM_KEY, String(zoom.value))
    showMinimap()
  }
  function updateCalendarBarHeight() {
    if (!calendarBarRef.value) return
    calendarBarHeight.value = calendarBarRef.value.offsetHeight
  }
  function onViewportContextMenu(e: MouseEvent) {
    const target = e.target as HTMLElement | null
    if (target?.closest('[data-container]') || target?.closest('[data-floating-ui]')) return
    const w = toWorld(e.clientX, e.clientY)
    closeMenus(); canvasMenu.visible = true; canvasMenu.x = e.clientX; canvasMenu.y = e.clientY; canvasMenu.worldX = w.x; canvasMenu.worldY = w.y
  }
  function openCanvasPresetMenu() {
    const menuWidth = 260
    const menuHeight = 320
    canvasPresetMenu.visible = true
    canvasPresetMenu.x = clamp(canvasMenu.x + 188, 8, window.innerWidth - menuWidth - 8)
    canvasPresetMenu.y = clamp(canvasMenu.y, 8, window.innerHeight - menuHeight - 8)
  }
function createContainerAt(x: number, y: number) {
  const c = currentCanvas.value
  const container: ContainerItem = {
    id: uid('container'),
    name: `Container ${c.containers.length + 1}`,
    x: clamp(x - 160, 0, CANVAS_SIZE - 260),
    y: clamp(y - 52, 0, CANVAS_SIZE - 170),
    width: 320,
    height: 230,
    color: 'oklch:0.62:0.18:220',
    role: null,
    z: c.nextZ++,
    taskIds: []
  }
  c.containers.push(container)
  startContainerRename(container.id)
  canvasMenu.visible = false
}
  function openContainerMenu(e: MouseEvent, id: string) {
  e.stopPropagation()
  e.preventDefault()
  closeMenus()
  const anchor = e.currentTarget as HTMLElement | null
  const rect = anchor?.getBoundingClientRect()
  const menuWidth = 224
  const menuHeight = 260
  containerMenu.visible = true
  containerMenu.id = id
  containerMenu.x = clamp(rect?.left ?? e.clientX, 8, window.innerWidth - menuWidth - 8)
  containerMenu.y = clamp(rect?.top ?? e.clientY, 8, window.innerHeight - menuHeight - 8)
    const container = getContainer(id)
    containerColorT.value = colorTFromColor(container?.color ?? 'cmap:0.6000')
  }
function openTaskMenu(e: MouseEvent, id: string) {
  e.stopPropagation()
  e.preventDefault()
  closeMenus()
  taskMenu.visible = true
  taskMenu.id = id
  taskMenu.x = e.clientX
  taskMenu.y = e.clientY
  const task = getTask(id)
  taskColorT.value = colorTFromColor(task?.color ?? 'cmap:0.6000')
}
function openRoleMenu(e: MouseEvent, id: string) { e.stopPropagation(); e.preventDefault(); closeMenus(); roleMenu.visible = true; roleMenu.id = id; roleMenu.x = e.clientX; roleMenu.y = e.clientY }
  function openContainerTaskMenu(e: MouseEvent, id: string) {
  const target = e.target as HTMLElement | null
  if (target?.closest('[data-task-chip]')) return
  e.stopPropagation()
  e.preventDefault()
  closeMenus()
  containerTaskMenu.visible = true
  containerTaskMenu.id = id
  containerTaskMenu.x = e.clientX
  containerTaskMenu.y = e.clientY
}

function startContainerDrag(e: PointerEvent, container: ContainerItem) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (target?.closest('input,button,textarea,select')) return
  e.stopPropagation(); closeMenus(); bringToFront(container.id)
  containerDrag = { id: container.id, sx: e.clientX, sy: e.clientY, ox: container.x, oy: container.y }
  draggingContainerId.value = container.id
}
function startResize(e: PointerEvent, container: ContainerItem) {
  if (e.button !== 0) return
  e.stopPropagation(); closeMenus()
  resizeDrag = { id: container.id, sx: e.clientX, sy: e.clientY, ow: container.width, oh: container.height }
  resizingId.value = container.id
}
function getTopContainerAt(x: number, y: number) {
  const sorted = [...currentCanvas.value.containers].sort((a, b) => b.z - a.z)
  return sorted.find((c) => x >= c.x && x <= c.x + c.width && y >= c.y && y <= c.y + c.height) ?? null
}
  function getContainerBody(containerId: string) {
    if (typeof document === 'undefined') return null
    const containerEl = document.querySelector(`[data-container-id="${containerId}"]`) as HTMLElement | null
    return (containerEl?.querySelector('[data-container-body]') as HTMLElement | null) ?? null
  }
  function getDropIndex(containerId: string, clientY: number) {
    const c = getContainer(containerId); if (!c) return 0
    const ids = c.taskIds.filter((id) => id !== taskDrag.value?.taskId)
    const body = getContainerBody(containerId)
    if (body) {
      const chips = Array.from(body.querySelectorAll('[data-task-chip]')) as HTMLElement[]
      const filtered = chips.filter((el) => el.dataset.taskId !== taskDrag.value?.taskId)
      for (let i = 0; i < filtered.length; i += 1) {
        const rect = filtered[i].getBoundingClientRect()
        const mid = rect.top + rect.height / 2
        if (clientY < mid) return i
      }
      return filtered.length
    }
    const localY = toWorld(0, clientY).y - c.y - CONTAINER_HEADER_H - 16
    for (let i = 0; i < ids.length; i += 1) { if (localY < i * TASK_ROW_H + TASK_ROW_H / 2) return i }
    return ids.length
  }
function startTaskDragFromContainer(e: PointerEvent, task: TaskInstance) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (target?.closest('input,button,textarea,select')) return
  e.stopPropagation(); closeMenus()
    taskDrag.value = { source: 'container', taskId: task.id, x: e.clientX, y: e.clientY, hoverContainerId: task.containerId, hoverIndex: getDropIndex(task.containerId, e.clientY), name: task.name, color: task.color }
}
function startTaskDragFromInventory(e: PointerEvent, t: TaskTemplate) {
  if (e.button !== 0) return
  e.stopPropagation(); closeMenus()
  taskDrag.value = { source: 'inventory', templateId: t.id, x: e.clientX, y: e.clientY, hoverContainerId: null, hoverIndex: 0, name: t.name, color: t.color }
}
  function startLayoutDrag(e: PointerEvent, target: 'calendar' | 'panel' | 'clock' | 'minimap') {
  if (!devLayoutMode.value || e.button !== 0) return
  e.preventDefault()
  e.stopPropagation()
  layoutDrag = { target, sx: e.clientX, sy: e.clientY, ox: layoutPositions[target].x, oy: layoutPositions[target].y }
}
function onMove(e: PointerEvent) {
  lastPointer.x = e.clientX
  lastPointer.y = e.clientY
    if (layoutDrag) {
      const nextX = layoutDrag.ox + (e.clientX - layoutDrag.sx)
      const nextY = layoutDrag.oy + (e.clientY - layoutDrag.sy)
      layoutPositions[layoutDrag.target].x = clamp(nextX, 16, window.innerWidth - 16)
      layoutPositions[layoutDrag.target].y = clamp(nextY, 16, window.innerHeight - 16)
      return
    }
    if (pan.active) {
      camera.x = pan.cx + (e.clientX - pan.sx)
      camera.y = pan.cy + (e.clientY - pan.sy)
      showMinimap()
      return
    }

  if (containerDrag) {
    const c = getContainer(containerDrag.id); if (!c) return
      c.x = clamp(containerDrag.ox + (e.clientX - containerDrag.sx) / zoom.value, 0, CANVAS_SIZE - c.width)
      c.y = clamp(containerDrag.oy + (e.clientY - containerDrag.sy) / zoom.value, 0, CANVAS_SIZE - c.height)
      return
    }

  if (resizeDrag) {
    const c = getContainer(resizeDrag.id); if (!c) return
      c.width = clamp(resizeDrag.ow + (e.clientX - resizeDrag.sx) / zoom.value, MIN_W, CANVAS_SIZE - c.x)
      c.height = clamp(resizeDrag.oh + (e.clientY - resizeDrag.sy) / zoom.value, MIN_H, CANVAS_SIZE - c.y)
      return
    }

  if (!taskDrag.value) return
  taskDrag.value.x = e.clientX; taskDrag.value.y = e.clientY
  const world = toWorld(e.clientX, e.clientY)
  const hovered = getTopContainerAt(world.x, world.y)
  if (!hovered || world.y < hovered.y + CONTAINER_HEADER_H || world.y > hovered.y + hovered.height - 8) { taskDrag.value.hoverContainerId = null; taskDrag.value.hoverIndex = 0; return }
  taskDrag.value.hoverContainerId = hovered.id
    taskDrag.value.hoverIndex = getDropIndex(hovered.id, e.clientY)
}

function onUp(e?: PointerEvent) {
  if (e) {
    lastPointer.x = e.clientX
    lastPointer.y = e.clientY
  }
  if (layoutDrag) { layoutDrag = null }
  pan.active = false
  if (containerDrag) { containerDrag = null; draggingContainerId.value = null }
  if (resizeDrag) { resizeDrag = null; resizingId.value = null }
  if (!taskDrag.value) return

  const drag = taskDrag.value
  const target = drag.hoverContainerId ? getContainer(drag.hoverContainerId) : null
  if (!target) {
    const px = e?.clientX ?? lastPointer.x ?? drag.x
    const py = e?.clientY ?? lastPointer.y ?? drag.y
    dropChip.value = { x: px, y: py, name: drag.name, color: drag.color }
    setTimeout(() => { dropChip.value = null }, 520)
    if (drag.source === 'container' && drag.taskId) removeTaskInstance(drag.taskId)
    taskDrag.value = null
    return
  }

  if (drag.source === 'inventory' && drag.templateId) {
    const tpl = state.value.templates.find((t) => t.id === drag.templateId)
    if (tpl) {
      const newTask: TaskInstance = { id: uid('task'), templateId: tpl.id, name: tpl.name, color: tpl.color, containerId: target.id }
      currentCanvas.value.tasks.push(newTask)
      target.taskIds.splice(clamp(drag.hoverIndex, 0, target.taskIds.length), 0, newTask.id)
    }
  }

  if (drag.source === 'container' && drag.taskId) {
    const task = getTask(drag.taskId)
    if (task) {
      const origin = getContainer(task.containerId)
      if (origin) origin.taskIds = origin.taskIds.filter((id) => id !== task.id)
      task.containerId = target.id
      target.taskIds.splice(clamp(drag.hoverIndex, 0, target.taskIds.length), 0, task.id)
    }
  }

  taskDrag.value = null
}
function toggleDevLayoutMode() {
  if (devLayoutMode.value) {
    saveLayoutPositions()
    devLayoutMode.value = false
    layoutSnapshot = null
    return
  }
  layoutSnapshot = JSON.parse(JSON.stringify(layoutPositions)) as LayoutPositions
  devLayoutMode.value = true
}
  function abortDevLayoutMode() {
    if (!devLayoutMode.value) return
    if (layoutSnapshot) {
      layoutPositions.calendar.x = layoutSnapshot.calendar.x
      layoutPositions.calendar.y = layoutSnapshot.calendar.y
      layoutPositions.panel.x = layoutSnapshot.panel.x
      layoutPositions.panel.y = layoutSnapshot.panel.y
      layoutPositions.clock.x = layoutSnapshot.clock.x
      layoutPositions.clock.y = layoutSnapshot.clock.y
      layoutPositions.minimap.x = layoutSnapshot.minimap.x
      layoutPositions.minimap.y = layoutSnapshot.minimap.y
    }
    devLayoutMode.value = false
    layoutSnapshot = null
  }

function getRenderableTaskIds(c: ContainerItem) {
  const ids = c.taskIds.filter((id) => id !== taskDrag.value?.taskId)
  if (taskDrag.value?.hoverContainerId === c.id) { const i = clamp(taskDrag.value.hoverIndex, 0, ids.length); const arr = [...ids]; arr.splice(i, 0, '__placeholder__'); return arr }
  return ids
}

function saveTemplate() {
  const name = creator.name.trim(); if (!name) return
  state.value.templates.push({ id: uid('template'), name, color: creator.color, repeatability: creator.repeatability })
  creator.name = ''; panelTab.value = 'inventory'
}
function colorFromT(t: number) {
  return `cmap:${clamp(t, 0, 1).toFixed(4)}`
}
function colorTFromColor(color: string) {
  if (color.startsWith('cmap:')) {
    const parsed = Number(color.slice(5))
    return Number.isFinite(parsed) ? clamp(parsed, 0, 1) : 0.6
  }

  const legacyHues: Record<string, number> = {
    slate: 220,
    sky: 200,
    emerald: 150,
    amber: 40,
    rose: 345,
    violet: 265,
    teal: 175,
    stone: 30
  }

  if (color.startsWith('oklch:')) {
    const parts = color.split(':')
    const hue = Number(parts[3])
    return Number.isFinite(hue) ? clamp(hue / 360, 0, 1) : 0.6
  }
  if (color.startsWith('hsb:')) {
    const parts = color.split(':')
    const hue = Number(parts[1])
    return Number.isFinite(hue) ? clamp(hue / 360, 0, 1) : 0.6
  }
  if (color.startsWith('hue:')) {
    const parsed = Number(color.slice(4))
    return Number.isFinite(parsed) ? clamp(parsed / 360, 0, 1) : 0.6
  }
  return clamp((legacyHues[color] ?? 220) / 360, 0, 1)
}
function selectCreatorSliderColor() {
  creator.color = colorFromT(creatorColorT.value)
}
function onCreatorSliderInput(event: Event) {
  const value = Number((event.target as HTMLInputElement).value)
  if (!Number.isFinite(value)) return
  creatorColorT.value = value
  creator.color = colorFromT(value)
}
function bringToFront(id: string) { const c = getContainer(id); if (c) c.z = currentCanvas.value.nextZ++ }
function sendToBack(id: string) { const c = getContainer(id); if (!c) return; const min = Math.min(...currentCanvas.value.containers.map((i) => i.z)); c.z = min - 1 }
function openInputModal(title: string, label: string, initialValue: string, confirmText: string, onSubmit: (value: string) => void) {
  closeMenus()
  inputModal.visible = true
  inputModal.title = title
  inputModal.label = label
  inputModal.value = initialValue
  inputModal.confirmText = confirmText
  inputModalSubmit = onSubmit
}
function closeInputModal() {
  inputModal.visible = false
  inputModalSubmit = null
}
function submitInputModal() {
  const value = inputModal.value.trim()
  if (!value) return
  inputModalSubmit?.(value)
  closeInputModal()
}
  function openConfirmModal(title: string, message: string, confirmText: string, onSubmit: () => void) {
    closeMenus()
    confirmModal.visible = true
    confirmModal.title = title
    confirmModal.message = message
    confirmModal.confirmText = confirmText
    confirmModalSubmit = onSubmit
  }
function closeConfirmModal() {
  confirmModal.visible = false
  confirmModalSubmit = null
}
function submitConfirmModal() {
  confirmModalSubmit?.()
  closeConfirmModal()
}
function restoreFromSnapshot(snapshot: string) {
  try {
    const parsed = JSON.parse(snapshot) as AppState
    isRestoringHistory = true
    state.value = parsed
    lastHistorySnapshot = snapshot
  } finally {
    isRestoringHistory = false
  }
}
function undoHistory() {
  if (undoStack.value.length <= 1) return
  const current = undoStack.value.pop()
  if (!current) return
  redoStack.value.push(current)
  const prev = undoStack.value[undoStack.value.length - 1]
  if (prev) restoreFromSnapshot(prev)
}
function redoHistory() {
  if (redoStack.value.length === 0) return
  const next = redoStack.value.pop()
  if (!next) return
  undoStack.value.push(next)
  restoreFromSnapshot(next)
}
  function loadCanvasPresets() {
    if (typeof window === 'undefined') return
    const raw = localStorage.getItem(CANVAS_PRESETS_KEY)
    if (!raw) return
    try {
      const parsed = JSON.parse(raw) as CanvasPreset[]
      if (Array.isArray(parsed)) canvasPresets.value = parsed
    } catch {}
  }
  function saveCanvasPresets() {
    if (typeof window === 'undefined') return
    localStorage.setItem(CANVAS_PRESETS_KEY, JSON.stringify(canvasPresets.value))
  }
  function saveCurrentCanvasPreset() {
    openInputModal('Save Layout Preset', 'Name', `Preset ${canvasPresets.value.length + 1}`, 'Save', (value) => {
      const current = currentCanvas.value
      const snapshot = JSON.parse(JSON.stringify(current)) as DayCanvas
      const preset: CanvasPreset = { id: uid('preset'), name: value, canvas: snapshot }
      canvasPresets.value = [...canvasPresets.value, preset]
      saveCanvasPresets()
    })
  }
  function togglePresetDeleteMode() {
    presetDeleteMode.value = !presetDeleteMode.value
    presetDeleteSelection.value = new Set()
  }
  function togglePresetSelection(id: string) {
    const next = new Set(presetDeleteSelection.value)
    if (next.has(id)) next.delete(id)
    else next.add(id)
    presetDeleteSelection.value = next
  }
  function removeSelectedPresets() {
    if (presetDeleteSelection.value.size === 0) return
    canvasPresets.value = canvasPresets.value.filter((p) => !presetDeleteSelection.value.has(p.id))
    presetDeleteSelection.value = new Set()
    saveCanvasPresets()
  }
  function applyCanvasPreset(preset: CanvasPreset) {
    const target = currentCanvas.value
    const snapshot = JSON.parse(JSON.stringify(preset.canvas)) as DayCanvas
    const idMap = new Map<string, string>()
    for (const container of snapshot.containers) {
      const newId = uid('container')
      idMap.set(container.id, newId)
      const clone: ContainerItem = {
        ...container,
        id: newId,
        z: target.nextZ++
      }
      target.containers.push(clone)
    }
    for (const task of snapshot.tasks) {
      const newContainerId = idMap.get(task.containerId)
      if (!newContainerId) continue
      const newTaskId = uid('task')
      const clone: TaskInstance = { ...task, id: newTaskId, containerId: newContainerId }
      target.tasks.push(clone)
      const container = target.containers.find((c) => c.id === newContainerId)
      if (container) container.taskIds.push(newTaskId)
    }
    canvasPresetMenu.visible = false
    canvasMenu.visible = false
  }
  function clearCurrentCanvas() {
    const key = selectedDate.value
    const confirmClear = () => {
      state.value.canvases[key] = { containers: [], tasks: [], nextZ: 1 }
    }
    canvasMenu.visible = false
    openConfirmModal('Clear Canvas', `Clear all containers and tasks for ${key}?`, 'Clear', confirmClear)
  }
function renameContainer(id: string) {
  startContainerRename(id)
}
function startContainerRename(id: string) {
  const c = getContainer(id)
  if (!c) return
  editingContainerId.value = id
  editingContainerName.value = c.name
  nextTick(() => {
    const input = document.getElementById(`container-name-input-${id}`) as HTMLInputElement | null
    input?.focus()
    input?.select()
  })
}
function commitContainerRename(id: string) {
  const c = getContainer(id)
  if (!c) return
  const nextName = editingContainerName.value.trim()
  if (nextName) c.name = nextName
  editingContainerId.value = null
}
function cancelContainerRename() {
  editingContainerId.value = null
}
  function setContainerColor(id: string, color: string) { const c = getContainer(id); if (c) c.color = color }
  function selectContainerSliderColor() {
    const id = containerMenu.id
    if (!id) return
    const next = colorFromT(containerColorT.value)
    setContainerColor(id, next)
  }
function selectTaskSliderColor() {
  const id = taskMenu.id
  if (!id) return
  setTaskColor(id, colorFromT(taskColorT.value))
}
function setContainerRole(id: string, role: ContainerRole) { const c = getContainer(id); if (c) c.role = role; roleMenu.visible = false }
  function copyContainer(id: string) {
    const c = getContainer(id)
    if (!c) return
    const tasks = c.taskIds
      .map((taskId) => getTask(taskId))
      .filter((t): t is TaskInstance => Boolean(t))
      .map((t) => ({ ...t }))
    containerClipboard.value = { ...c, tasks }
    containerMenu.visible = false
  }
  function pasteContainer(target?: { x: number; y: number }) {
    if (!containerClipboard.value) return
    const clip = containerClipboard.value
    const c = currentCanvas.value
    const offset = 24
    const newId = uid('container')
    const baseX = target ? target.x - clip.width / 2 : clip.x + offset
    const baseY = target ? target.y - CONTAINER_HEADER_H / 2 : clip.y + offset
    const newContainer: ContainerItem = {
      id: newId,
      name: `${clip.name} Copy`,
      x: clamp(baseX, 0, CANVAS_SIZE - clip.width),
      y: clamp(baseY, 0, CANVAS_SIZE - clip.height),
      width: clip.width,
      height: clip.height,
      color: clip.color,
      role: clip.role,
      z: c.nextZ++,
      taskIds: []
    }
    c.containers.push(newContainer)
    for (const t of clip.tasks) {
      const clone: TaskInstance = {
        id: uid('task'),
        templateId: t.templateId,
        name: t.name,
        color: t.color,
        containerId: newId
      }
      c.tasks.push(clone)
      newContainer.taskIds.push(clone.id)
    }
    containerMenu.visible = false
  }
  function removeContainer(id: string) {
  const c = getContainer(id)
  if (!c) return
  const removeNow = () => {
    currentCanvas.value.tasks = currentCanvas.value.tasks.filter((t) => t.containerId !== id)
    currentCanvas.value.containers = currentCanvas.value.containers.filter((x) => x.id !== id)
    containerMenu.visible = false
  }
  if (c.taskIds.length > 0) {
    openConfirmModal('Remove Container', 'Are you sure?', 'Remove', removeNow)
    return
  }
  removeNow()
}
function renameTask(id: string) {
  const t = getTask(id)
  if (!t) return
  editingTaskId.value = id
  editingTaskName.value = t.name
  nextTick(() => {
    const input = document.getElementById(`task-name-input-${id}`) as HTMLInputElement | null
    input?.focus()
    input?.select()
  })
}
function commitTaskRename(id: string) {
  const t = getTask(id)
  if (!t) return
  const nextName = editingTaskName.value.trim()
  if (nextName) t.name = nextName
  editingTaskId.value = null
}
function cancelTaskRename() {
  editingTaskId.value = null
}
function setTaskColor(id: string, color: string) { const t = getTask(id); if (t) t.color = color }
function removeTaskInstance(id: string) {
  const t = getTask(id); if (!t) return
  const c = getContainer(t.containerId); if (c) c.taskIds = c.taskIds.filter((x) => x !== id)
  currentCanvas.value.tasks = currentCanvas.value.tasks.filter((x) => x.id !== id)
  if (editingTaskId.value === id) editingTaskId.value = null
}
function createTaskInContainer(containerId: string) {
  const c = getContainer(containerId)
  if (!c) return
  const task: TaskInstance = { id: uid('task'), templateId: 'manual', name: 'New Task', color: 'dusty-blue', containerId: c.id }
  currentCanvas.value.tasks.push(task)
  c.taskIds.push(task.id)
  containerTaskMenu.visible = false
}
function removeTemplate(id: string) {
  state.value.templates = state.value.templates.filter((item) => item.id !== id)
}
function hexToRgb(hex: string) {
  const clean = hex.replace('#', '')
  const full = clean.length === 3 ? clean.split('').map((c) => `${c}${c}`).join('') : clean
  const value = Number.parseInt(full, 16)
  return {
    r: (value >> 16) & 255,
    g: (value >> 8) & 255,
    b: value & 255
  }
}
  function mixRgb(a: { r: number, g: number, b: number }, b: { r: number, g: number, b: number }, t: number) {
    return {
      r: Math.round(a.r + (b.r - a.r) * t),
      g: Math.round(a.g + (b.g - a.g) * t),
      b: Math.round(a.b + (b.b - a.b) * t)
    }
  }
  function rgbToHsl(rgb: { r: number, g: number, b: number }) {
    const r = rgb.r / 255
    const g = rgb.g / 255
    const b = rgb.b / 255
    const max = Math.max(r, g, b)
    const min = Math.min(r, g, b)
    const l = (max + min) / 2
    if (max === min) return { h: 0, s: 0, l }
    const d = max - min
    const s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
    let h = 0
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break
      case g: h = (b - r) / d + 2; break
      case b: h = (r - g) / d + 4; break
      default: h = 0; break
    }
    h /= 6
    return { h, s, l }
  }
  function hslToRgb(hsl: { h: number, s: number, l: number }) {
    const { h, s, l } = hsl
    if (s === 0) {
      const v = Math.round(l * 255)
      return { r: v, g: v, b: v }
    }
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s
    const p = 2 * l - q
    const hue2rgb = (t: number) => {
      let tt = t
      if (tt < 0) tt += 1
      if (tt > 1) tt -= 1
      if (tt < 1 / 6) return p + (q - p) * 6 * tt
      if (tt < 1 / 2) return q
      if (tt < 2 / 3) return p + (q - p) * (2 / 3 - tt) * 6
      return p
    }
    return {
      r: Math.round(hue2rgb(h + 1 / 3) * 255),
      g: Math.round(hue2rgb(h) * 255),
      b: Math.round(hue2rgb(h - 1 / 3) * 255)
    }
  }
function rgbCss(rgb: { r: number, g: number, b: number }) {
  return `rgb(${rgb.r} ${rgb.g} ${rgb.b})`
}
function isLightColor(color: string) {
  const rgb = parseRgb(color)
  if (!rgb) return false
  const luma = (rgb.r * 299 + rgb.g * 587 + rgb.b * 114) / 1000
  return luma > 140
}
  function parseRgb(input: string) {
    const raw = input.trim()
    if (raw.startsWith('#')) {
      const hex = raw.slice(1)
      const full = hex.length === 3 ? hex.split('').map((c) => `${c}${c}`).join('') : hex
      if (full.length >= 6) {
        const value = Number.parseInt(full.slice(0, 6), 16)
        return {
          r: (value >> 16) & 255,
          g: (value >> 8) & 255,
          b: value & 255
        }
      }
    }
    const rgbMatch = raw.match(/rgba?\(([^)]+)\)/i)
    if (rgbMatch) {
      const cleaned = rgbMatch[1].replace(/\s*\/\s*/g, ' ')
      const parts = cleaned.split(/[\s,]+/).filter(Boolean)
      if (parts.length >= 3) {
        const toByte = (value: string) => {
          if (value.endsWith('%')) return Math.round(clamp(Number(value.slice(0, -1)) / 100, 0, 1) * 255)
          return Math.round(clamp(Number(value), 0, 255))
        }
        const r = toByte(parts[0])
        const g = toByte(parts[1])
        const b = toByte(parts[2])
        if ([r, g, b].every((n) => Number.isFinite(n))) return { r, g, b }
      }
    }
    return null
  }
  function applySaturation(color: string, saturation?: number) {
    const value = clamp(saturation ?? 1, 0.6, 1.4)
    if (value === 1) return color
    const rgb = parseRgb(color)
    if (!rgb) return color
    const hsl = rgbToHsl(rgb)
    hsl.s = clamp(hsl.s * value, 0, 1)
    return rgbCss(hslToRgb(hsl))
  }
  function applyBrightness(color: string, brightness?: number) {
    const value = clamp(brightness ?? 1, 0.6, 1.4)
    if (value === 1) return color
    if (value < 1) {
      const pct = Math.round(value * 100)
      return `color-mix(in srgb, ${color} ${pct}%, black)`
    }
    const pct = Math.round((2 - value) * 100)
    return `color-mix(in srgb, ${color} ${pct}%, white)`
  }
function sampleRainbow(t: number) {
  const stops = ['#ff3b30', '#ff9500', '#ffcc00', '#34c759', '#00c7be', '#007aff', '#af52de', '#ff2d55']
  const clamped = clamp(t, 0, 1)
  const scaled = clamped * (stops.length - 1)
  const index = Math.floor(scaled)
  const local = scaled - index
  const start = hexToRgb(stops[index] ?? stops[0])
  const end = hexToRgb(stops[Math.min(stops.length - 1, index + 1)] ?? stops[stops.length - 1])
  return mixRgb(start, end, local)
}

function resolveColor(colorKey: string) {
  if (colorKey.startsWith('cmap:') || colorKey.startsWith('hue:') || colorKey.startsWith('hsb:') || colorKey.startsWith('oklch:')) {
    const t = colorTFromColor(colorKey)
    const chipRgb = sampleRainbow(t)
    const barRgb = mixRgb(chipRgb, { r: 0, g: 0, b: 0 }, 0.62)
    const borderRgb = mixRgb(barRgb, { r: 255, g: 255, b: 255 }, 0.05)
    const luma = (chipRgb.r * 299 + chipRgb.g * 587 + chipRgb.b * 114) / 1000
    return {
      key: colorKey,
      name: 'Rainbow',
      bar: rgbCss(barRgb),
      border: rgbCss(borderRgb),
      chip: rgbCss(chipRgb),
      chipText: luma > 150 ? '#0f172a' : '#f8fafc'
    }
  }

  return palette.find((c) => c.key === colorKey) ?? palette[0]
}
function getTaskStyle(colorKey: string) {
  const c = resolveColor(colorKey)
  const lightUi = isLightColor(theme.appBg)
  const bg = `color-mix(in srgb, ${c.chip} 10%, transparent)`
  return {
    backgroundColor: bg,
    color: lightUi ? '#111827' : '#f8fafc',
    borderColor: c.chip,
    borderWidth: '1px',
    borderLeftWidth: '4px'
  }
}
  function getContainerHeaderAccentStyle(c: ContainerItem) {
    const p = resolveColor(c.color)
    const borderBase = applySaturation(p.border, theme.containerBorderSaturation)
    const border = applyBrightness(borderBase, theme.containerBorderBrightness)
    return { borderColor: border, borderWidth: '2px' }
  }
  function getContainerStyle(c: ContainerItem) {
    const p = resolveColor(c.color)
    const borderBase = applySaturation(p.border, theme.containerBorderSaturation)
    const border = applyBrightness(borderBase, theme.containerBorderBrightness)
    return {
      borderColor: border,
      borderWidth: '2px',
      zIndex: c.z,
      left: `${c.x}px`,
      top: `${c.y}px`,
      width: `${c.width}px`,
      height: `${c.height}px`,
      boxShadow: resizingId.value === c.id ? `0 0 0 2px ${border}, 0 10px 24px rgba(2, 6, 23, 0.42)` : undefined,
      filter: resizingId.value === c.id ? 'brightness(1.06)' : undefined
    }
  }
  function getHeaderStyle(c: ContainerItem) {
    const p = resolveColor(c.color)
    const headerBase = applySaturation(p.bar, theme.containerHeaderSaturation)
    const borderBase = applySaturation(p.border, theme.containerBorderSaturation)
    const headerBg = applyBrightness(headerBase, theme.containerHeaderBrightness)
    const headerBorder = applyBrightness(borderBase, theme.containerBorderBrightness)
    return { backgroundColor: headerBg, borderBottomColor: headerBorder, color: '#f8fafc' }
  }

function goToPreviousWeek() { weekDirection.value = 'prev'; weekStart.value = addDays(weekStart.value, -7) }
function goToNextWeek() { weekDirection.value = 'next'; weekStart.value = addDays(weekStart.value, 7) }
function selectDay(day: Date) { persistentCanvas.value = null; selectedDate.value = formatDateKey(day) }
function isToday(day: Date) { return formatDateKey(day) === formatDateKey(new Date()) }
function isSelectedDay(day: Date) { if (persistentCanvas.value) return false; return formatDateKey(day) === selectedDate.value }
function isPastDay(day: Date) { return formatDateKey(day) < formatDateKey(new Date()) }
  function rolloverFromSourceToTarget(sourceKey: string, targetKey: string, shouldFocus: boolean) {
    const source = state.value.canvases[sourceKey]
    if (!source) return 0
    const todoContainers = source.containers.filter((c) => c.role === 'todo' && c.taskIds.length > 0)
    if (todoContainers.length === 0) return 0
    const target = ensureCanvas(targetKey)
    const center = getViewportWorldCenter()
    const totalWidth = todoContainers.length * ROLL_CONTAINER_W + (todoContainers.length - 1) * ROLL_CONTAINER_GAP
    const startX = center.x - totalWidth / 2
    const startY = center.y - ROLL_CONTAINER_H / 2
    const created: ContainerItem[] = []
    let rolledCount = 0

    for (let i = 0; i < todoContainers.length; i += 1) {
      const src = todoContainers[i]
      const baseName = src.name.replace(/[, ]*roll[- ]?over.*$/i, '').trim() || src.name.trim()
      const rollName = `${baseName}, roll-over`
      const x = clamp(startX + i * (ROLL_CONTAINER_W + ROLL_CONTAINER_GAP), 0, CANVAS_SIZE - ROLL_CONTAINER_W)
      const y = clamp(startY, 0, CANVAS_SIZE - ROLL_CONTAINER_H)
      const roll: ContainerItem = {
        id: uid('container'),
        name: rollName,
        x,
        y,
        width: ROLL_CONTAINER_W,
        height: ROLL_CONTAINER_H,
        color: src.color,
        role: 'todo',
        z: target.nextZ++,
        taskIds: []
      }
      target.containers.push(roll)
      const map = new Map(source.tasks.map((t) => [t.id, t]))
      const copied = src.taskIds.map((id) => map.get(id)).filter((x): x is TaskInstance => Boolean(x))
      for (const t of copied) {
        const clone: TaskInstance = { id: uid('task'), templateId: t.templateId, name: t.name, color: t.color, containerId: roll.id }
        target.tasks.push(clone)
        roll.taskIds.push(clone.id)
      }
      rolledCount += copied.length
      created.push(roll)
    }

    if (shouldFocus && created.length > 0) {
      const minX = Math.min(...created.map((c) => c.x))
      const maxX = Math.max(...created.map((c) => c.x + c.width))
      const minY = Math.min(...created.map((c) => c.y))
      const maxY = Math.max(...created.map((c) => c.y + c.height))
      focusCameraOnPoint({ x: (minX + maxX) / 2, y: (minY + maxY) / 2 })
    }

    return rolledCount
  }
  function rolloverToToday() {
    let rolledCount = 0
    const today = new Date(); today.setHours(0, 0, 0, 0)
    const start = state.value.meta.lastRolloverDate ? parseDateKey(state.value.meta.lastRolloverDate) : addDays(today, -1)
    let cursor = addDays(start, 1)
    while (cursor <= today) {
      const sourceKey = formatDateKey(addDays(cursor, -1)); const targetKey = formatDateKey(cursor)
      rolledCount += rolloverFromSourceToTarget(sourceKey, targetKey, false)
      state.value.meta.lastRolloverDate = formatDateKey(cursor)
      cursor = addDays(cursor, 1)
    }
    return rolledCount
  }
function getTodoTasksForCanvas(key: string) {
  const canvas = state.value.canvases[key]
  if (!canvas) return [] as TaskInstance[]
  const todoIds = canvas.containers.filter((c) => c.role === 'todo').flatMap((c) => c.taskIds)
  if (todoIds.length === 0) return [] as TaskInstance[]
  const map = new Map(canvas.tasks.map((t) => [t.id, t]))
  return todoIds.map((id) => map.get(id)).filter((x): x is TaskInstance => Boolean(x))
}
function getManualRolloverCount() {
  if (persistentCanvas.value) return 0
  return getTodoTasksForCanvas(selectedDate.value).length
}
function triggerManualRollover() {
  if (rolloverCount.value <= 0) return
  const sourceKey = selectedDate.value
  const targetKey = formatDateKey(addDays(parseDateKey(sourceKey), 1))
  const rolledCount = rolloverFromSourceToTarget(sourceKey, targetKey, true)
  if (rolledCount === 0) return
  selectedDate.value = targetKey
  weekStart.value = startOfWeek(parseDateKey(targetKey))
  rolloverModal.message = `${rolledCount} task${rolledCount === 1 ? '' : 's'} rolled over to ${targetKey}.`
  rolloverModal.visible = true
}
</script>

<template>
  <div class="relative h-screen w-screen overflow-hidden select-none theme-app" :style="themeVars" @contextmenu.prevent>
    <div ref="viewportRef" class="absolute inset-0" @pointerdown="onViewportPointerDown" @contextmenu.prevent="onViewportContextMenu">
        <div class="absolute" :style="{ width: `${CANVAS_SIZE}px`, height: `${CANVAS_SIZE}px`, transform: `translate(${camera.x}px, ${camera.y}px) scale(${zoom})`, transformOrigin: 'top left' }">
        <div class="absolute inset-0 rounded-3xl border theme-canvas shadow-inner" />

        <div v-for="container in containersSorted" :key="container.id" data-container :data-container-id="container.id" class="absolute flex flex-col overflow-hidden rounded-xl border theme-panel backdrop-blur-md theme-panel-shadow" :class="{ 'scale-[1.015]': draggingContainerId === container.id }" :style="getContainerStyle(container)">
          <div class="flex h-11 cursor-grab items-center justify-between border-b pl-4 pr-3 text-[15px] font-semibold" :style="getHeaderStyle(container)" @pointerdown="startContainerDrag($event, container)">
            <div class="flex items-center gap-2">
                <input
                  v-if="editingContainerId === container.id"
                  :id="`container-name-input-${container.id}`"
                  v-model="editingContainerName"
                  type="text"
                      class="w-36 max-w-[46vw] rounded-md border px-2 py-1 text-[14px] outline-none focus:ring-0 theme-input"
                  @pointerdown.stop
                  @keydown.enter.prevent="commitContainerRename(container.id)"
                  @keydown.esc.prevent="cancelContainerRename"
                  @blur="commitContainerRename(container.id)"
                >
              <span v-else class="truncate">{{ container.name }}</span>
              <button
                v-if="!container.role"
                class="rounded-md border p-1 theme-hover"
                :style="getContainerHeaderAccentStyle(container)"
                @pointerdown.stop
                @click.stop="openRoleMenu($event, container.id)"
                @contextmenu.prevent
              ><Plus :size="12" /></button>
              <button
                v-else
                class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium theme-hover"
                :style="getContainerHeaderAccentStyle(container)"
                @pointerdown.stop
                @click.stop="openRoleMenu($event, container.id)"
                @contextmenu.prevent
              ><Tag :size="11" />{{ roleLabels[container.role] }}</button>
            </div>
            <button class="rounded-md p-1 theme-hover" @pointerdown.stop @click="openContainerMenu($event, container.id)" @contextmenu.prevent><EllipsisVertical :size="15" /></button>
          </div>

            <div class="flex-1 overflow-auto p-4" data-container-body @contextmenu.prevent="openContainerTaskMenu($event, container.id)">
            <div class="flex flex-col gap-2">
              <template v-for="taskId in getRenderableTaskIds(container)" :key="`${container.id}-${taskId}`">
                <div v-if="taskId === '__placeholder__'" class="h-7 rounded-lg border border-dashed theme-placeholder" />
                <template v-else>
                    <div v-if="getTask(taskId)" data-task-chip :data-task-id="taskId" class="inline-flex w-max max-w-full cursor-grab items-center rounded-lg border px-3 py-1.5 text-sm font-normal tracking-[0.01em] shadow-sm transition" :class="{ 'opacity-25': taskDrag?.taskId === taskId, 'hover:-translate-y-0.5 hover:shadow-md': taskDrag?.taskId !== taskId }" :style="getTaskStyle(getTask(taskId)?.color || 'slate')" @pointerdown="startTaskDragFromContainer($event, getTask(taskId)!)" @contextmenu.prevent="openTaskMenu($event, taskId)">
                    <input
                      v-if="editingTaskId === taskId"
                      :id="`task-name-input-${taskId}`"
                      v-model="editingTaskName"
                      type="text"
                      class="w-36 max-w-[46vw] rounded-md border px-2 py-1 text-[13px] outline-none focus:ring-0 theme-input"
                      @pointerdown.stop
                      @keydown.enter.prevent="commitTaskRename(taskId)"
                      @keydown.esc.prevent="cancelTaskRename"
                      @blur="commitTaskRename(taskId)"
                    >
                    <span v-else class="truncate">{{ getTask(taskId)?.name }}</span>
                  </div>
                </template>
              </template>
            </div>
          </div>

          <button class="absolute bottom-1.5 right-1.5 cursor-se-resize rounded p-1 theme-text-muted theme-hover" @pointerdown="startResize($event, container)"><ArrowDownRight :size="15" /></button>
        </div>
      </div>
    </div>
      <div
        class="absolute z-50 flex items-stretch gap-2"
        data-floating-ui
        :style="{ left: `${layoutPositions.calendar.x}px`, top: `${layoutPositions.calendar.y}px`, transform: 'translateX(-50%) scale(1.1)', transformOrigin: 'top center' }"
      >
        <div
          ref="calendarBarRef"
          class="relative overflow-hidden rounded-2xl border theme-panel theme-panel-shadow px-3 py-2 backdrop-blur"
          @pointerdown.stop="devLayoutMode && startLayoutDrag($event, 'calendar')"
        >
          <div class="flex items-center gap-2" :class="devLayoutMode ? 'pointer-events-none' : ''">
          <button class="shrink-0 rounded-lg p-2 theme-btn theme-hover" @click="goToPreviousWeek"><ChevronLeft :size="18" /></button>
          <Transition :name="weekTransitionName" mode="out-in">
            <div :key="weekDays[0].toDateString()" class="grid flex-1 grid-cols-7 gap-1.5">
                <button v-for="day in weekDays" :key="day.toDateString()" class="rounded-xl border px-2 py-2 text-center transition" :class="[isSelectedDay(day) ? 'theme-day-selected' : 'theme-day', isToday(day) && !isSelectedDay(day) ? 'theme-day-today' : '', isPastDay(day) && !isSelectedDay(day) ? 'opacity-60' : '']" @click="selectDay(day)">
                <div class="text-[11px] uppercase tracking-wide">{{ day.toLocaleDateString(undefined, { weekday: 'long' }) }}</div>
                <div class="mx-auto my-1.5 w-10 border-t theme-divider" />
                <div class="text-[12px] font-semibold">{{ formatDayDDMMYY(day) }}</div>
              </button>
            </div>
          </Transition>
          <button class="shrink-0 rounded-lg p-2 theme-btn theme-hover" @click="goToNextWeek"><ChevronRight :size="18" /></button>
          </div>
        </div>
          <div
            class="flex flex-col overflow-hidden rounded-2xl border theme-panel backdrop-blur"
            :style="{ height: calendarBarHeight ? `${Math.max(0, calendarBarHeight - 2)}px` : 'auto' }"
            @pointerdown.stop="devLayoutMode && startLayoutDrag($event, 'calendar')"
          >
            <button class="flex-1 px-2 text-[11px] font-semibold transition" :class="persistentCanvas === 1 ? 'theme-day-selected' : 'theme-day'" @click="persistentCanvas = 1" :disabled="devLayoutMode">1</button>
            <div class="h-px theme-divider" />
            <button class="flex-1 px-2 text-[11px] font-semibold transition" :class="persistentCanvas === 2 ? 'theme-day-selected' : 'theme-day'" @click="persistentCanvas = 2" :disabled="devLayoutMode">2</button>
            <div class="h-px theme-divider" />
            <button class="flex-1 px-2 text-[11px] font-semibold transition" :class="persistentCanvas === 3 ? 'theme-day-selected' : 'theme-day'" @click="persistentCanvas = 3" :disabled="devLayoutMode">3</button>
          </div>
      </div>

    <div
      class="absolute z-50 w-[380px] max-w-[92vw] duration-300 ease-out"
      data-floating-ui
      :class="[
        devLayoutMode ? 'transition-none' : 'transition-all',
        panelOpen ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-8 pointer-events-none'
      ]"
      :style="{ left: `${layoutPositions.panel.x}px`, top: `${layoutPositions.panel.y}px` }"
      @pointerdown.stop="devLayoutMode && startLayoutDrag($event, 'panel')"
    >
      <div class="relative rounded-2xl border theme-panel theme-panel-shadow backdrop-blur" :class="devLayoutMode ? 'pointer-events-none' : ''">
        <div class="flex border-b p-2 theme-border">
          <button class="flex-1 rounded-lg px-3 py-2 text-sm font-semibold theme-btn" :class="panelTab === 'creator' ? 'theme-btn-primary' : 'theme-hover'" @click="panelTab = 'creator'">Task Creator</button>
          <button class="flex-1 rounded-lg px-3 py-2 text-sm font-semibold theme-btn" :class="panelTab === 'inventory' ? 'theme-btn-primary' : 'theme-hover'" @click="panelTab = 'inventory'">Task Inventory</button>
        </div>
  
            <div v-if="panelTab === 'creator'" class="space-y-3 p-4">
              <label class="block text-sm font-medium theme-text">Name
                <input v-model="creator.name" type="text" class="mt-1 w-full appearance-none rounded-lg border px-3 py-2 text-sm outline-none focus:ring-0 theme-input" placeholder="Task name" @contextmenu.prevent>
              </label>
              <div class="h-px theme-divider" />
              <div>
                <div class="mb-1 text-sm font-medium theme-text">Color</div>
                <input
                  :value="creatorColorT"
                  type="range"
                  :min="0"
                  :max="1"
                  :step="0.001"
                  class="h-2 w-full cursor-pointer appearance-none rounded-full border bg-transparent accent-white theme-border"
                  :style="mapSliderBg"
                  @input="onCreatorSliderInput"
                >
              </div>
              <div class="h-px theme-divider" />
              <fieldset class="space-y-2">
                <legend class="text-sm font-medium theme-text">Repeatability</legend>
                <div class="grid grid-cols-3 gap-1 rounded-xl border p-1 theme-border theme-panel">
                  <label class="cursor-pointer">
                    <input v-model="creator.repeatability" type="radio" value="daily" class="peer sr-only">
                    <span class="flex items-center justify-center rounded-lg px-2 py-2 text-sm font-medium transition theme-text" :class="creator.repeatability === 'daily' ? 'theme-btn-primary shadow' : 'theme-hover'">Daily</span>
                  </label>
                  <label class="cursor-pointer">
                    <input v-model="creator.repeatability" type="radio" value="weekly" class="peer sr-only">
                    <span class="flex items-center justify-center rounded-lg px-2 py-2 text-sm font-medium transition theme-text" :class="creator.repeatability === 'weekly' ? 'theme-btn-primary shadow' : 'theme-hover'">Weekly</span>
                  </label>
                  <label class="cursor-pointer">
                    <input v-model="creator.repeatability" type="radio" value="occasional" class="peer sr-only">
                    <span class="flex items-center justify-center rounded-lg px-2 py-2 text-sm font-medium transition theme-text" :class="creator.repeatability === 'occasional' ? 'theme-btn-primary shadow' : 'theme-hover'">Occasional</span>
                  </label>
                </div>
              </fieldset>
              <div class="h-px theme-divider" />
              <div>
                <div class="mb-2 text-sm font-medium theme-text">Preview</div>
                <div class="inline-flex w-max max-w-full items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-normal tracking-[0.01em] shadow-sm" :style="getTaskStyle(creator.color)">{{ creator.name.trim() || 'New task' }}</div>
              </div>
              <div class="h-px theme-divider" />
              <button class="w-full rounded-lg px-3 py-2 text-sm font-semibold theme-btn theme-btn-primary disabled:cursor-not-allowed disabled:opacity-40" :disabled="!creator.name.trim()" @click="saveTemplate">Save Template</button>
            </div>
  
            <div v-else class="max-h-[60vh] space-y-4 overflow-auto p-4">
              <div v-for="group in (['daily', 'weekly', 'occasional'] as Repeatability[])" :key="group" class="space-y-2">
                <div class="text-xs font-bold uppercase tracking-wide theme-text-muted">{{ repeatabilityLabels[group] }}</div>
                <div v-if="groupedTemplates[group].length === 0" class="rounded-lg border border-dashed px-3 py-2 text-xs theme-text-muted theme-border">No templates yet</div>
                <div
                  v-for="t in groupedTemplates[group]"
                  :key="t.id"
                  class="relative mr-[6px] inline-flex max-w-full items-start"
                >
                  <button
                    class="inline-flex max-w-full cursor-grab items-center rounded-lg border px-3 py-1.5 text-sm font-normal tracking-[0.01em] transition hover:-translate-y-0.5 hover:shadow-md"
                    :class="inventoryDeleteMode ? 'opacity-75' : ''"
                    :style="getTaskStyle(t.color)"
                    @pointerdown="!inventoryDeleteMode && startTaskDragFromInventory($event, t)"
                  >
                    {{ t.name }}
                  </button>
                  <button
                    v-if="inventoryDeleteMode"
                    class="absolute -right-1.5 -top-1.5 inline-flex h-5 w-5 items-center justify-center rounded-full border shadow-md theme-btn theme-btn-danger"
                    @pointerdown.stop
                    @click="removeTemplate(t.id)"
                  >
                    <X :size="10" />
                  </button>
                </div>
              </div>
              <div class="pt-2">
                <button
                  class="inline-flex w-full items-center justify-center gap-2 rounded-lg border px-3 py-2 text-sm font-semibold transition"
                  :class="inventoryDeleteMode ? 'theme-btn theme-btn-danger' : 'theme-btn theme-hover'"
                  @click="inventoryDeleteMode = !inventoryDeleteMode"
                >
                  <Trash2 :size="14" />
                  {{ inventoryDeleteMode ? 'Done Deleting' : 'Delete Templates' }}
                </button>
              </div>
            </div>
      </div>
    </div>

    <input ref="importThemeInputRef" type="file" accept="application/json" class="hidden" @change="onThemeImportChange">
    <input ref="importDataInputRef" type="file" accept="application/json" class="hidden" @change="onImportDataSelected">

    <div v-if="themePanelOpen" class="absolute z-[85] w-[440px] max-w-[96vw]" data-floating-ui :style="{ left: '16px', top: '96px' }">
      <DesignPanel
        :theme="theme"
        :base-presets="baseThemePresets"
        :user-presets="userThemePresets"
        :selected-preset-id="selectedThemeId"
        :theme-name="themeName"
        @select-preset="selectTheme"
        @reset="resetTheme"
        @export="exportThemeConfig"
        @import="openThemeImport"
        @save="saveCurrentTheme"
        @update-name="updateThemeName"
      />
    </div>

    <div
      class="absolute z-[85] duration-300"
      data-floating-ui
      :class="[
        devLayoutMode ? 'transition-none' : 'transition-opacity',
        minimapVisible ? 'opacity-100' : 'opacity-0 pointer-events-none'
      ]"
      :style="{ left: `${layoutPositions.minimap.x}px`, top: `${layoutPositions.minimap.y}px` }"
    >
      <div class="relative">
        <button
          v-if="devLayoutMode"
          class="absolute -left-2 -top-2 inline-flex items-center gap-1 rounded-md border px-2 py-1 text-[11px] font-semibold shadow-md theme-btn theme-btn-primary"
          data-layout-handle
          @pointerdown="startLayoutDrag($event, 'minimap')"
        >
          Drag
        </button>
        <div class="mb-1 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-wide theme-text-muted">
          <span>{{ Math.round(zoom * 100) }}%</span>
          <button v-if="Math.round(zoom * 100) !== 100" class="p-0.5 theme-hover" title="Reset zoom" @click="resetZoomToOne">
            <Undo2 :size="10" />
          </button>
        </div>
        <div class="relative overflow-hidden rounded-lg border theme-border" :style="{ width: `${minimapSize}px`, height: `${minimapSize}px` }">
          <div
            v-for="c in containersSorted"
            :key="c.id"
            class="absolute rounded-sm border"
            :style="{
              left: `${c.x * minimapScale}px`,
              top: `${c.y * minimapScale}px`,
              width: `${c.width * minimapScale}px`,
              height: `${c.height * minimapScale}px`,
              borderColor: resolveColor(c.color).border,
              background: 'rgba(15,23,42,0.35)'
            }"
          />
          <div
            class="absolute rounded-sm border"
            :style="{
              left: `${minimapViewport.x}px`,
              top: `${minimapViewport.y}px`,
              width: `${minimapViewport.width}px`,
              height: `${minimapViewport.height}px`,
              borderColor: 'rgba(226,232,240,0.9)',
              boxShadow: '0 0 0 1px rgba(2,6,23,0.6)'
            }"
          />
        </div>
      </div>
    </div>

    <div v-if="canvasMenu.visible" class="absolute z-[70] w-48 rounded-xl border p-1 theme-menu theme-panel-shadow" data-floating-ui :style="{ left: `${canvasMenu.x}px`, top: `${canvasMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="createContainerAt(canvasMenu.worldX, canvasMenu.worldY)"><Plus :size="14" />Create Container</button>
      <div v-if="containerClipboard" class="mx-1 my-1 h-px theme-divider" />
      <button v-if="containerClipboard" class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="pasteContainer({ x: canvasMenu.worldX, y: canvasMenu.worldY }); canvasMenu.visible = false"><ClipboardPaste :size="14" />Paste Container</button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="openCanvasPresetMenu"><SquarePen :size="14" />Layout Presets</button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-btn theme-btn-danger"
        @mousedown.stop.prevent="clearCurrentCanvas"
        @pointerdown.stop.prevent="clearCurrentCanvas"
        @pointerup.stop.prevent="clearCurrentCanvas"
        @click.stop.prevent="clearCurrentCanvas"
      >
        <Trash2 :size="14" />Clear Canvas
      </button>
    </div>

    <div v-if="canvasPresetMenu.visible" class="absolute z-[70] w-64 rounded-xl border p-1 theme-menu theme-panel-shadow" data-floating-ui :style="{ left: `${canvasPresetMenu.x}px`, top: `${canvasPresetMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <div class="flex items-center justify-between px-2.5 py-1.5 text-xs font-semibold uppercase tracking-wide theme-text-muted">
        <span>Presets</span>
        <div class="flex items-center gap-2">
          <Trash2 :size="14" />
          <button
            class="relative inline-flex h-4 w-8 items-center rounded-full border transition theme-btn"
            :class="presetDeleteMode ? 'theme-btn-danger' : 'theme-hover'"
            :aria-pressed="presetDeleteMode"
            aria-label="Toggle preset delete mode"
            @click="togglePresetDeleteMode"
          >
            <span
              class="inline-block h-3 w-3 rounded-full bg-white transition"
              :style="{ transform: presetDeleteMode ? 'translateX(16px)' : 'translateX(2px)' }"
            />
          </button>
        </div>
      </div>
      <div v-if="canvasPresets.length === 0" class="mx-2 mb-1 rounded-lg border border-dashed px-3 py-2 text-xs theme-text-muted theme-border">No presets yet</div>
      <button
        v-for="preset in canvasPresets"
        :key="preset.id"
        class="flex w-full items-center justify-between gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item"
        :class="presetDeleteMode && presetDeleteSelection.has(preset.id) ? 'theme-btn theme-btn-danger' : ''"
        @click="presetDeleteMode ? togglePresetSelection(preset.id) : applyCanvasPreset(preset)"
      >
        <span>{{ preset.name }}</span>
      </button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-btn"
        :class="presetDeleteMode ? (presetDeleteSelection.size > 0 ? 'theme-btn-danger' : 'opacity-50 cursor-not-allowed') : 'theme-btn-primary'"
        :disabled="presetDeleteMode && presetDeleteSelection.size === 0"
        @click="presetDeleteMode ? removeSelectedPresets() : saveCurrentCanvasPreset()"
      >
        <template v-if="presetDeleteMode">
          <Trash2 :size="14" />Delete Selected
        </template>
        <template v-else>
          <Plus :size="14" />Save Current as Preset
        </template>
      </button>
    </div>

    <div v-if="containerMenu.visible" class="absolute z-[70] w-56 rounded-xl border p-1 theme-menu theme-panel-shadow" data-floating-ui :style="{ left: `${containerMenu.x}px`, top: `${containerMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="renameContainer(containerMenu.id as string)"><SquarePen :size="14" />Edit Container</button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <div class="px-2.5 py-1.5">
        <div class="mb-1 text-xs font-semibold uppercase tracking-wide theme-text-muted">Color</div>
        <input
          v-model.number="containerColorT"
          type="range"
          :min="0"
          :max="1"
          :step="0.001"
          class="h-2 w-full cursor-pointer appearance-none rounded-full border bg-transparent accent-white theme-border"
          :style="mapSliderBg"
          @input="selectContainerSliderColor"
        >
      </div>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="bringToFront(containerMenu.id as string); containerMenu.visible = false"><BringToFront :size="14" />Bring to front</button>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="sendToBack(containerMenu.id as string); containerMenu.visible = false"><SendToBack :size="14" />Send to back</button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="copyContainer(containerMenu.id as string)"><Copy :size="14" />Copy</button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button class="mt-0.5 flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-btn theme-btn-danger" @click="removeContainer(containerMenu.id as string)"><Trash2 :size="14" />Remove</button>
    </div>

    <div v-if="taskMenu.visible" class="absolute z-[70] w-56 rounded-xl border p-1 theme-menu theme-panel-shadow" data-floating-ui :style="{ left: `${taskMenu.x}px`, top: `${taskMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="renameTask(taskMenu.id as string); taskMenu.visible = false"><SquarePen :size="14" />Rename Task</button>
      <div class="mx-1 my-1 h-px theme-divider" />
      <div class="px-2.5 py-1.5">
        <div class="mb-1 text-xs font-semibold uppercase tracking-wide theme-text-muted">Color Override</div>
        <input
          v-model.number="taskColorT"
          type="range"
          :min="0"
          :max="1"
          :step="0.001"
          class="h-2 w-full cursor-pointer appearance-none rounded-full border bg-transparent accent-white theme-border"
          :style="mapSliderBg"
          @input="selectTaskSliderColor"
        >
      </div>
      <div class="mx-1 my-1 h-px theme-divider" />
      <button class="mt-0.5 flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-btn theme-btn-danger" @click="removeTaskInstance(taskMenu.id as string); taskMenu.visible = false"><Trash2 :size="14" />Remove Task</button>
    </div>

    <div v-if="roleMenu.visible" class="absolute z-[70] w-44 rounded-xl border p-0.5 theme-menu theme-panel-shadow" data-floating-ui :style="{ left: `${roleMenu.x}px`, top: `${roleMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <div>
        <button class="flex w-full items-center gap-2 rounded-md pl-2.5 pr-1 py-1.5 text-left text-sm theme-text theme-menu-item" @click="setContainerRole(roleMenu.id as string, 'todo')"><Circle :size="12" />To-Do</button>
        <div class="mx-1 h-px theme-divider" />
        <button class="flex w-full items-center gap-2 rounded-md pl-2.5 pr-1 py-1.5 text-left text-sm theme-text theme-menu-item" @click="setContainerRole(roleMenu.id as string, 'done')"><Check :size="12" />Done</button>
        <div class="mx-1 h-px theme-divider" />
        <button class="flex w-full items-center gap-2 rounded-md pl-2.5 pr-1 py-1.5 text-left text-sm theme-text-muted theme-menu-item" @click="setContainerRole(roleMenu.id as string, null)"><Minus :size="12" />No role</button>
      </div>
    </div>

    <div v-if="containerTaskMenu.visible" class="absolute z-[70] w-48 rounded-xl border p-1 theme-menu theme-panel-shadow" data-floating-ui :style="{ left: `${containerTaskMenu.x}px`, top: `${containerTaskMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm theme-text theme-menu-item" @click="createTaskInContainer(containerTaskMenu.id as string)"><Plus :size="14" />Create Task</button>
    </div>

    <div v-if="inputModal.visible" class="fixed inset-0 z-[90] flex items-center justify-center p-4 theme-overlay" @pointerdown.self="closeInputModal">
      <div class="w-full max-w-md rounded-2xl border p-4 backdrop-blur theme-panel theme-panel-shadow">
        <div class="mb-3 text-lg font-semibold theme-text">{{ inputModal.title }}</div>
        <label class="block text-sm font-medium theme-text-muted">
          {{ inputModal.label }}
          <input v-model="inputModal.value" type="text" autofocus class="mt-1 w-full appearance-none rounded-lg border px-3 py-2 text-sm outline-none focus:ring-0 theme-input">
        </label>
        <div class="mt-4 flex justify-end gap-2">
          <button class="rounded-lg border px-3 py-2 text-sm theme-btn theme-hover" @click="closeInputModal">Cancel</button>
          <button class="rounded-lg px-3 py-2 text-sm font-semibold theme-btn theme-btn-primary disabled:opacity-40" :disabled="!inputModal.value.trim()" @click="submitInputModal">{{ inputModal.confirmText }}</button>
        </div>
      </div>
    </div>

    <div v-if="confirmModal.visible" class="fixed inset-0 z-[90] flex items-center justify-center p-4 theme-overlay" @pointerdown.self="closeConfirmModal">
      <div class="w-full max-w-md rounded-2xl border p-4 backdrop-blur theme-panel theme-panel-shadow">
        <div class="mb-2 text-lg font-semibold theme-text">{{ confirmModal.title }}</div>
        <div class="text-sm theme-text-muted">{{ confirmModal.message }}</div>
        <div class="mt-4 flex justify-end gap-2">
          <button class="rounded-lg border px-3 py-2 text-sm theme-btn theme-hover" @click="closeConfirmModal">Cancel</button>
          <button class="rounded-lg px-3 py-2 text-sm font-semibold theme-btn theme-btn-danger" @click="submitConfirmModal">{{ confirmModal.confirmText }}</button>
        </div>
      </div>
    </div>

    <div v-if="rolloverModal.visible" class="fixed inset-0 z-[90] flex items-center justify-center p-4 theme-overlay" @pointerdown.self="rolloverModal.visible = false">
      <div class="w-full max-w-md rounded-2xl border p-4 backdrop-blur theme-panel theme-panel-shadow">
        <div class="mb-2 text-lg font-semibold theme-text">{{ rolloverModal.title }}</div>
        <div class="text-sm theme-text-muted">{{ rolloverModal.message }}</div>
        <div class="mt-4 flex justify-end gap-2">
          <button class="rounded-lg px-3 py-2 text-sm font-semibold theme-btn theme-btn-primary" @click="rolloverModal.visible = false">OK</button>
        </div>
      </div>
    </div>

    <div v-if="taskDrag" class="pointer-events-none fixed z-[80] -translate-x-1/2 -translate-y-1/2 scale-105 rounded-lg border px-3 py-1.5 text-sm font-medium shadow-2xl" :style="{ ...getTaskStyle(taskDrag.color), left: `${taskDrag.x}px`, top: `${taskDrag.y}px` }">{{ taskDrag.name }}</div>
    <div v-if="dropChip" class="pointer-events-none fixed z-[82] -translate-x-1/2 -translate-y-1/2 rounded-lg border px-3 py-1.5 text-sm font-medium shadow-xl drop-fall" :style="{ ...getTaskStyle(dropChip.color), left: `${dropChip.x}px`, top: `${dropChip.y}px` }">{{ dropChip.name }}</div>

    <div
      class="absolute z-50"
      data-floating-ui
      :style="{ left: `${layoutPositions.clock.x}px`, top: `${layoutPositions.clock.y}px` }"
      @pointerdown.stop="devLayoutMode && startLayoutDrag($event, 'clock')"
    >
      <button
        class="liquid-clock font-semibold leading-none [font-family:-apple-system,BlinkMacSystemFont,'SF_Pro_Display','SF_Pro_Text',system-ui,sans-serif]"
        :class="`liquid-clock--${clockStyle}`"
        :style="{ fontSize: `${theme.clockSize}px`, opacity: theme.clockOpacity }"
        title="Switch clock style"
        @click="devLayoutMode ? undefined : cycleClockStyle()"
      >
        {{ time24 }}
      </button>
    </div>

    <div class="absolute bottom-4 right-4 z-[85] flex flex-col items-end gap-2" data-floating-ui @contextmenu.prevent>
      <div v-if="infoPanelOpen" class="w-[min(440px,92vw)] rounded-2xl border p-4 theme-panel theme-panel-shadow">
        <div class="mb-2 flex items-center justify-between">
          <div class="text-sm font-semibold theme-text">How Tmap Works</div>
          <button class="rounded-md p-1 theme-btn theme-hover" @click="infoPanelOpen = false"><X :size="14" /></button>
        </div>
        <div class="space-y-2 text-xs leading-relaxed theme-text-muted">
          <p><span class="font-semibold theme-text">Navigation:</span> Pan with middle-mouse or Space + drag. Use the week bar for daily canvases, or 1-3 for persistent canvases.</p>
          <p><span class="font-semibold theme-text">Tasks:</span> Create containers, drag tasks between them, and right-click for edit, color, copy, or delete.</p>
          <p><span class="font-semibold theme-text">Rollover:</span> To-Do containers roll forward at midnight or via the Rollover button for the selected day.</p>
          <p><span class="font-semibold theme-text">Presets:</span> Save and paste layout presets from the canvas menu.</p>
          <p><span class="font-semibold theme-text">Design:</span> Use Design Studio to edit themes, clock size/opacity, and container styling.</p>
          <p><span class="font-semibold theme-text">History:</span> Undo/Redo from the bottom bar or Ctrl/Cmd+Z.</p>
          <p><span class="font-semibold theme-text">Storage:</span> Data is saved locally in your browser. Export/import in Edit Layout.</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <div class="flex items-center gap-2 transition-all duration-200 ease-out" :class="quickMenuOpen ? 'translate-y-0 opacity-100' : 'translate-y-3 opacity-0 pointer-events-none'">
          <button
            class="inline-flex h-9 w-9 items-center justify-center rounded-full border shadow-lg transition theme-btn"
            :class="undoStack.length > 1 ? 'theme-hover' : 'opacity-50 cursor-not-allowed'"
            :disabled="undoStack.length <= 1"
            title="Undo"
            @click="undoHistory"
          >
            <Undo2 :size="16" />
          </button>
          <button
            class="inline-flex h-9 w-9 items-center justify-center rounded-full border shadow-lg transition theme-btn"
            :class="redoStack.length > 0 ? 'theme-hover' : 'opacity-50 cursor-not-allowed'"
            :disabled="redoStack.length === 0"
            title="Redo"
            @click="redoHistory"
          >
            <Redo2 :size="16" />
          </button>
          <div class="h-6 w-px theme-divider" />
          <button
            class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn"
            :class="rolloverCount > 0 ? 'theme-hover' : 'opacity-50 cursor-not-allowed'"
            :disabled="rolloverCount === 0"
            @click="triggerManualRollover"
          >
            Rollover
          </button>
          <div class="h-6 w-px theme-divider" />
          <button class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn" :class="themePanelOpen ? 'theme-btn-primary' : 'theme-hover'" @click="themePanelOpen = !themePanelOpen">
            Design
          </button>
          <button class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn" :class="devLayoutMode ? 'theme-btn-primary' : 'theme-hover'" @click="toggleDevLayoutMode">
            {{ devLayoutMode ? 'Save Layout' : 'Edit Layout' }}
          </button>
          <button v-if="devLayoutMode" class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn theme-btn-danger" @click="abortDevLayoutMode">
            Abort
          </button>
          <button v-if="devLayoutMode" class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn theme-hover" @click="exportAppData">
            Export Data
          </button>
          <button v-if="devLayoutMode" class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn theme-hover" @click="requestImportAppData">
            Import Data
          </button>
          <div class="h-6 w-px theme-divider" />
          <button class="inline-flex h-9 w-9 items-center justify-center rounded-full border shadow-lg transition theme-btn theme-hover" title="Info" @click="infoPanelOpen = !infoPanelOpen">
            <CircleHelp :size="16" />
          </button>
            <button v-if="devLayoutMode" class="inline-flex h-9 w-9 items-center justify-center rounded-full border shadow-lg transition theme-btn theme-hover" title="Reset Install Prompt" @click="resetInstallPrompt">
              <Minus :size="16" />
            </button>
          </div>
          <button
            class="inline-flex items-center justify-center rounded-full border px-3 py-2 text-xs font-semibold shadow-lg transition theme-btn theme-hover"
            title="Toggle panel"
            @click="panelOpen = !panelOpen"
          >
            {{ panelOpen ? 'Hide' : 'Tasks' }}
          </button>
          <button class="inline-flex h-9 w-9 items-center justify-center rounded-full border shadow-lg transition theme-btn theme-hover" title="Menu" @click="quickMenuOpen = !quickMenuOpen">
            <EllipsisVertical :size="16" />
          </button>
      </div>
    </div>
  </div>
</template>

