import { invoke, withAddr } from "./transport";

export interface NetworkDiagnosticsSnapshot {
  enabled: boolean;
  refreshing: boolean;
  refreshScheduled: boolean;
  ip: string | null;
  countryCode: string | null;
  country: string | null;
  asn: number | null;
  organization: string | null;
  checkedAt: number | null;
  lastAttemptAt: number | null;
  source: string | null;
  error: string | null;
}

function normalizeSnapshot(value: unknown): NetworkDiagnosticsSnapshot {
  const record = value && typeof value === "object"
    ? (value as Record<string, unknown>)
    : {};
  const optionalString = (key: string) => {
    const candidate = record[key];
    return typeof candidate === "string" && candidate.trim()
      ? candidate.trim()
      : null;
  };
  const optionalNumber = (key: string) => {
    const candidate = Number(record[key]);
    return Number.isFinite(candidate) && candidate > 0 ? candidate : null;
  };
  return {
    enabled: record.enabled !== false,
    refreshing: record.refreshing === true,
    refreshScheduled: record.refreshScheduled === true,
    ip: optionalString("ip"),
    countryCode: optionalString("countryCode"),
    country: optionalString("country"),
    asn: optionalNumber("asn"),
    organization: optionalString("organization"),
    checkedAt: optionalNumber("checkedAt"),
    lastAttemptAt: optionalNumber("lastAttemptAt"),
    source: optionalString("source"),
    error: optionalString("error"),
  };
}

export const networkDiagnosticsClient = {
  async get(): Promise<NetworkDiagnosticsSnapshot> {
    return normalizeSnapshot(
      await invoke<unknown>("service_network_diagnostics_get", withAddr()),
    );
  },
  async refresh(): Promise<NetworkDiagnosticsSnapshot> {
    return normalizeSnapshot(
      await invoke<unknown>("service_network_diagnostics_refresh", withAddr()),
    );
  },
};
