<script setup lang="ts">
import { ArrowDownRight, BringToFront, Check, ChevronLeft, ChevronRight, Circle, CircleHelp, EllipsisVertical, Minus, Plus, SendToBack, SquarePen, Tag, Trash2, X } from 'lucide-vue-next'

const CANVAS_SIZE = 3000
const STORAGE_KEY = 'tmap:state:v1'
const CLOCK_STYLE_KEY = 'tmap:clock-style:v1'
const SCHEMA_VERSION = 1
const CONTAINER_HEADER_H = 44
const TASK_ROW_H = 34
const MIN_W = 220
const MIN_H = 140

type Repeatability = 'daily' | 'weekly' | 'occasional'
type ContainerRole = 'todo' | 'done' | null
type ClockStyle = 'solid' | 'transparent' | 'outline'

interface PaletteColor { key: string; name: string; bar: string; border: string; chip: string; chipText: string }
interface TaskTemplate { id: string; name: string; color: string; repeatability: Repeatability }
interface TaskInstance { id: string; templateId: string; name: string; color: string; containerId: string }
interface ContainerItem { id: string; name: string; x: number; y: number; width: number; height: number; color: string; role: ContainerRole; z: number; taskIds: string[] }
interface DayCanvas { containers: ContainerItem[]; tasks: TaskInstance[]; nextZ: number }
interface AppState { version: number; templates: TaskTemplate[]; canvases: Record<string, DayCanvas>; meta: { lastRolloverDate: string | null } }
interface TaskDrag { source: 'inventory' | 'container'; templateId?: string; taskId?: string; x: number; y: number; hoverContainerId: string | null; hoverIndex: number; name: string; color: string }

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
const state = ref<AppState>(createInitialState())
const selectedDate = ref(formatDateKey(new Date()))
const weekStart = ref(startOfWeek(new Date()))
const weekDirection = ref<'next' | 'prev'>('next')
const camera = reactive({ x: 0, y: 0 })
const hasCentered = ref(false)
const spacePressed = ref(false)
const now = ref(new Date())
const clockStyle = ref<ClockStyle>('transparent')

const canvasMenu = reactive({ visible: false, x: 0, y: 0, worldX: 0, worldY: 0 })
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
const vanishChip = ref<{ x: number; y: number; name: string; color: string } | null>(null)
const editingContainerId = ref<string | null>(null)
const editingContainerName = ref('')
const editingTaskId = ref<string | null>(null)
const editingTaskName = ref('')
const infoPanelOpen = ref(false)
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
let inputModalSubmit: ((value: string) => void) | null = null
let confirmModalSubmit: (() => void) | null = null

const pan = reactive({ active: false, sx: 0, sy: 0, cx: 0, cy: 0 })
let containerDrag: { id: string; sx: number; sy: number; ox: number; oy: number } | null = null
let resizeDrag: { id: string; sx: number; sy: number; ow: number; oh: number } | null = null
let saveTimer: ReturnType<typeof setTimeout> | null = null
let clockTimer: ReturnType<typeof setInterval> | null = null

const currentCanvas = computed(() => ensureCanvas(selectedDate.value))
const containersSorted = computed(() => [...currentCanvas.value.containers].sort((a, b) => a.z - b.z))
const weekDays = computed(() => Array.from({ length: 7 }, (_, i) => addDays(weekStart.value, i)))
const weekTransitionName = computed(() => `week-slide-${weekDirection.value}`)
const groupedTemplates = computed(() => ({
  daily: state.value.templates.filter((t) => t.repeatability === 'daily'),
  weekly: state.value.templates.filter((t) => t.repeatability === 'weekly'),
  occasional: state.value.templates.filter((t) => t.repeatability === 'occasional')
}))
const time24 = computed(() => now.value.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false }))
const mapSliderBg = computed(() => ({
  background: 'linear-gradient(90deg, #ff3b30 0%, #ff9500 16%, #ffcc00 32%, #34c759 48%, #00c7be 64%, #007aff 80%, #af52de 92%, #ff2d55 100%)'
}))

watch(state, () => {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => localStorage.setItem(STORAGE_KEY, JSON.stringify(state.value)), 120)
}, { deep: true })

onMounted(() => {
  loadState(); rolloverToToday(); selectedDate.value = formatDateKey(new Date()); weekStart.value = startOfWeek(new Date()); ensureCanvas(selectedDate.value); centerCamera()
  const savedClockStyle = localStorage.getItem(CLOCK_STYLE_KEY)
  if (savedClockStyle === 'solid' || savedClockStyle === 'transparent' || savedClockStyle === 'outline') clockStyle.value = savedClockStyle
  clockTimer = setInterval(() => { now.value = new Date() }, 1000)
  window.addEventListener('resize', centerCamera)
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
  document.addEventListener('pointerdown', closeMenus)
})

watch(clockStyle, (value) => {
  localStorage.setItem(CLOCK_STYLE_KEY, value)
})

onBeforeUnmount(() => {
  if (clockTimer) clearInterval(clockTimer)
  window.removeEventListener('resize', centerCamera)
  window.removeEventListener('pointermove', onMove)
  window.removeEventListener('pointerup', onUp)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
  document.removeEventListener('pointerdown', closeMenus)
})

function createInitialState(): AppState { return { version: SCHEMA_VERSION, templates: [], canvases: {}, meta: { lastRolloverDate: null } } }
function ensureCanvas(key: string): DayCanvas { if (!state.value.canvases[key]) state.value.canvases[key] = { containers: [], tasks: [], nextZ: 1 }; return state.value.canvases[key] }
function loadState() { const raw = localStorage.getItem(STORAGE_KEY); if (!raw) return; try { const parsed = JSON.parse(raw) as AppState; if (parsed.version === SCHEMA_VERSION && parsed.canvases && parsed.templates) state.value = parsed } catch { state.value = createInitialState() } }
function centerCamera() { if (!viewportRef.value || hasCentered.value) return; const rect = viewportRef.value.getBoundingClientRect(); camera.x = Math.round(rect.width / 2 - CANVAS_SIZE / 2); camera.y = Math.round(rect.height / 2 - CANVAS_SIZE / 2); hasCentered.value = true }
function onKeyDown(e: KeyboardEvent) { if (e.code === 'Space') spacePressed.value = true }
function onKeyUp(e: KeyboardEvent) { if (e.code === 'Space') spacePressed.value = false }
function closeMenus() { canvasMenu.visible = false; containerMenu.visible = false; taskMenu.visible = false; roleMenu.visible = false; containerTaskMenu.visible = false }
function toWorld(clientX: number, clientY: number) { const r = viewportRef.value?.getBoundingClientRect(); if (!r) return { x: 0, y: 0 }; return { x: clientX - r.left - camera.x, y: clientY - r.top - camera.y } }
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
function onViewportContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement | null
  if (target?.closest('[data-container]') || target?.closest('[data-floating-ui]')) return
  const w = toWorld(e.clientX, e.clientY)
  closeMenus(); canvasMenu.visible = true; canvasMenu.x = e.clientX; canvasMenu.y = e.clientY; canvasMenu.worldX = w.x; canvasMenu.worldY = w.y
}
function createContainerAt(x: number, y: number) {
  const c = currentCanvas.value
  const container: ContainerItem = { id: uid('container'), name: `Container ${c.containers.length + 1}`, x: clamp(x - 160, 0, CANVAS_SIZE - 260), y: clamp(y - 52, 0, CANVAS_SIZE - 170), width: 320, height: 230, color: 'oklch:0.62:0.18:220', role: null, z: c.nextZ++, taskIds: [] }
  c.containers.push(container)
  canvasMenu.visible = false
  startContainerRename(container.id)
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
function getDropIndex(containerId: string, worldY: number) {
  const c = getContainer(containerId); if (!c) return 0
  const ids = c.taskIds.filter((id) => id !== taskDrag.value?.taskId)
  const localY = worldY - c.y - CONTAINER_HEADER_H - 12
  for (let i = 0; i < ids.length; i += 1) { if (localY < i * TASK_ROW_H + TASK_ROW_H / 2) return i }
  return ids.length
}
function startTaskDragFromContainer(e: PointerEvent, task: TaskInstance) {
  if (e.button !== 0) return
  const target = e.target as HTMLElement | null
  if (target?.closest('input,button,textarea,select')) return
  e.stopPropagation(); closeMenus()
  taskDrag.value = { source: 'container', taskId: task.id, x: e.clientX, y: e.clientY, hoverContainerId: task.containerId, hoverIndex: getDropIndex(task.containerId, toWorld(e.clientX, e.clientY).y), name: task.name, color: task.color }
}
function startTaskDragFromInventory(e: PointerEvent, t: TaskTemplate) {
  if (e.button !== 0) return
  e.stopPropagation(); closeMenus()
  taskDrag.value = { source: 'inventory', templateId: t.id, x: e.clientX, y: e.clientY, hoverContainerId: null, hoverIndex: 0, name: t.name, color: t.color }
}
function onMove(e: PointerEvent) {
  if (pan.active) { camera.x = pan.cx + (e.clientX - pan.sx); camera.y = pan.cy + (e.clientY - pan.sy); return }

  if (containerDrag) {
    const c = getContainer(containerDrag.id); if (!c) return
    c.x = clamp(containerDrag.ox + (e.clientX - containerDrag.sx), 0, CANVAS_SIZE - c.width)
    c.y = clamp(containerDrag.oy + (e.clientY - containerDrag.sy), 0, CANVAS_SIZE - c.height)
    return
  }

  if (resizeDrag) {
    const c = getContainer(resizeDrag.id); if (!c) return
    c.width = clamp(resizeDrag.ow + (e.clientX - resizeDrag.sx), MIN_W, CANVAS_SIZE - c.x)
    c.height = clamp(resizeDrag.oh + (e.clientY - resizeDrag.sy), MIN_H, CANVAS_SIZE - c.y)
    return
  }

  if (!taskDrag.value) return
  taskDrag.value.x = e.clientX; taskDrag.value.y = e.clientY
  const world = toWorld(e.clientX, e.clientY)
  const hovered = getTopContainerAt(world.x, world.y)
  if (!hovered || world.y < hovered.y + CONTAINER_HEADER_H || world.y > hovered.y + hovered.height - 8) { taskDrag.value.hoverContainerId = null; taskDrag.value.hoverIndex = 0; return }
  taskDrag.value.hoverContainerId = hovered.id
  taskDrag.value.hoverIndex = getDropIndex(hovered.id, world.y)
}

function onUp() {
  pan.active = false
  if (containerDrag) { containerDrag = null; draggingContainerId.value = null }
  if (resizeDrag) { resizeDrag = null; resizingId.value = null }
  if (!taskDrag.value) return

  const drag = taskDrag.value
  const target = drag.hoverContainerId ? getContainer(drag.hoverContainerId) : null
  if (!target) {
    vanishChip.value = { x: drag.x, y: drag.y, name: drag.name, color: drag.color }
    setTimeout(() => { vanishChip.value = null }, 190)
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
function rgbCss(rgb: { r: number, g: number, b: number }) {
  return `rgb(${rgb.r} ${rgb.g} ${rgb.b})`
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
  return {
    backgroundColor: `color-mix(in srgb, ${c.bar} 50%, black)`,
    color: '#f8fafc',
    borderColor: c.chip,
    borderWidth: '1px',
    borderLeftWidth: '4px'
  }
}
function getContainerHeaderAccentStyle(c: ContainerItem) {
  const p = resolveColor(c.color)
  return { borderColor: p.border, borderWidth: '2px' }
}
function getContainerStyle(c: ContainerItem) {
  const p = resolveColor(c.color)
  return {
    borderColor: p.border,
    borderWidth: '2px',
    zIndex: c.z,
    left: `${c.x}px`,
    top: `${c.y}px`,
    width: `${c.width}px`,
    height: `${c.height}px`,
    boxShadow: resizingId.value === c.id ? `0 0 0 2px ${p.border}, 0 10px 24px rgba(2, 6, 23, 0.42)` : undefined,
    filter: resizingId.value === c.id ? 'brightness(1.06)' : undefined
  }
}
function getHeaderStyle(c: ContainerItem) { const p = resolveColor(c.color); return { backgroundColor: p.bar, borderBottomColor: p.border, color: '#f8fafc' } }

function goToPreviousWeek() { weekDirection.value = 'prev'; weekStart.value = addDays(weekStart.value, -7) }
function goToNextWeek() { weekDirection.value = 'next'; weekStart.value = addDays(weekStart.value, 7) }
function selectDay(day: Date) { selectedDate.value = formatDateKey(day) }
function isToday(day: Date) { return formatDateKey(day) === formatDateKey(new Date()) }
function isSelectedDay(day: Date) { return formatDateKey(day) === selectedDate.value }
function isPastDay(day: Date) { return formatDateKey(day) < formatDateKey(new Date()) }
function rolloverToToday() {
  const today = new Date(); today.setHours(0, 0, 0, 0)
  const start = state.value.meta.lastRolloverDate ? parseDateKey(state.value.meta.lastRolloverDate) : addDays(today, -1)
  let cursor = addDays(start, 1)
  while (cursor <= today) {
    const sourceKey = formatDateKey(addDays(cursor, -1)); const targetKey = formatDateKey(cursor)
    const source = state.value.canvases[sourceKey]
    if (source) {
      const todoIds = source.containers.filter((c) => c.role === 'todo').flatMap((c) => c.taskIds)
      if (todoIds.length > 0) {
        const map = new Map(source.tasks.map((t) => [t.id, t]))
        const copied = todoIds.map((id) => map.get(id)).filter((x): x is TaskInstance => Boolean(x))
        if (copied.length > 0) {
          const target = ensureCanvas(targetKey)
          const roll: ContainerItem = { id: uid('container'), name: 'Rolled Over To-Do', x: 160, y: 160, width: 340, height: 240, color: 'amber', role: 'todo', z: target.nextZ++, taskIds: [] }
          target.containers.push(roll)
          for (const t of copied) { const clone: TaskInstance = { id: uid('task'), templateId: t.templateId, name: t.name, color: t.color, containerId: roll.id }; target.tasks.push(clone); roll.taskIds.push(clone.id) }
        }
      }
    }
    state.value.meta.lastRolloverDate = formatDateKey(cursor)
    cursor = addDays(cursor, 1)
  }
}
</script>

<template>
  <div class="relative h-screen w-screen overflow-hidden select-none" @contextmenu.prevent>
    <div ref="viewportRef" class="absolute inset-0" @pointerdown="onViewportPointerDown" @contextmenu.prevent="onViewportContextMenu">
      <div class="absolute" :style="{ width: `${CANVAS_SIZE}px`, height: `${CANVAS_SIZE}px`, transform: `translate(${camera.x}px, ${camera.y}px)` }">
        <div class="absolute inset-0 rounded-3xl border border-slate-700/80 bg-[radial-gradient(circle_at_1px_1px,rgba(71,85,105,0.30)_1px,transparent_0)] [background-size:22px_22px] shadow-inner" />

        <div v-for="container in containersSorted" :key="container.id" data-container class="absolute flex flex-col overflow-hidden rounded-xl border bg-slate-900/58 backdrop-blur-md shadow-[0_14px_22px_rgba(0,0,0,0.34)]" :class="{ 'scale-[1.015] shadow-[0_16px_30px_rgba(0,0,0,0.38)]': draggingContainerId === container.id }" :style="getContainerStyle(container)">
          <div class="flex h-11 cursor-grab items-center justify-between border-b pl-4 pr-3 text-[15px] font-semibold" :style="getHeaderStyle(container)" @pointerdown="startContainerDrag($event, container)">
            <div class="flex items-center gap-2">
              <input
                v-if="editingContainerId === container.id"
                :id="`container-name-input-${container.id}`"
                v-model="editingContainerName"
                type="text"
                class="w-36 max-w-[46vw] rounded-md border border-slate-500/60 bg-slate-950/70 px-2 py-1 text-[14px] text-slate-100 outline-none ring-cyan-300 focus:ring"
                @pointerdown.stop
                @keydown.enter.prevent="commitContainerRename(container.id)"
                @keydown.esc.prevent="cancelContainerRename"
                @blur="commitContainerRename(container.id)"
              >
              <span v-else class="truncate">{{ container.name }}</span>
              <button
                v-if="!container.role"
                class="rounded-md border p-1 hover:bg-slate-700/70"
                :style="getContainerHeaderAccentStyle(container)"
                @pointerdown.stop
                @click.stop="openRoleMenu($event, container.id)"
                @contextmenu.prevent
              ><Plus :size="12" /></button>
              <button
                v-else
                class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[11px] font-medium hover:bg-slate-700/70"
                :style="getContainerHeaderAccentStyle(container)"
                @pointerdown.stop
                @click.stop="openRoleMenu($event, container.id)"
                @contextmenu.prevent
              ><Tag :size="11" />{{ roleLabels[container.role] }}</button>
            </div>
            <button class="rounded-md p-1 hover:bg-slate-700/70" @pointerdown.stop @click="openContainerMenu($event, container.id)" @contextmenu.prevent><EllipsisVertical :size="15" /></button>
          </div>

          <div class="flex-1 overflow-auto p-4" @contextmenu.prevent="openContainerTaskMenu($event, container.id)">
            <div class="flex flex-col gap-2">
              <template v-for="taskId in getRenderableTaskIds(container)" :key="`${container.id}-${taskId}`">
                <div v-if="taskId === '__placeholder__'" class="h-7 rounded-lg border border-dashed border-cyan-500/80 bg-cyan-900/25" />
                <template v-else>
                  <div v-if="getTask(taskId)" data-task-chip class="inline-flex w-max max-w-full cursor-grab items-center rounded-lg border px-3 py-1.5 text-sm font-normal tracking-[0.01em] shadow-sm transition" :class="{ 'opacity-25': taskDrag?.taskId === taskId, 'hover:-translate-y-0.5 hover:shadow-md': taskDrag?.taskId !== taskId }" :style="getTaskStyle(getTask(taskId)?.color || 'slate')" @pointerdown="startTaskDragFromContainer($event, getTask(taskId)!)" @contextmenu.prevent="openTaskMenu($event, taskId)">
                    <input
                      v-if="editingTaskId === taskId"
                      :id="`task-name-input-${taskId}`"
                      v-model="editingTaskName"
                      type="text"
                      class="w-36 max-w-[46vw] rounded-md border border-slate-500/60 bg-slate-950/70 px-2 py-1 text-[13px] text-slate-100 outline-none ring-cyan-300 focus:ring"
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

          <button class="absolute bottom-1.5 right-1.5 cursor-se-resize rounded p-1 text-slate-500 hover:bg-slate-800" @pointerdown="startResize($event, container)"><ArrowDownRight :size="15" /></button>
        </div>
      </div>
    </div>
    <div class="absolute left-1/2 top-4 z-50 w-[min(760px,92vw)] -translate-x-1/2" data-floating-ui>
      <div class="rounded-2xl border border-slate-700/80 bg-slate-900/85 px-3 py-2 shadow-[0_14px_26px_rgba(0,0,0,0.36)] backdrop-blur">
        <div class="flex items-center gap-2">
          <button class="shrink-0 rounded-lg p-2 hover:bg-slate-800" @click="goToPreviousWeek"><ChevronLeft :size="18" /></button>
          <Transition :name="weekTransitionName" mode="out-in">
            <div :key="weekDays[0].toDateString()" class="grid flex-1 grid-cols-7 gap-1.5">
              <button v-for="day in weekDays" :key="day.toDateString()" class="rounded-xl border px-2 py-2 text-center transition" :class="[isSelectedDay(day) ? 'border-cyan-300/70 bg-cyan-500 text-slate-950 shadow-lg' : 'border-slate-600/70 bg-slate-800/80 text-slate-200 hover:bg-slate-700', isToday(day) && !isSelectedDay(day) ? 'ring-2 ring-sky-300' : '', isPastDay(day) && !isSelectedDay(day) ? 'opacity-60' : '']" @click="selectDay(day)">
                <div class="text-[11px] uppercase tracking-wide">{{ day.toLocaleDateString(undefined, { weekday: 'long' }) }}</div>
                <div class="mx-auto my-1.5 w-10 border-t" :class="isSelectedDay(day) ? 'border-slate-900/45' : 'border-slate-500/70'" />
                <div class="text-[12px] font-semibold">{{ formatDayDDMMYY(day) }}</div>
              </button>
            </div>
          </Transition>
          <button class="shrink-0 rounded-lg p-2 hover:bg-slate-800" @click="goToNextWeek"><ChevronRight :size="18" /></button>
        </div>
      </div>
    </div>

    <div class="absolute right-4 top-24 z-50 w-[380px] max-w-[92vw]" data-floating-ui>
      <div class="rounded-2xl border border-slate-700/80 bg-slate-900/92 shadow-[0_16px_32px_rgba(0,0,0,0.37)] backdrop-blur">
        <div class="flex border-b border-slate-700 p-2">
          <button class="flex-1 rounded-lg px-3 py-2 text-sm font-semibold" :class="panelTab === 'creator' ? 'bg-cyan-500 text-slate-950' : 'text-slate-300 hover:bg-slate-800'" @click="panelTab = 'creator'">Task Creator</button>
          <button class="flex-1 rounded-lg px-3 py-2 text-sm font-semibold" :class="panelTab === 'inventory' ? 'bg-cyan-500 text-slate-950' : 'text-slate-300 hover:bg-slate-800'" @click="panelTab = 'inventory'">Task Inventory</button>
        </div>

        <div v-if="panelTab === 'creator'" class="space-y-3 p-4">
          <label class="block text-sm font-medium text-slate-200">Name
            <input v-model="creator.name" type="text" class="mt-1 w-full appearance-none rounded-lg border border-slate-600 bg-slate-950 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 outline-none ring-sky-300 focus:ring" placeholder="Task name" @contextmenu.prevent>
          </label>
          <div class="h-px bg-slate-700/80" />
          <div>
            <div class="mb-1 text-sm font-medium text-slate-200">Color</div>
            <input
              v-model.number="creatorColorT"
              type="range"
              :min="0"
              :max="1"
              :step="0.001"
              class="h-2 w-full cursor-pointer appearance-none rounded-full border border-slate-600 bg-transparent accent-white"
              :style="mapSliderBg"
              @input="selectCreatorSliderColor"
            >
          </div>
          <div class="h-px bg-slate-700/80" />
          <fieldset class="space-y-2">
            <legend class="text-sm font-medium text-slate-200">Repeatability</legend>
            <div class="grid grid-cols-3 gap-1 rounded-xl border border-slate-700 bg-slate-950 p-1">
              <label class="cursor-pointer">
                <input v-model="creator.repeatability" type="radio" value="daily" class="peer sr-only">
                <span class="flex items-center justify-center rounded-lg px-2 py-2 text-sm font-medium text-slate-300 transition peer-checked:bg-cyan-500 peer-checked:text-slate-950 peer-checked:shadow hover:bg-slate-800">Daily</span>
              </label>
              <label class="cursor-pointer">
                <input v-model="creator.repeatability" type="radio" value="weekly" class="peer sr-only">
                <span class="flex items-center justify-center rounded-lg px-2 py-2 text-sm font-medium text-slate-300 transition peer-checked:bg-cyan-500 peer-checked:text-slate-950 peer-checked:shadow hover:bg-slate-800">Weekly</span>
              </label>
              <label class="cursor-pointer">
                <input v-model="creator.repeatability" type="radio" value="occasional" class="peer sr-only">
                <span class="flex items-center justify-center rounded-lg px-2 py-2 text-sm font-medium text-slate-300 transition peer-checked:bg-cyan-500 peer-checked:text-slate-950 peer-checked:shadow hover:bg-slate-800">Occasional</span>
              </label>
            </div>
          </fieldset>
          <div class="h-px bg-slate-700/80" />
          <div>
            <div class="mb-2 text-sm font-medium text-slate-200">Preview</div>
            <div class="inline-flex w-max max-w-full items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-normal tracking-[0.01em] shadow-sm" :style="getTaskStyle(creator.color)">{{ creator.name.trim() || 'New task' }}</div>
          </div>
          <div class="h-px bg-slate-700/80" />
          <button class="w-full rounded-lg bg-cyan-500 px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-cyan-400 disabled:cursor-not-allowed disabled:opacity-40" :disabled="!creator.name.trim()" @click="saveTemplate">Save Template</button>
        </div>

        <div v-else class="max-h-[60vh] space-y-4 overflow-auto p-4">
          <div v-for="group in (['daily', 'weekly', 'occasional'] as Repeatability[])" :key="group" class="space-y-2">
            <div class="text-xs font-bold uppercase tracking-wide text-slate-400">{{ repeatabilityLabels[group] }}</div>
            <div v-if="groupedTemplates[group].length === 0" class="rounded-lg border border-dashed border-slate-600 px-3 py-2 text-xs text-slate-400">No templates yet</div>
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
                class="absolute -right-1.5 -top-1.5 inline-flex h-5 w-5 items-center justify-center rounded-full border border-rose-300/80 bg-rose-500 text-rose-50 shadow-md hover:bg-rose-400"
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
              :class="inventoryDeleteMode ? 'border-rose-400/60 bg-rose-500/20 text-rose-200' : 'border-slate-600 text-slate-300 hover:bg-slate-800'"
              @click="inventoryDeleteMode = !inventoryDeleteMode"
            >
              <Trash2 :size="14" />
              {{ inventoryDeleteMode ? 'Done Deleting' : 'Delete Templates' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="canvasMenu.visible" class="absolute z-[70] w-48 rounded-xl border border-slate-700/90 bg-slate-950 p-1 shadow-[0_14px_32px_rgba(0,0,0,0.38)]" data-floating-ui :style="{ left: `${canvasMenu.x}px`, top: `${canvasMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="createContainerAt(canvasMenu.worldX, canvasMenu.worldY)"><Plus :size="14" />Create Container</button>
    </div>

    <div v-if="containerMenu.visible" class="absolute z-[70] w-56 rounded-xl border border-slate-700/90 bg-slate-950 p-1 shadow-[0_14px_32px_rgba(0,0,0,0.38)]" data-floating-ui :style="{ left: `${containerMenu.x}px`, top: `${containerMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="renameContainer(containerMenu.id as string)"><SquarePen :size="14" />Edit Container</button>
      <div class="mx-1 my-1 h-px bg-slate-700/80" />
      <div class="px-2.5 py-1.5">
        <div class="mb-1 text-xs font-semibold uppercase tracking-wide text-slate-400">Color</div>
        <input
          v-model.number="containerColorT"
          type="range"
          :min="0"
          :max="1"
          :step="0.001"
          class="h-2 w-full cursor-pointer appearance-none rounded-full border border-slate-600 bg-transparent accent-white"
          :style="mapSliderBg"
          @input="selectContainerSliderColor"
        >
      </div>
      <div class="mx-1 my-1 h-px bg-slate-700/80" />
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="bringToFront(containerMenu.id as string); containerMenu.visible = false"><BringToFront :size="14" />Bring to front</button>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="sendToBack(containerMenu.id as string); containerMenu.visible = false"><SendToBack :size="14" />Send to back</button>
      <div class="mx-1 my-1 h-px bg-slate-700/80" />
      <button class="mt-0.5 flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-rose-400 hover:bg-slate-800 hover:text-rose-300" @click="removeContainer(containerMenu.id as string)"><Trash2 :size="14" />Remove</button>
    </div>

    <div v-if="taskMenu.visible" class="absolute z-[70] w-56 rounded-xl border border-slate-700/90 bg-slate-950 p-1 shadow-[0_14px_32px_rgba(0,0,0,0.38)]" data-floating-ui :style="{ left: `${taskMenu.x}px`, top: `${taskMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="renameTask(taskMenu.id as string); taskMenu.visible = false"><SquarePen :size="14" />Rename Task</button>
      <div class="mx-1 my-1 h-px bg-slate-700/80" />
      <div class="px-2.5 py-1.5">
        <div class="mb-1 text-xs font-semibold uppercase tracking-wide text-slate-400">Color Override</div>
        <input
          v-model.number="taskColorT"
          type="range"
          :min="0"
          :max="1"
          :step="0.001"
          class="h-2 w-full cursor-pointer appearance-none rounded-full border border-slate-600 bg-transparent accent-white"
          :style="mapSliderBg"
          @input="selectTaskSliderColor"
        >
      </div>
      <div class="mx-1 my-1 h-px bg-slate-700/80" />
      <button class="mt-0.5 flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-rose-400 hover:bg-slate-800 hover:text-rose-300" @click="removeTaskInstance(taskMenu.id as string); taskMenu.visible = false"><Trash2 :size="14" />Remove Task</button>
    </div>

    <div v-if="roleMenu.visible" class="absolute z-[70] w-44 rounded-xl border border-slate-700/90 bg-slate-950 p-0.5 shadow-[0_14px_32px_rgba(0,0,0,0.38)]" data-floating-ui :style="{ left: `${roleMenu.x}px`, top: `${roleMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <div class="divide-y divide-slate-700/80">
        <button class="flex w-full items-center gap-2 rounded-md pl-2.5 pr-1 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="setContainerRole(roleMenu.id as string, 'todo')"><Circle :size="12" />To-Do</button>
        <button class="flex w-full items-center gap-2 rounded-md pl-2.5 pr-1 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="setContainerRole(roleMenu.id as string, 'done')"><Check :size="12" />Done</button>
        <button class="flex w-full items-center gap-2 rounded-md pl-2.5 pr-1 py-1.5 text-left text-sm text-slate-400 hover:bg-slate-800" @click="setContainerRole(roleMenu.id as string, null)"><Minus :size="12" />No role</button>
      </div>
    </div>

    <div v-if="containerTaskMenu.visible" class="absolute z-[70] w-48 rounded-xl border border-slate-700/90 bg-slate-950 p-1 shadow-[0_14px_32px_rgba(0,0,0,0.38)]" data-floating-ui :style="{ left: `${containerTaskMenu.x}px`, top: `${containerTaskMenu.y}px` }" @pointerdown.stop @contextmenu.prevent>
      <button class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm text-slate-200 hover:bg-slate-800" @click="createTaskInContainer(containerTaskMenu.id as string)"><Plus :size="14" />Create Task</button>
    </div>

    <div v-if="inputModal.visible" class="fixed inset-0 z-[90] flex items-center justify-center bg-slate-950/65 p-4" @pointerdown.self="closeInputModal">
      <div class="w-full max-w-md rounded-2xl border border-slate-700 bg-slate-900/95 p-4 shadow-2xl backdrop-blur">
        <div class="mb-3 text-lg font-semibold text-slate-100">{{ inputModal.title }}</div>
        <label class="block text-sm font-medium text-slate-300">
          {{ inputModal.label }}
          <input v-model="inputModal.value" type="text" autofocus class="mt-1 w-full appearance-none rounded-lg border border-slate-600 bg-slate-950 px-3 py-2 text-sm text-slate-100 outline-none ring-cyan-400 focus:ring">
        </label>
        <div class="mt-4 flex justify-end gap-2">
          <button class="rounded-lg border border-slate-600 px-3 py-2 text-sm text-slate-200 hover:bg-slate-800" @click="closeInputModal">Cancel</button>
          <button class="rounded-lg bg-cyan-500 px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-cyan-400 disabled:opacity-40" :disabled="!inputModal.value.trim()" @click="submitInputModal">{{ inputModal.confirmText }}</button>
        </div>
      </div>
    </div>

    <div v-if="confirmModal.visible" class="fixed inset-0 z-[90] flex items-center justify-center bg-slate-950/65 p-4" @pointerdown.self="closeConfirmModal">
      <div class="w-full max-w-md rounded-2xl border border-slate-700 bg-slate-900/95 p-4 shadow-2xl backdrop-blur">
        <div class="mb-2 text-lg font-semibold text-slate-100">{{ confirmModal.title }}</div>
        <div class="text-sm text-slate-300">{{ confirmModal.message }}</div>
        <div class="mt-4 flex justify-end gap-2">
          <button class="rounded-lg border border-slate-600 px-3 py-2 text-sm text-slate-200 hover:bg-slate-800" @click="closeConfirmModal">Cancel</button>
          <button class="rounded-lg bg-rose-500 px-3 py-2 text-sm font-semibold text-rose-50 hover:bg-rose-400" @click="submitConfirmModal">{{ confirmModal.confirmText }}</button>
        </div>
      </div>
    </div>

    <div v-if="taskDrag" class="pointer-events-none fixed z-[80] -translate-x-1/2 -translate-y-1/2 scale-105 rounded-lg border px-3 py-1.5 text-sm font-medium shadow-2xl" :style="{ ...getTaskStyle(taskDrag.color), left: `${taskDrag.x}px`, top: `${taskDrag.y}px` }">{{ taskDrag.name }}</div>
    <div v-if="vanishChip" class="pointer-events-none fixed z-[82] -translate-x-1/2 -translate-y-1/2 rounded-lg border px-3 py-1.5 text-sm font-medium shadow-xl drop-vanish" :style="{ ...getTaskStyle(vanishChip.color), left: `${vanishChip.x}px`, top: `${vanishChip.y}px` }">{{ vanishChip.name }}</div>

    <div class="absolute left-4 top-4 z-50" data-floating-ui>
      <button class="liquid-clock text-6xl font-semibold leading-none [font-family:-apple-system,BlinkMacSystemFont,'SF_Pro_Display','SF_Pro_Text',system-ui,sans-serif]" :class="`liquid-clock--${clockStyle}`" title="Switch clock style" @click="cycleClockStyle">{{ time24 }}</button>
    </div>

    <div class="absolute bottom-4 right-4 z-[85] flex flex-col items-end gap-2" data-floating-ui @contextmenu.prevent>
      <div v-if="infoPanelOpen" class="w-[min(440px,92vw)] rounded-2xl border border-slate-700 bg-slate-950/98 p-4 shadow-[0_14px_32px_rgba(0,0,0,0.38)]">
        <div class="mb-2 flex items-center justify-between">
          <div class="text-sm font-semibold text-slate-100">How Tmap Works</div>
          <button class="rounded-md p-1 text-slate-300 hover:bg-slate-800 hover:text-slate-100" @click="infoPanelOpen = false"><X :size="14" /></button>
        </div>
        <div class="space-y-2 text-xs leading-relaxed text-slate-300">
          <p><span class="font-semibold text-slate-100">Canvas:</span> Middle-mouse drag to pan. You can also hold Space and drag with left mouse. Right-click empty canvas to create a container.</p>
          <p><span class="font-semibold text-slate-100">Containers:</span> Drag by header, resize from bottom-right arrow, and use the 3-dot menu for rename, color, layers, and removal. Right-click empty space inside a container to create a new task.</p>
          <p><span class="font-semibold text-slate-100">Tasks:</span> Drag tasks between containers. Right-click a task for rename, color override, or delete. Dropping outside a container removes that task instance.</p>
          <p><span class="font-semibold text-slate-100">Roles:</span> Set container role to To-Do or Done from the header tag. To-Do tasks roll over to the next day at midnight.</p>
          <p><span class="font-semibold text-slate-100">Templates:</span> Use Task Creator to make reusable templates. Inventory is an infinite source, so dragging out tasks does not consume the template.</p>
          <p><span class="font-semibold text-slate-100">Calendar:</span> Each day has its own canvas. Use the week bar to switch dates and revisit previous day layouts.</p>
          <p><span class="font-semibold text-slate-100">Storage:</span> Everything is local-only and saved in your browser storage on this device.</p>
        </div>
      </div>
      <button class="inline-flex h-9 w-9 items-center justify-center rounded-full border border-slate-600 bg-slate-900/90 text-slate-300 shadow-lg transition hover:bg-slate-800 hover:text-slate-100" title="Info" @click="infoPanelOpen = !infoPanelOpen">
        <CircleHelp :size="16" />
      </button>
    </div>
  </div>
</template>

