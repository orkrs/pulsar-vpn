<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import * as THREE from 'three'

const props = defineProps<{ speed: number; isConnected: boolean; active?: boolean }>()
const containerRef = ref<HTMLElement | null>(null)

let renderer: THREE.WebGLRenderer
let scene: THREE.Scene
let camera: THREE.PerspectiveCamera
let animationFrameId: number
let clock: THREE.Clock
let resizeObserver: ResizeObserver

let group: THREE.Group
let core: THREE.Mesh
let innerCore1: THREE.Points
let innerCore2: THREE.Points
let wireframe: THREE.Mesh
let particles: THREE.Points
let ambient: THREE.Points
let pointLightMain: THREE.PointLight

let coreMaterial: THREE.MeshPhysicalMaterial
let innerMaterial: THREE.PointsMaterial
let wireMaterial: THREE.MeshBasicMaterial
let particleMaterial: THREE.PointsMaterial
let ambientMaterial: THREE.PointsMaterial

let currentVisualSpeed = 0
let stateLerp = 0
let isRunning = false

// Переиспользуемые объекты (аллокации в onMounted, не в animation loop)
const tmpColor = new THREE.Color()
const cOnCore = new THREE.Color(0x2e1065)
const cOnEmissive = new THREE.Color(0x6d28d9)
const cOnInner = new THREE.Color(0xf3e8ff)
const cOnWire = new THREE.Color(0xc084fc)
const cOnParts = new THREE.Color(0xe9d5ff)
const cOnLight = new THREE.Color(0xa855f7)
const cOffCore = new THREE.Color(0x111111)
const cOffEmissive = new THREE.Color(0x000000)
const cOffInner = new THREE.Color(0x222222)
const cOffWire = new THREE.Color(0x333333)
const cOffParts = new THREE.Color(0x444444)
const cOffLight = new THREE.Color(0x4c1d95)

function startLoop() {
  if (isRunning) return
  isRunning = true
  clock.start()
  const animate = () => {
    if (!isRunning) return
    animationFrameId = requestAnimationFrame(animate)

    const delta = Math.min(clock.getDelta(), 0.05)
    const elapsed = clock.getElapsedTime()

    const targetState = props.isConnected ? 1 : 0
    stateLerp = THREE.MathUtils.lerp(stateLerp, targetState, delta * 3.0)

    const targetSpeed = props.isConnected ? (props.speed / 1024 / 1024) : 0
    currentVisualSpeed = THREE.MathUtils.lerp(currentVisualSpeed, targetSpeed, delta * 2.0)

    const activityMult = 0.2 + (stateLerp * 0.8) + (currentVisualSpeed * 0.05)

    group.rotation.y += delta * activityMult * 0.4
    group.rotation.z = Math.sin(elapsed * 0.3) * 0.1

    wireframe.rotation.x -= delta * activityMult * 0.3
    wireframe.rotation.y -= delta * activityMult * 0.2
    wireframe.scale.setScalar(1.05 + Math.cos(elapsed * 2) * (0.02 + stateLerp * 0.03))

    particles.rotation.y += delta * activityMult * 0.6
    particles.rotation.z = Math.cos(elapsed * 0.4) * 0.15
    particles.scale.setScalar(1.0 + stateLerp * 0.2 + Math.sin(elapsed * 5) * (0.02 * currentVisualSpeed))

    ambient.rotation.y -= delta * 0.02
    ambient.rotation.x = Math.sin(elapsed * 0.1) * 0.05

    const breathSpeed = 2 + stateLerp * 3 + currentVisualSpeed
    const breathAmp = 0.01 + stateLerp * 0.04 + (currentVisualSpeed * 0.01)
    core.scale.setScalar(1 + Math.sin(elapsed * breathSpeed) * breathAmp)
    core.rotation.y -= delta * activityMult * 0.5
    core.rotation.x += delta * activityMult * 0.3

    const plasmaSpeed = delta * (1 + stateLerp * 3 + currentVisualSpeed * 2)
    innerCore1.rotation.y += plasmaSpeed
    innerCore1.rotation.x += plasmaSpeed * 0.5

    innerCore2.rotation.y -= plasmaSpeed * 1.2
    innerCore2.rotation.z += plasmaSpeed * 0.8

    // Lerp c переиспользованием tmpColor — никаких new THREE.Color() в горячем цикле
    tmpColor.copy(cOffCore).lerp(cOnCore, stateLerp); coreMaterial.color.copy(tmpColor)
    tmpColor.copy(cOffEmissive).lerp(cOnEmissive, stateLerp); coreMaterial.emissive.copy(tmpColor)
    tmpColor.copy(cOffInner).lerp(cOnInner, stateLerp); innerMaterial.color.copy(tmpColor)
    tmpColor.copy(cOffWire).lerp(cOnWire, stateLerp); wireMaterial.color.copy(tmpColor)
    tmpColor.copy(cOffParts).lerp(cOnParts, stateLerp); particleMaterial.color.copy(tmpColor)

    coreMaterial.emissiveIntensity = (0.2 + stateLerp * 2.0) + (Math.sin(elapsed * 12) * 0.3 * stateLerp)
    innerMaterial.opacity = (stateLerp * 0.9) + (Math.sin(elapsed * 8) * 0.2 * stateLerp)
    wireMaterial.opacity = 0.1 + (stateLerp * 0.3)
    particleMaterial.opacity = 0.1 + (stateLerp * 0.6)

    tmpColor.copy(cOffLight).lerp(cOnLight, stateLerp); pointLightMain.color.copy(tmpColor)
    pointLightMain.intensity = (5 + stateLerp * 35) + (Math.sin(elapsed * 10) * 10 * stateLerp)

    renderer.render(scene, camera)
  }
  animationFrameId = requestAnimationFrame(animate)
}

function stopLoop() {
  if (!isRunning) return
  isRunning = false
  if (animationFrameId) cancelAnimationFrame(animationFrameId)
  animationFrameId = 0
}

onMounted(() => {
  if (!containerRef.value) return

  scene = new THREE.Scene()
  clock = new THREE.Clock(false)

  const width = containerRef.value.clientWidth
  const height = containerRef.value.clientHeight

  camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 100)
  camera.position.z = 7

  renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, preserveDrawingBuffer: true, powerPreference: 'low-power' })
  renderer.setSize(width, height)
  // devicePixelRatio cap = 1.5 даёт 2-4x ускорение на 2K/4K мониторах,
  // визуально разница для additive-blended частиц незаметна.
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5))
  containerRef.value.appendChild(renderer.domElement)

  const ambientLight = new THREE.AmbientLight(0xffffff, 0.3)
  scene.add(ambientLight)

  pointLightMain = new THREE.PointLight(0x9D68FF, 0, 20)
  pointLightMain.position.set(2, 2, 5)
  scene.add(pointLightMain)

  const pointLightSub = new THREE.PointLight(0x4c1d95, 2)
  pointLightSub.position.set(-2, -2, -5)
  scene.add(pointLightSub)

  group = new THREE.Group()
  scene.add(group)

  const coreGeo = new THREE.IcosahedronGeometry(0.8, 2)
  coreMaterial = new THREE.MeshPhysicalMaterial({
    color: 0x111111,
    emissive: 0x000000,
    roughness: 0.1,
    metalness: 0.9,
    transparent: true,
    opacity: 0.7,
    transmission: 0.5,
    wireframe: false,
  })
  core = new THREE.Mesh(coreGeo, coreMaterial)
  group.add(core)

  const createInnerCore = (count: number, radius: number, _size: number) => {
    const geo = new THREE.BufferGeometry()
    const pos = new Float32Array(count * 3)
    for (let i = 0; i < count; i++) {
      const r = Math.pow(Math.random(), 1/3) * radius
      const theta = Math.random() * 2 * Math.PI
      const phi = Math.acos(2 * Math.random() - 1)
      pos[i * 3] = r * Math.sin(phi) * Math.cos(theta)
      pos[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta)
      pos[i * 3 + 2] = r * Math.cos(phi)
    }
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3))
    return geo
  }

  innerMaterial = new THREE.PointsMaterial({
    size: 0.025,
    color: 0x333333,
    transparent: true,
    opacity: 0.0,
    blending: THREE.AdditiveBlending,
    depthWrite: false
  })

  innerCore1 = new THREE.Points(createInnerCore(600, 0.6, 0.03), innerMaterial)
  innerCore2 = new THREE.Points(createInnerCore(400, 0.4, 0.04), innerMaterial)
  core.add(innerCore1)
  core.add(innerCore2)

  const wireGeo = new THREE.IcosahedronGeometry(1.05, 2)
  wireMaterial = new THREE.MeshBasicMaterial({
    color: 0x333333,
    wireframe: true,
    transparent: true,
    opacity: 0.1
  })
  wireframe = new THREE.Mesh(wireGeo, wireMaterial)
  group.add(wireframe)

  const pCount = 2000
  const pGeo = new THREE.BufferGeometry()
  const pPos = new Float32Array(pCount * 3)
  for (let i = 0; i < pCount; i++) {
    const r = 1.6 + Math.random() * 1.5
    const theta = Math.random() * Math.PI * 2
    const phi = Math.acos(2 * Math.random() - 1)
    pPos[i * 3] = r * Math.sin(phi) * Math.cos(theta)
    pPos[i * 3 + 1] = (r * Math.sin(phi) * Math.sin(theta)) * 0.2
    pPos[i * 3 + 2] = r * Math.cos(phi)
  }
  pGeo.setAttribute('position', new THREE.BufferAttribute(pPos, 3))
  particleMaterial = new THREE.PointsMaterial({
    size: 0.03,
    color: 0x444444,
    transparent: true,
    opacity: 0.2,
    sizeAttenuation: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false
  })
  particles = new THREE.Points(pGeo, particleMaterial)
  group.add(particles)

  const dCount = 800
  const dGeo = new THREE.BufferGeometry()
  const dPos = new Float32Array(dCount * 3)
  for (let i = 0; i < dCount * 3; i++) dPos[i] = (Math.random() - 0.5) * 15
  dGeo.setAttribute('position', new THREE.BufferAttribute(dPos, 3))
  ambientMaterial = new THREE.PointsMaterial({
    size: 0.02,
    color: 0x333333,
    transparent: true,
    opacity: 0.2,
    depthWrite: false
  })
  ambient = new THREE.Points(dGeo, ambientMaterial)
  scene.add(ambient)

  // Запускаем анимацию только когда вкладка активна.
  if (props.active !== false) startLoop()

  resizeObserver = new ResizeObserver(() => {
    if (!containerRef.value) return
    const newWidth = containerRef.value.clientWidth
    const newHeight = containerRef.value.clientHeight
    if (newWidth === 0 || newHeight === 0) return
    camera.aspect = newWidth / newHeight
    camera.updateProjectionMatrix()
    renderer.setSize(newWidth, newHeight)
  })
  resizeObserver.observe(containerRef.value)
})

// Реактивная остановка/возобновление rAF при смене вкладки.
// Проп active = true на dashboard, иначе false. Без рендеринга вне dashboard
// WebView2 dispatcher не получает постоянных invalidate-сообщений.
watch(() => props.active, (v) => {
  if (v) startLoop(); else stopLoop()
})

onBeforeUnmount(() => {
  stopLoop()
  if (resizeObserver && containerRef.value) resizeObserver.unobserve(containerRef.value)

  if (renderer) {
    renderer.dispose()
    renderer.forceContextLoss()
  }

  const disposeMesh = (mesh: THREE.Mesh | THREE.Points) => {
    if (mesh) {
      mesh.geometry.dispose()
      ;(mesh.material as THREE.Material).dispose()
    }
  }

  disposeMesh(core)
  disposeMesh(wireframe)
  disposeMesh(particles)
  disposeMesh(ambient)
  disposeMesh(innerCore1)
  disposeMesh(innerCore2)
})
</script>

<template>
  <div ref="containerRef" class="w-full h-full pointer-events-none absolute inset-0"></div>
</template>
