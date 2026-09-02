// useTunnelStore — central state management for tunnels and config
import { useState, useEffect, useCallback } from "react";
import { api } from "../utils/api";
import type { AppConfig, TunnelStatus } from "../types";

export function useTunnelStore() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [statuses, setStatuses] = useState<TunnelStatus[]>([]);
  const [cloudflaredInstalled, setCloudflaredInstalled] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refreshConfig = useCallback(async () => {
    try {
      const cfg = await api.getConfig();
      setConfig(cfg);
    } catch (e: any) {
      setError(e.toString());
    }
  }, []);

  const refreshStatuses = useCallback(async () => {
    try {
      const sts = await api.getAllStatuses();
      setStatuses(sts);
    } catch (e: any) {
      // Silent fail — statuses refresh on interval
    }
  }, []);

  const checkInstalled = useCallback(async () => {
    try {
      const installed = await api.checkInstalled();
      setCloudflaredInstalled(installed);
    } catch {
      setCloudflaredInstalled(false);
    }
  }, []);

  // Initial load
  useEffect(() => {
    (async () => {
      setLoading(true);
      await Promise.all([refreshConfig(), refreshStatuses(), checkInstalled()]);
      setLoading(false);
    })();
  }, []);

  // Poll statuses every 3 seconds
  useEffect(() => {
    const interval = setInterval(refreshStatuses, 3000);
    return () => clearInterval(interval);
  }, [refreshStatuses]);

  return {
    config,
    statuses,
    cloudflaredInstalled,
    loading,
    error,
    refreshConfig,
    refreshStatuses,
    checkInstalled,
    setError,
  };
}
