<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'

const SVG_NS = 'http://www.w3.org/2000/svg'

const props = defineProps({
  roseScale: { type: Number, default: 3.25 },
  color: { type: String, default: 'var(--color-accent)' },
  text: { type: String, default: '' },
})

const frameSize = computed(() => `${Math.round(props.roseScale * 30)}px`)

const config = {
  rotate: true,
  particleCount: 78,
  trailSpan: 0.32,
  durationMs: 5400,
  rotationDurationMs: 28000,
  pulseDurationMs: 4500,
  strokeWidth: 4.6,
  roseA: 9.2,
  roseABoost: 0.6,
  roseBreathBase: 0.72,
  roseBreathBoost: 0.28,
}

function point(progress, detailScale) {
  const t = progress * Math.PI * 2
  const a = config.roseA + detailScale * config.roseABoost
  const r = a * (config.roseBreathBase + detailScale * config.roseBreathBoost) * Math.cos(4 * t)
  return {
    x: 50 + Math.cos(t) * r * props.roseScale,
    y: 50 + Math.sin(t) * r * props.roseScale,
  }
}

const groupRef = ref(null)
const pathRef = ref(null)
let particles = []
let animFrameId = null
let startedAt = null

function normalizeProgress(progress) {
  return ((progress % 1) + 1) % 1
}

function getDetailScale(time) {
  const pulseProgress = (time % config.pulseDurationMs) / config.pulseDurationMs
  const pulseAngle = pulseProgress * Math.PI * 2
  return 0.52 + ((Math.sin(pulseAngle + 0.55) + 1) / 2) * 0.48
}

function getRotation(time) {
  if (!config.rotate) return 0
  return -((time % config.rotationDurationMs) / config.rotationDurationMs) * 360
}

function buildPath(detailScale, steps = 480) {
  return Array.from({ length: steps + 1 }, (_, index) => {
    const pt = point(index / steps, detailScale)
    return `${index === 0 ? 'M' : 'L'} ${pt.x.toFixed(2)} ${pt.y.toFixed(2)}`
  }).join(' ')
}

function getParticle(index, progress, detailScale) {
  const tailOffset = index / (config.particleCount - 1)
  const pt = point(normalizeProgress(progress - tailOffset * config.trailSpan), detailScale)
  const fade = Math.pow(1 - tailOffset, 0.56)
  return {
    x: pt.x,
    y: pt.y,
    radius: 0.9 + fade * 2.7,
    opacity: 0.04 + fade * 0.96,
  }
}

function render(now) {
  if (!startedAt) startedAt = now
  const time = now - startedAt
  const progress = (time % config.durationMs) / config.durationMs
  const detailScale = getDetailScale(time)

  groupRef.value?.setAttribute('transform', `rotate(${getRotation(time)} 50 50)`)
  pathRef.value?.setAttribute('d', buildPath(detailScale))

  for (let i = 0; i < particles.length; i++) {
    const p = getParticle(i, progress, detailScale)
    particles[i].setAttribute('cx', p.x.toFixed(2))
    particles[i].setAttribute('cy', p.y.toFixed(2))
    particles[i].setAttribute('r', p.radius.toFixed(2))
    particles[i].setAttribute('opacity', p.opacity.toFixed(3))
  }

  animFrameId = requestAnimationFrame(render)
}

onMounted(() => {
  pathRef.value?.setAttribute('stroke-width', String(config.strokeWidth))

  particles = Array.from({ length: config.particleCount }, () => {
    const circle = document.createElementNS(SVG_NS, 'circle')
    circle.setAttribute('fill', 'currentColor')
    groupRef.value?.appendChild(circle)
    return circle
  })

  animFrameId = requestAnimationFrame(render)
})

onBeforeUnmount(() => {
  if (animFrameId) {
    cancelAnimationFrame(animFrameId)
    animFrameId = null
  }
})
</script>

<template>
  <div class="rose-spinner inline-flex flex-col items-center gap-2" role="status" :aria-label="text || 'Loading'">
    <div class="rose-spinner-frame" :style="{ width: frameSize, height: frameSize }">
      <svg viewBox="0 0 100 100" fill="none" aria-hidden="true" :style="{ color: color }">
        <g ref="groupRef">
          <path ref="pathRef" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" opacity="0.1" />
        </g>
      </svg>
    </div>
    <p v-if="text" class="text-sm text-[var(--color-text-tertiary)]">{{ text }}</p>
  </div>
</template>
