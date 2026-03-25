import { createRouter, createWebHistory } from "vue-router";
import AnalysisOverviewView from "./views/AnalysisOverviewView.vue";
import DeviceAnalysisView from "./views/DeviceAnalysisView.vue";
import DeviceDetailView from "./views/DeviceDetailView.vue";
import SummaryView from "./views/SummaryView.vue";

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "summary",
      component: SummaryView
    },
    {
      path: "/devices/:deviceId",
      name: "device-detail",
      component: DeviceDetailView
    },
    {
      path: "/analysis",
      name: "analysis-overview",
      component: AnalysisOverviewView
    },
    {
      path: "/devices/:deviceId/analysis",
      name: "device-analysis",
      component: DeviceAnalysisView
    }
  ]
});
