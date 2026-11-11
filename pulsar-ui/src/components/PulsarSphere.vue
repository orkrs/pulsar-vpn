<script setup lang="ts">
import { onMounted, onBeforeUnmount, watch, ref } from 'vue'

const props = defineProps<{
  speed: number
  isConnected: boolean
  busy?: boolean
  active?: boolean
}>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const note = ref('CANVAS2D · 30 FPS · IDLE')

const TAU = Math.PI * 2
const STEP = 1000 / 30
const REDUCED = typeof matchMedia !== 'undefined'
  && matchMedia('(prefers-reduced-motion: reduce)').matches

const pts: [number, number, number][] = []
;(() => {
  const N = 240
  const ga = Math.PI * (3 - Math.sqrt(5))
  for (let i = 0; i < N; i++) {
    const y = 1 - (i / (N - 1)) * 2
    const r = Math.sqrt(1 - y * y)
    const th = ga * i
    pts.push([Math.cos(th) * r, y, Math.sin(th) * r])
  }
})()

type Mode = 'off' | 'on' | 'connecting'
let ctx: CanvasRenderingContext2D | null = null
let running = false
let raf = 0
let last = 0
let acc = 0
let mode: Mode = 'off'
let alpha = 0.3
let targetAlpha = 0.3
let rot = 0
let ringAcc = 0
let rings: { k: number }[] = []
let beamI = 0
let targetBeam = 0
let speedF = 0
let targetSpeedF = 0
let resizeObserver: ResizeObserver | null = null

function resolveMode(): Mode {
  if (props.busy) return 'connecting'
  if (props.isConnected) return 'on'
  return 'off'
}

function applyMode(m: Mode) {
  mode = m
  targetAlpha = m === 'on' ? 1 : m === 'connecting' ? 0.62 : 0.3
  targetBeam = m === 'on' ? 0.6 : m === 'connecting' ? 0.3 : 0
  if (m === 'connecting') note.value = 'CANVAS2D · 30 FPS · HANDSHAKE…'
  else if (m === 'on') note.value = 'CANVAS2D · 30 FPS · RENDERING'
  else if (!running) note.value = 'CANVAS2D · 30 FPS · IDLE'
  start()
}

function fit() {
  const cv = canvasRef.value
  if (!cv || !ctx) return
  const rect = cv.getBoundingClientRect()
  const dpr = Math.min(window.devicePixelRatio || 1, 1.5)
  cv.width = Math.max(1, Math.round(rect.width * dpr))
  cv.height = Math.max(1, Math.round(rect.height * dpr))
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  if (!running) draw(performance.now())
}

function drawBeams(cx: number, cy: number, R: number, now: number) {
  if (!ctx || beamI < 0.02) return
  const col = mode === 'on' ? '61,220,151' : '240,179,92'
  const L = R * 1.6
  ctx.save()
  ctx.translate(cx, cy)
  ctx.rotate(now * 0.00045 + Math.sin(now * 0.001) * 0.06)
  let g = ctx.createLinearGradient(-L, 0, L, 0)
  g.addColorStop(0, `rgba(${col},0)`)
  g.addColorStop(0.5, `rgba(${col},${0.55 * beamI * alpha})`)
  g.addColorStop(1, `rgba(${col},0)`)
  ctx.fillStyle = g
  ctx.fillRect(-L, -1.6, L * 2, 3.2)
  g = ctx.createLinearGradient(-L, 0, L, 0)
  g.addColorStop(0, `rgba(${col},0)`)
  g.addColorStop(0.5, `rgba(${col},${0.14 * beamI * alpha})`)
  g.addColorStop(1, `rgba(${col},0)`)
  ctx.fillStyle = g
  ctx.fillRect(-L, -5, L * 2, 10)
  ctx.rotate(Math.PI / 2.6)
  g = ctx.createLinearGradient(-L, 0, L, 0)
  g.addColorStop(0, `rgba(${col},0)`)
  g.addColorStop(0.5, `rgba(${col},${0.22 * beamI * alpha})`)
  g.addColorStop(1, `rgba(${col},0)`)
  ctx.fillStyle = g
  ctx.fillRect(-L, -1.2, L * 2, 2.4)
  ctx.restore()
}

function draw(now: number) {
  const cv = canvasRef.value
  if (!cv || !ctx) return
  const dpr = Math.min(window.devicePixelRatio || 1, 1.5)
  const w = cv.width / dpr
  const h = cv.height / dpr
  if (w < 2) return
  ctx.clearRect(0, 0, w, h)
  const cx = w / 2
  const cy = h / 2
  const R = Math.min(w, h) * 0.37

  for (const r of rings) {
    const rad = R * (1 + r.k * 1.05)
    ctx.beginPath()
    ctx.arc(cx, cy, rad, 0, TAU)
    ctx.strokeStyle = '#3ddc97'
    ctx.globalAlpha = (1 - r.k) * 0.22 * alpha
    ctx.lineWidth = 1
    ctx.stroke()
  }

  const pulse = 1 + Math.sin(now * 0.004) * (0.012 + speedF * 0.03)
  const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, R * 0.9 * pulse)
  g.addColorStop(0, `rgba(139,124,255,${0.30 * alpha})`)
  g.addColorStop(0.5, `rgba(139,124,255,${0.09 * alpha})`)
  g.addColorStop(1, 'rgba(139,124,255,0)')
  ctx.fillStyle = g
  ctx.fillRect(0, 0, w, h)

  drawBeams(cx, cy, R, now)

  ctx.beginPath()
  ctx.arc(cx, cy, R * 0.09 * pulse, 0, TAU)
  ctx.fillStyle = `rgba(236,238,246,${0.85 * alpha})`
  ctx.fill()

  const cs = Math.cos(rot)
  const sn = Math.sin(rot)
  const ct = Math.cos(0.42)
  const st = Math.sin(0.42)
  for (const p of pts) {
    const x1 = p[0] * cs + p[2] * sn
    const z1 = -p[0] * sn + p[2] * cs
    const y2 = p[1] * ct - z1 * st
    const z2 = p[1] * st + z1 * ct
    const depth = (z2 + 1) / 2
    const s = (0.7 + depth * 1.6) * (1 + speedF * 0.25)
    ctx.globalAlpha = (0.10 + 0.55 * depth) * alpha
    ctx.fillStyle = depth > 0.74 ? '#d6dbff' : '#8b7cff'
    ctx.fillRect(cx + x1 * R * pulse - s / 2, cy + y2 * R * pulse - s / 2, s, s)
  }
  ctx.globalAlpha = 1
}

function frame(now: number) {
  if (!running) return
  raf = requestAnimationFrame(frame)
  if (document.hidden) return
  const dt = Math.min(now - last, 120)
  last = now
  acc += dt
  if (acc < STEP) return
  acc %= STEP

  alpha += (targetAlpha - alpha) * 0.09
  beamI += (targetBeam - beamI) * 0.08
  speedF += (targetSpeedF - speedF) * 0.05
  rot += mode === 'on' ? 0.0042 + speedF * 0.007 : 0.0019

  if (mode === 'on') {
    ringAcc += STEP
    const period = Math.max(750, 1500 - speedF * 700)
    if (ringAcc > period) {
      ringAcc = 0
      rings.push({ k: 0 })
    }
  }
  for (const r of rings) r.k += STEP / 2000
  rings = rings.filter(r => r.k < 1)

  if (
    mode === 'off'
    && Math.abs(alpha - targetAlpha) < 0.004
    && Math.abs(beamI - targetBeam) < 0.01
    && rings.length === 0
  ) {
    running = false
    cancelAnimationFrame(raf)
    alpha = targetAlpha
    beamI = targetBeam
    draw(now)
    note.value = 'CANVAS2D · 30 FPS · IDLE'
    return
  }
  draw(now)
}

function start() {
  if (props.active === false) return
  if (REDUCED) {
    alpha = targetAlpha
    beamI = targetBeam
    speedF = targetSpeedF
    draw(performance.now())
    return
  }
  if (running) return
  running = true
  last = performance.now()
  acc = STEP
  raf = requestAnimationFrame(frame)
}

function stop() {
  if (!running) return
  running = false
  cancelAnimationFrame(raf)
  raf = 0
}

function onVisibility() {
  if (document.hidden) {
    stop()
  } else if (mode !== 'off' || alpha > 0.31 || beamI > 0.02) {
    if (props.active !== false) start()
  }
}

onMounted(() => {
  const cv = canvasRef.value
  if (!cv) return
  ctx = cv.getContext('2d', { alpha: true })
  if (!ctx) return

  resizeObserver = new ResizeObserver(() => fit())
  resizeObserver.observe(cv)
  document.addEventListener('visibilitychange', onVisibility)
  fit()
  applyMode(resolveMode())
})

watch(
  () => [props.isConnected, props.busy, props.active] as const,
  () => {
    if (props.active === false) {
      stop()
      return
    }
    applyMode(resolveMode())
  },
)

watch(
  () => props.speed,
  (v) => {
    targetSpeedF = Math.min(1, (v || 0) / (1024 * 1024) / 8)
  },
)

onBeforeUnmount(() => {
  stop()
  document.removeEventListener('visibilitychange', onVisibility)
  if (resizeObserver && canvasRef.value) resizeObserver.unobserve(canvasRef.value)
  resizeObserver = null
  ctx = null
})
</script>

<template>
  <div class="sphere-root">
    <canvas ref="canvasRef" class="pulsar-canvas" width="380" height="380" aria-hidden="true" />
    <span class="orbit-note">{{ note }}</span>
  </div>
</template>

<style scoped>
.sphere-root {
  position: relative;
  width: 100%;
  height: 100%;
  display: grid;
  place-items: center;
}
.pulsar-canvas {
  width: min(42vh, 380px);
  height: min(42vh, 380px);
  max-width: 100%;
  max-height: 100%;
}
.orbit-note {
  position: absolute;
  bottom: -4px;
  left: 50%;
  transform: translateX(-50%);
  font: 500 10.5px var(--f-mono);
  letter-spacing: 0.1em;
  color: var(--faint);
  white-space: nowrap;
}
@media (max-width: 760px) {
  .pulsar-canvas {
    width: 210px;
    height: 210px;
  }
}
</style>
