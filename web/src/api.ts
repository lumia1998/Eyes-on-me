import type {
  AnalysisRange,
  AnalysisOverviewResponse,
  DashboardSnapshot,
  DeviceAnalysisResponse,
  DeviceDetailResponse,
  DevicesResponse,
  StreamMessage
} from "./types";
import { DEFAULT_ANALYSIS_RANGE } from "./lib/analysis-range";

class ApiError extends Error {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.status = status;
  }
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url, {
    headers: {
      accept: "application/json"
    }
  });

  if (!response.ok) {
    throw new ApiError(`Failed to fetch ${url}: HTTP ${response.status}`, response.status);
  }

  return response.json() as Promise<T>;
}

export async function fetchCurrent(): Promise<DashboardSnapshot> {
  return fetchJson<DashboardSnapshot>("/api/current");
}

export async function fetchDevices(): Promise<DevicesResponse> {
  return fetchJson<DevicesResponse>("/api/devices");
}

export async function fetchDeviceDetail(deviceId: string): Promise<DeviceDetailResponse | null> {
  try {
    return await fetchJson<DeviceDetailResponse>(`/api/devices/${encodeURIComponent(deviceId)}`);
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) {
      return null;
    }

    throw error;
  }
}

export async function fetchAnalysisOverview(range: AnalysisRange = DEFAULT_ANALYSIS_RANGE): Promise<AnalysisOverviewResponse> {
  return fetchJson<AnalysisOverviewResponse>(`/api/analysis?range=${encodeURIComponent(range)}`);
}

export async function fetchDeviceAnalysis(
  deviceId: string,
  range: AnalysisRange = DEFAULT_ANALYSIS_RANGE
): Promise<DeviceAnalysisResponse | null> {
  try {
    return await fetchJson<DeviceAnalysisResponse>(
      `/api/devices/${encodeURIComponent(deviceId)}/analysis?range=${encodeURIComponent(range)}`
    );
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) {
      return null;
    }

    throw error;
  }
}

export function connectStream(onMessage: (message: StreamMessage<DashboardSnapshot>) => void): EventSource {
  const stream = new EventSource("/api/stream");

  stream.addEventListener("message", (event) => {
    const parsed = JSON.parse(event.data) as StreamMessage<DashboardSnapshot>;
    onMessage(parsed);
  });

  return stream;
}
