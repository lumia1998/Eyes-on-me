<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { RouterLink } from "vue-router";
import { activityDurationMs, activityHeadline, activitySubline, activityUrl, formatDuration, formatTime } from "../lib/activity";
import { fetchAnalysisOverview, fetchDevices } from "../api";
import { DEFAULT_ANALYSIS_RANGE } from "../lib/analysis-range";
import type { AnalysisOverviewResponse, DevicesResponse } from "../types";

const props = defineProps<{
  connection: "connecting" | "live" | "closed";
  nowMs: number;
  refreshToken: number;
}>();

const loading = ref(true);
const error = ref<string | null>(null);
const devicesResponse = ref<DevicesResponse | null>(null);
const analysisResponse = ref<AnalysisOverviewResponse | null>(null);

async function loadData() {
  loading.value = true;

  try {
    const [devices, analysis] = await Promise.all([
      fetchDevices(),
      fetchAnalysisOverview(DEFAULT_ANALYSIS_RANGE)
    ]);
    devicesResponse.value = devices;
    analysisResponse.value = analysis;
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);
watch(() => props.refreshToken, loadData);
</script>

<template>
  <section v-if="loading" class="panel">
    <p>Loading devices...</p>
  </section>

  <section v-else-if="error" class="panel error-panel">
    <p>{{ error }}</p>
  </section>

  <template v-else>
    <section class="page-actions">
      <span class="muted">连接状态：{{ connection }}</span>
      <RouterLink class="button-link" :to="`/analysis?range=${DEFAULT_ANALYSIS_RANGE}`">进入分析页</RouterLink>
    </section>

    <section class="grid">
      <article class="panel">
        <div class="panel-header">
          <h2>设备汇总</h2>
          <span>{{ devicesResponse?.devices.length ?? 0 }}</span>
        </div>

        <ul class="card-list">
          <li
            v-for="item in devicesResponse?.devices ?? []"
            :key="item.device.eventId"
            class="device-card"
          >
            <div class="card-top">
              <div>
                <strong>{{ item.device.deviceId }}</strong>
                <p>{{ activityHeadline(item.device) }}</p>
              </div>
              <div class="action-row">
                <RouterLink class="button-link" :to="`/devices/${encodeURIComponent(item.device.deviceId)}`">
                  查看明细
                </RouterLink>
                <RouterLink class="button-link" :to="`/devices/${encodeURIComponent(item.device.deviceId)}/analysis?range=${DEFAULT_ANALYSIS_RANGE}`">
                  分析页
                </RouterLink>
              </div>
            </div>

            <span class="inline-meta">
              {{ item.device.app.name }} · {{ item.device.platform }} · 已持续 {{ formatDuration(activityDurationMs(item.device, props.nowMs)) }}
            </span>
            <p class="summary-line">{{ item.latestStatus?.statusText || activitySubline(item.device) }}</p>
            <code v-if="activityUrl(item.device)" class="url">{{ activityUrl(item.device) }}</code>
            <span class="inline-meta">最近更新 {{ formatTime(item.device.ts) }}</span>
          </li>
        </ul>
      </article>

      <article class="panel">
        <div class="panel-header">
          <h2>分析入口</h2>
          <span>{{ (analysisResponse?.topAppUsage.length ?? 0) + (analysisResponse?.topDomainUsage.length ?? 0) }}</span>
        </div>

        <div class="placeholder-stack">
          <div
            v-if="(analysisResponse?.topAppUsage.length ?? 0) === 0 && (analysisResponse?.topDomainUsage.length ?? 0) === 0"
            class="placeholder-card"
          >
            <strong>还没有分析数据</strong>
            <p>先启动当前这套 rust-monolith 的 server 和 agent，等设备上报活动后，这里会自动出现应用窗口和域名排行。</p>
            <span class="inline-meta">注意：必须连接到当前 `rust-monolith` 这套服务，不是旧目录里的 bundle。</span>
          </div>
          <div
            v-for="bucket in analysisResponse?.topAppUsage.slice(0, 3) ?? []"
            :key="bucket.key"
            class="placeholder-card"
          >
            <strong>{{ bucket.label }}</strong>
            <p>{{ bucket.sublabel || "应用窗口累计时长" }}</p>
            <span class="inline-meta">累计 {{ formatDuration(bucket.totalTrackedMs) }}</span>
          </div>
          <div
            v-for="bucket in analysisResponse?.topDomainUsage.slice(0, 2) ?? []"
            :key="bucket.key"
            class="placeholder-card"
          >
            <strong>{{ bucket.label }}</strong>
            <p>{{ bucket.sublabel || "浏览器域名累计时长" }}</p>
            <span class="inline-meta">累计 {{ formatDuration(bucket.totalTrackedMs) }}</span>
          </div>
        </div>
      </article>
    </section>
  </template>
</template>
