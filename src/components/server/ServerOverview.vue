<script setup>
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick, toRef } from 'vue'
import * as echarts from 'echarts/core'
import { PieChart, EffectScatterChart } from 'echarts/charts'
import { GeoComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([PieChart, EffectScatterChart, GeoComponent, CanvasRenderer])

import { useServerStore } from '@/stores/useServerStore'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useServerData } from '@/composables/useServerData'
import { useI18n } from '@/composables/useI18n'
import { useLogger } from '@/composables/useLogger'

const props = defineProps({
  serverId:  { type: String, required: true },
  sessionId: { type: String, default: null },
  host:      { type: String, required: true },
})

const { t } = useI18n()
const log = useLogger('ServerOverview')
const settings = useSettingsStore()
const serverStore = useServerStore()

const sessionIdRef = toRef(props, 'sessionId')
const data = useServerData(sessionIdRef)

const {
  hostname, osInfo, kernelVer, uptime, timezone,
  cpuCores, cpuPercent,
  memoryUsedMib, memoryTotalMib, memoryPercent,
  swapUsedMib, swapTotalMib, swapPercent,
  diskTotal, diskUsed, diskPercent,
  allIps, geoLocation,
  staticLoading, firstLoadDone,
} = data

// ── Accent / Theme ─────────────────────────────────────────────────
const accentColor = computed(() => settings.accentColor || '#007AFF')
const ACCENT = computed(() => accentColor.value)

const textPri = ref('#1d1d1f'); const textSec = ref('#86868b')
const textTer = ref('#aeaeb2'); const bgSec = ref('#f5f5f7')
const bgTer = ref('#e8e8ed'); const borderClr = ref('#d0d0d5')
const trackClr = ref('#e8e8ed')

function resolveTheme() {
  const d = document.documentElement.classList.contains('dark')
  if (d) { textPri.value='#f5f5f7'; textSec.value='#98989d'; textTer.value='#6e6e73'; bgSec.value='#252526'; bgTer.value='#2d2d30'; borderClr.value='#3d3d40'; trackClr.value='#3d3d40' }
  else   { textPri.value='#1d1d1f'; textSec.value='#86868b'; textTer.value='#aeaeb2'; bgSec.value='#f5f5f7'; bgTer.value='#e8e8ed'; borderClr.value='#d0d0d5'; trackClr.value='#e8e8ed' }
}

// ── Copy IP to clipboard ──────────────────────────────────────────
const copiedIp = ref(null) // the IP that was just copied, for brief feedback
function copyIp(ip) {
  navigator.clipboard.writeText(ip).then(() => {
    copiedIp.value = ip
    setTimeout(() => { copiedIp.value = null }, 1500)
  }).catch(() => {})
}

// ── Format ────────────────────────────────────────────────────────
function fmtMib(v) { if (v == null) return '--'; return v >= 1024 ? (v / 1024).toFixed(1) + ' GiB' : v + ' MiB' }

// ── ECharts ───────────────────────────────────────────────────────
const cpuGaugeEl = ref(null); const memGaugeEl = ref(null)
const diskGaugeEl = ref(null); const swapGaugeEl = ref(null)
const mapEl = ref(null)
let cpuGauge=null, memGauge=null, diskGauge=null, swapGauge=null, mapChart=null
let wGeoLoaded=false, wGeoP=null

function gaugeOption(pct) {
  const v = pct ?? 0; const c = ACCENT.value
  return { series: [{ type: 'pie', radius: ['68%','88%'], center: ['50%','50%'], silent: true, hoverAnimation: false, label: { show: false }, emphasis: { disabled: true }, animation: true, animationDuration: 500, data: [{ value: v, itemStyle: { color: c, borderRadius: 4 } }, { value: Math.max(0, 100 - v), itemStyle: { color: trackClr.value } }] }] }
}
function updateGauges() { if (cpuGauge) { cpuGauge.setOption(gaugeOption(cpuPercent.value), true); memGauge.setOption(gaugeOption(memoryPercent.value), true); diskGauge.setOption(gaugeOption(diskPercent.value), true); swapGauge.setOption(gaugeOption(swapPercent.value), true) } }

function mapOption() {
  const g = geoLocation.value
  return { backgroundColor: 'transparent', geo: { map: 'world', roam: false, zoom: 1.1, center: [0,20], silent: true, itemStyle: { areaColor: bgTer.value, borderColor: borderClr.value, borderWidth: 0.5 }, emphasis: { disabled: true } }, series: g ? [{ type: 'effectScatter', coordinateSystem: 'geo', data: [[g.lng, g.lat, 1]], symbolSize: 8, showEffectOn: 'render', rippleEffect: { brushType: 'stroke', scale: 3.5, period: 4.5 }, itemStyle: { color: ACCENT.value }, zlevel: 1 }] : [] }
}
function updateMap() { if (mapChart) mapChart.setOption(mapOption(), true) }

let resizeObs = null
function disposeAll() { resizeObs?.disconnect(); resizeObs = null; cpuGauge?.dispose(); cpuGauge = null; memGauge?.dispose(); memGauge = null; diskGauge?.dispose(); diskGauge = null; swapGauge?.dispose(); swapGauge = null; mapChart?.dispose(); mapChart = null }
function startResizeObserver() { resizeObs?.disconnect(); resizeObs = new ResizeObserver(() => { cpuGauge?.resize(); memGauge?.resize(); diskGauge?.resize(); swapGauge?.resize(); mapChart?.resize() }); [cpuGaugeEl.value, memGaugeEl.value, diskGaugeEl.value, swapGaugeEl.value, mapEl.value].forEach(el => { if (el) resizeObs.observe(el) }) }
async function loadWorldGeo() { if (wGeoLoaded) return true; if (wGeoP) return wGeoP; wGeoP = (async () => { try { const r = await fetch('/world.json'); const j = await r.json(); echarts.registerMap('world', j); wGeoLoaded = true; return true } catch (e) { log.error('world.json', e?.message); return false } })(); return wGeoP }
async function initCharts() { await nextTick(); resolveTheme(); const geoOk = await loadWorldGeo(); disposeAll(); if (cpuGaugeEl.value) { cpuGauge = echarts.init(cpuGaugeEl.value); memGauge = echarts.init(memGaugeEl.value); diskGauge = echarts.init(diskGaugeEl.value); swapGauge = echarts.init(swapGaugeEl.value); updateGauges() } if (mapEl.value && geoOk) { mapChart = echarts.init(mapEl.value); updateMap() } startResizeObserver() }

watch([cpuPercent, memoryPercent, diskPercent, swapPercent], () => { if (firstLoadDone.value) updateGauges() })
watch(geoLocation, loc => { if (loc && mapChart) updateMap() })

let dmObs = null
onMounted(() => { setTimeout(initCharts, 50); dmObs = new MutationObserver(() => { resolveTheme(); disposeAll(); initCharts() }); dmObs.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] }) })
onBeforeUnmount(() => { dmObs?.disconnect(); disposeAll() })

// ── Derived ───────────────────────────────────────────────────────
const memLabel  = computed(() => memoryUsedMib.value != null && memoryTotalMib.value != null ? `${fmtMib(memoryUsedMib.value)} / ${fmtMib(memoryTotalMib.value)}` : null)
const swapLabel = computed(() => { const u=swapUsedMib.value; const t=swapTotalMib.value; if (t==null||t===0) return null; return u!=null ? `${fmtMib(u)} / ${fmtMib(t)}` : `0 / ${fmtMib(t)}` })
const diskLabel = computed(() => diskTotal.value && diskUsed.value ? `${diskUsed.value} / ${diskTotal.value}` : null)
const hasIps    = computed(() => allIps.value.length > 0)
</script>

<template>
  <div class="h-full overflow-y-auto bg-[var(--color-bg-primary)]">
    <div class="max-w-6xl mx-auto px-4 py-5 sm:px-6 sm:py-6 space-y-5">

      <!-- ═══ SKELETON ═══ -->
      <Transition name="skel-fade">
        <div v-if="!firstLoadDone" class="space-y-5">
          <div class="flex items-start justify-between gap-3">
            <div class="space-y-2 flex-1">
              <div class="skeleton-block w-48 h-6 rounded" />
              <div class="skeleton-block w-72 h-3.5 rounded" />
            </div>
            <div class="flex gap-2">
              <div class="skeleton-block w-24 h-8 rounded-lg" />
              <div class="skeleton-block w-20 h-8 rounded-lg" />
            </div>
          </div>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
            <div v-for="i in 4" :key="i" class="skeleton-card rounded-xl p-4 flex flex-col items-center gap-3">
              <div class="skeleton-block w-20 h-20 rounded-full" />
              <div class="skeleton-block w-12 h-3 rounded" />
            </div>
          </div>
        </div>
      </Transition>

      <!-- ═══ CONTENT ═══ -->
      <div :class="['space-y-5 transition-opacity duration-400', firstLoadDone ? 'opacity-100' : 'opacity-0']">

        <!-- ── HEADER ──────────────────────────────────────────── -->
        <div class="flex items-start justify-between flex-wrap gap-3">
          <div class="min-w-0">
            <h2 class="text-lg font-bold tracking-tight" :style="{ color: textPri }">
              {{ hostname || props.host }}
            </h2>
            <div class="flex flex-wrap items-center gap-x-3 gap-y-0.5 mt-1">
              <span v-if="osInfo" class="text-xs" :style="{ color: textSec }">{{ osInfo }}</span>
              <span v-if="kernelVer" class="text-xs font-mono" :style="{ color: textTer }">{{ kernelVer }}</span>
              <span v-if="staticLoading" class="text-[10px] animate-pulse" :style="{ color: textTer }">{{ t('overview.loading') }}</span>
            </div>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button @click="serverStore.addTerminalTab(props.serverId)" class="btn-primary">
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
              {{ t('overview.openTerminal') }}
            </button>
            <button @click="serverStore.addFileManagerTab(props.serverId)" class="btn-secondary">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
              {{ t('overview.fileManager') }}
            </button>
          </div>
        </div>

        <!-- ── GAUGES ──────────────────────────────────────────── -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <div class="gauge-card">
            <div class="gauge-chart-wrap">
              <div ref="cpuGaugeEl" class="gauge-chart" />
              <div class="gauge-center"><span class="gauge-value" :style="{ color: textPri }">{{ cpuPercent ?? '--' }}</span><span class="gauge-unit" :style="{ color: textTer }">%</span></div>
            </div>
            <span class="gauge-title" :style="{ color: textSec }">{{ t('overview.cpu') }}</span>
            <span v-if="cpuCores" class="gauge-sub" :style="{ color: textTer }">{{ cpuCores }} {{ t('overview.cores') }}</span>
          </div>
          <div class="gauge-card">
            <div class="gauge-chart-wrap">
              <div ref="memGaugeEl" class="gauge-chart" />
              <div class="gauge-center"><span class="gauge-value" :style="{ color: textPri }">{{ memoryPercent ?? '--' }}</span><span class="gauge-unit" :style="{ color: textTer }">%</span></div>
            </div>
            <span class="gauge-title" :style="{ color: textSec }">{{ t('overview.memory') }}</span>
            <span v-if="memLabel" class="gauge-sub font-mono" :style="{ color: textTer }">{{ memLabel }}</span>
          </div>
          <div class="gauge-card">
            <div class="gauge-chart-wrap">
              <div ref="diskGaugeEl" class="gauge-chart" />
              <div class="gauge-center"><span class="gauge-value" :style="{ color: textPri }">{{ diskPercent ?? '--' }}</span><span class="gauge-unit" :style="{ color: textTer }">%</span></div>
            </div>
            <span class="gauge-title" :style="{ color: textSec }">{{ t('overview.storage') }}</span>
            <span v-if="diskLabel" class="gauge-sub font-mono" :style="{ color: textTer }">{{ diskLabel }}</span>
          </div>
          <div class="gauge-card">
            <div class="gauge-chart-wrap">
              <div ref="swapGaugeEl" class="gauge-chart" />
              <div class="gauge-center"><span class="gauge-value" :style="{ color: textPri }">{{ swapPercent ?? '--' }}</span><span class="gauge-unit" :style="{ color: textTer }">%</span></div>
            </div>
            <span class="gauge-title" :style="{ color: textSec }">{{ t('overview.swap') }}</span>
            <span v-if="swapLabel" class="gauge-sub font-mono" :style="{ color: textTer }">{{ swapLabel }}</span>
            <span v-else class="gauge-sub" :style="{ color: textTer }">{{ t('overview.none') }}</span>
          </div>
        </div>

        <!-- ── INFO CARDS + WORLD MAP ──────────────────────────── -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <!-- Left: info cards (2×2) -->
          <div class="grid grid-cols-2 gap-3 content-start">
            <!-- Uptime -->
            <div class="info-card" :style="{ background: bgSec, borderColor: borderClr }">
              <div class="info-card-row">
                <svg class="info-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" :style="{ color: textTer }"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                <span class="info-card-label" :style="{ color: textTer }">{{ t('overview.uptime') }}</span>
              </div>
              <p class="info-card-value" :style="{ color: uptime ? textPri : textTer }">{{ uptime || '--' }}</p>
            </div>
            <!-- Timezone -->
            <div class="info-card" :style="{ background: bgSec, borderColor: borderClr }">
              <div class="info-card-row">
                <svg class="info-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" :style="{ color: textTer }"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10"/><path d="M12 2a15.3 15.3 0 00-4 10 15.3 15.3 0 004 10"/></svg>
                <span class="info-card-label" :style="{ color: textTer }">{{ t('overview.timezone') }}</span>
              </div>
              <p class="info-card-value truncate" :style="{ color: timezone ? textPri : textTer }" :title="timezone">{{ timezone || '--' }}</p>
            </div>
            <!-- Latency -->
            <div class="info-card" :style="{ background: bgSec, borderColor: borderClr }">
              <div class="info-card-row">
                <svg class="info-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" :style="{ color: textTer }"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
                <span class="info-card-label" :style="{ color: textTer }">{{ t('overview.latency') }}</span>
              </div>
              <p class="info-card-value tabular-nums" :style="{ color: textPri }">
                {{ serverStore.activeServer?.latency != null ? serverStore.activeServer.latency + ' ms' : '--' }}
              </p>
            </div>
            <!-- Host + All IPs -->
            <div class="info-card row-span-2 flex flex-col" :style="{ background: bgSec, borderColor: borderClr }">
              <div class="info-card-row">
                <svg class="info-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" :style="{ color: textTer }"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
                <span class="info-card-label" :style="{ color: textTer }">{{ t('overview.host') }}</span>
              </div>
              <p class="info-card-value font-mono truncate mb-1" :style="{ color: textPri }">{{ props.host }}</p>
              <!-- IP list — each IP clickable to copy -->
              <div v-if="hasIps" class="flex-1 space-y-0.5">
                <button
                  v-for="ip in allIps" :key="ip"
                  @click="copyIp(ip)"
                  class="ip-line group"
                  :class="{ 'ip-copied': copiedIp === ip }"
                  :style="{ color: copiedIp === ip ? ACCENT : textTer }"
                >
                  <span class="font-mono ip-text">{{ ip }}</span>
                  <span class="ip-copy-hint" :style="{ color: ACCENT }">
                    {{ copiedIp === ip ? t('overview.copied') : t('overview.copy') }}
                  </span>
                </button>
              </div>
              <div v-else-if="!staticLoading" class="flex-1 flex items-center">
                <span class="text-xs" :style="{ color: textTer }">--</span>
              </div>
            </div>
          </div>

          <!-- Right: world map -->
          <div class="rounded-xl border flex flex-col min-h-[300px]" :style="{ borderColor: borderClr, background: bgSec }">
            <template v-if="geoLocation">
              <div class="flex items-center gap-2 px-4 pt-3">
                <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" :style="{ color: ACCENT }"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z"/><circle cx="12" cy="10" r="3"/></svg>
                <span class="text-xs font-semibold uppercase tracking-wide" :style="{ color: textSec }">{{ t('overview.location') }}</span>
                <span class="text-xs ml-auto font-medium" :style="{ color: textPri }">{{ geoLocation.label }}</span>
              </div>
            </template>
            <template v-else>
              <div class="flex flex-col items-center gap-2 p-6">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" :style="{ color: textTer }"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0118 0z"/><circle cx="12" cy="10" r="3"/></svg>
                <span class="text-xs" :style="{ color: textTer }">{{ t('overview.locationUnavailable') }}</span>
              </div>
            </template>
            <div ref="mapEl" class="flex-1 w-full min-h-[240px]" />
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── Skeleton ─────────────────────────────────────────────────── */
.skeleton-block { background: linear-gradient(90deg, var(--color-bg-secondary) 25%, var(--color-bg-tertiary) 50%, var(--color-bg-secondary) 75%); background-size: 200% 100%; animation: shimmer 1.5s ease-in-out infinite; }
.skeleton-card  { background: var(--color-bg-secondary); border: 1px solid var(--color-border); }
@keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
.skel-fade-enter-active { transition: opacity 200ms ease; }
.skel-fade-leave-active { transition: opacity 300ms ease; }
.skel-fade-enter-from, .skel-fade-leave-to { opacity: 0; }

/* ── Buttons ──────────────────────────────────────────────────── */
.btn-primary { display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; font-size: 0.75rem; font-weight: 500; border-radius: 0.5rem; color: #fff; background: var(--color-accent); transition: filter 150ms; }
.btn-primary:hover { filter: brightness(1.1); }
.btn-secondary { display: inline-flex; align-items: center; gap: 0.375rem; padding: 0.375rem 0.75rem; font-size: 0.75rem; font-weight: 500; border-radius: 0.5rem; border: 1px solid var(--color-border); color: var(--color-text-primary); background: var(--color-bg-secondary); transition: background 150ms; }
.btn-secondary:hover { background: var(--color-bg-tertiary); }

/* ── Gauge ────────────────────────────────────────────────────── */
.gauge-card { display: flex; flex-direction: column; align-items: center; gap: 0.125rem; padding: 1rem 0.75rem; border-radius: 0.75rem; border: 1px solid var(--color-border); background: var(--color-bg-secondary); transition: border-color 200ms; }
.gauge-card:hover { border-color: color-mix(in srgb, var(--color-accent) 30%, transparent); }
.gauge-chart-wrap { display: grid; place-items: center; width: 100px; height: 100px; position: relative; }
.gauge-chart { position: absolute; inset: 0; width: 100%; height: 100%; }
.gauge-center { position: relative; z-index: 2; display: flex; flex-direction: column; align-items: center; pointer-events: none; transform: translateY(2px); }
.gauge-value { font-size: 1.5rem; font-weight: 700; line-height: 1; font-variant-numeric: tabular-nums; }
.gauge-unit  { font-size: 0.625rem; font-weight: 500; line-height: 1; margin-top: -1px; }
.gauge-title { font-size: 0.65rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; }
.gauge-sub   { font-size: 0.6rem; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%; }

/* ── Info card ────────────────────────────────────────────────── */
.info-card { padding: 0.75rem; border-radius: 0.75rem; border: 1px solid; }
.info-card-row { display: flex; align-items: center; gap: 0.375rem; margin-bottom: 0.375rem; }
.info-card-icon { width: 0.875rem; height: 0.875rem; flex-shrink: 0; }
.info-card-label { font-size: 0.6rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; }
.info-card-value { font-size: 0.8125rem; font-weight: 500; }

/* ── IP list ──────────────────────────────────────────────────── */
.ip-line {
  display: flex; align-items: flex-start; gap: 0.25rem;
  width: 100%; padding: 1px 0; border: none; background: none; cursor: pointer;
  transition: color 150ms;
}
.ip-line:hover { color: var(--color-accent) !important; }
.ip-copied { }
.ip-copy-hint {
  font-size: 0.55rem; opacity: 0; transition: opacity 150ms;
  white-space: nowrap; flex-shrink: 0;
}
.ip-line:hover .ip-copy-hint,
.ip-copied .ip-copy-hint { opacity: 1; }
.ip-text {
  font-size: 0.7rem; word-break: break-all; overflow-wrap: break-word;
  min-width: 0;
}

/* ── Responsive ───────────────────────────────────────────────── */
@media (max-width: 640px) {
  .gauge-chart-wrap { width: 80px; height: 80px; }
  .gauge-value { font-size: 1.25rem; }
}
</style>
