<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";
import { fetchAnalysisOverview } from "../api";
import { formatDateTime, formatDurationLong, formatTime, usageShare } from "../lib/activity";
import { ANALYSIS_RANGE_OPTIONS, analysisRangeLabel, normalizeAnalysisRange } from "../lib/analysis-range";
import type { AnalysisOverviewResponse, AnalysisRange } from "../types";

const props = defineProps<{
  connection: "connecting" | "live" | "closed";
  nowMs: number;
  refreshToken: number;
}>();

const route = useRoute();
const router = useRouter();

const loading = ref(true);
const error = ref<string | null>(null);
const analysisResponse = ref<AnalysisOverviewResponse | null>(null);
const selectedRange = computed(() => normalizeAnalysisRange(route.query.range));

const topAppUsage = computed(() => analysisResponse.value?.topAppUsage ?? []);
const topDomainUsage = computed(() => analysisResponse.value?.topDomainUsage ?? []);
const devices = computed(() => analysisResponse.value?.devices ?? []);
const hasAnyAnalysis = computed(() =>
  (analysisResponse.value?.deviceCount ?? 0) > 0 ||
  topAppUsage.value.length > 0 ||
  topDomainUsage.value.length > 0
);

async function loadData() {
  loading.value = true;

  try {
    analysisResponse.value = await fetchAnalysisOverview(selectedRange.value);
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

async function updateRange(range: AnalysisRange) {
  if (range === selectedRange.value) {
    return;
  }

  await router.replace({
    query: {
      ...route.query,
      range
    }
  });
}

onMounted(loadData);
watch([selectedRange, () => props.refreshToken], loadData);
</script>

<template>
  <section v-if="loading" class="panel">
    <p>Loading analysis overview...</p>
  </section>

  <section v-else-if="error" class="panel error-panel">
    <p>{{ error }}</p>
  </section>

  <template v-else-if="analysisResponse">
    <section class="page-actions">
      <RouterLink class="button-link" to="/">返回汇总</RouterLink>
      <span class="muted">连接状态：{{ connection }}</span>
      <span class="muted">统计范围：{{ analysisRangeLabel(selectedRange) }}</span>
      <span class="muted">生成时间：{{ formatTime(analysisResponse.generatedAt) }}</span>
    </section>

    <section class="panel range-panel">
      <div class="panel-header">
        <h2>时间范围</h2>
        <span>{{ analysisRangeLabel(selectedRange) }}</span>
      </div>
      <div class="range-switcher">
        <button
          v-for="option in ANALYSIS_RANGE_OPTIONS"
          :key="option.value"
          type="button"
          class="range-chip"
          :class="{ active: option.value === selectedRange }"
          @click="updateRange(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </section>

    <section v-if="!hasAnyAnalysis" class="panel empty-state">
      <span class="eyebrow">No Activity Yet</span>
      <h2 class="analysis-title">当前还没有可分析的活动记录</h2>
      <p class="analysis-lede">
        这通常不是页面问题，而是当前这个时间范围内还没有数据，或者当前这套 Eyes on Me 服务还没有积累到活动记录。
        先启动当前目录下的 server 和 client-desktop，等前台应用上报几次后，这里就会出现设备、窗口和域名时长统计。
      </p>
      <div class="placeholder-stack">
        <div class="placeholder-card">
          <strong>先启动服务端</strong>
          <p><code>/Users/wong/Code/RustLang/am-i-okay/rust-monolith/_scripts/run-server.sh</code></p>
        </div>
        <div class="placeholder-card">
          <strong>再启动客户端</strong>
          <p><code>/Users/wong/Code/RustLang/am-i-okay/rust-monolith/_scripts/run-agent.sh</code></p>
        </div>
        <div class="placeholder-card">
          <strong>确认不是旧 bundle</strong>
          <p>如果你之前跑的是外层旧 bundle，或者还在看老数据库，那边的数据不会自动出现在这里。</p>
        </div>
      </div>
    </section>

    <template v-else>
    <section class="analysis-summary">
      <article class="panel">
        <span class="eyebrow">Analysis Ledger</span>
        <h2 class="analysis-title">全局累计使用画像</h2>
        <p class="analysis-lede">基于 {{ analysisRangeLabel(selectedRange) }} 的活动记录，按设备、窗口和域名重新聚合使用时长。</p>

        <div class="stats-row">
          <div class="metric-block">
            <span class="label">累计记录时长</span>
            <strong>{{ formatDurationLong(analysisResponse.totalTrackedMs) }}</strong>
          </div>
          <div class="metric-block">
            <span class="label">设备数量</span>
            <strong>{{ analysisResponse.deviceCount }}</strong>
          </div>
          <div class="metric-block">
            <span class="label">最近生成</span>
            <strong>{{ formatDateTime(analysisResponse.generatedAt) }}</strong>
          </div>
        </div>
      </article>
    </section>

    <section class="grid">
      <article class="panel">
        <div class="panel-header">
          <h2>设备累计时长</h2>
          <span>{{ devices.length }}</span>
        </div>

        <ul class="usage-list">
          <li v-for="device in devices" :key="device.deviceId" class="usage-item">
            <div class="usage-copy">
              <strong>{{ device.deviceId }}</strong>
              <p>{{ device.latestStatusText || device.currentLabel }}</p>
              <span class="inline-meta">{{ device.platform }} · {{ device.eventCount }} 次切换 · 最近 {{ formatDateTime(device.lastSeen) }}</span>
            </div>
            <div class="usage-side">
              <strong>{{ formatDurationLong(device.totalTrackedMs) }}</strong>
              <span class="inline-meta">{{ usageShare(analysisResponse.totalTrackedMs, device.totalTrackedMs).toFixed(1) }}%</span>
              <RouterLink class="button-link" :to="`/devices/${encodeURIComponent(device.deviceId)}/analysis?range=${selectedRange}`">
                查看设备分析
              </RouterLink>
            </div>
          </li>
        </ul>
      </article>

      <article class="panel">
        <div class="panel-header">
          <h2>全局高频窗口</h2>
          <span>{{ topAppUsage.length }}</span>
        </div>

        <ul class="usage-list">
          <li v-for="bucket in topAppUsage" :key="bucket.key" class="usage-item">
            <div class="usage-copy">
              <strong>{{ bucket.label }}</strong>
              <p>{{ bucket.sublabel || "未提供附加信息" }}</p>
              <div class="usage-bar">
                <span :style="{ width: `${usageShare(analysisResponse.totalTrackedMs, bucket.totalTrackedMs)}%` }" />
              </div>
              <span class="inline-meta">最近 {{ formatDateTime(bucket.lastSeen) }}</span>
            </div>
            <div class="usage-side">
              <strong>{{ formatDurationLong(bucket.totalTrackedMs) }}</strong>
              <span class="inline-meta">{{ bucket.sessions }} 次进入</span>
            </div>
          </li>
        </ul>
      </article>
    </section>

    <section class="panel">
      <div class="panel-header">
        <h2>浏览器域名累计</h2>
        <span>{{ topDomainUsage.length }}</span>
      </div>

      <ul class="usage-list">
        <li v-for="bucket in topDomainUsage" :key="bucket.key" class="usage-item">
          <div class="usage-copy">
            <strong>{{ bucket.label }}</strong>
            <p>{{ bucket.sublabel || "未提供页面标题" }}</p>
            <div class="usage-bar">
              <span :style="{ width: `${usageShare(analysisResponse.totalTrackedMs, bucket.totalTrackedMs)}%` }" />
            </div>
          </div>
          <div class="usage-side">
            <strong>{{ formatDurationLong(bucket.totalTrackedMs) }}</strong>
            <span class="inline-meta">{{ bucket.sessions }} 次访问 · {{ formatDateTime(bucket.lastSeen) }}</span>
          </div>
        </li>
      </ul>
    </section>
    </template>
  </template>
</template>
