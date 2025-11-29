import { useState, useEffect } from 'react';

/**
 * Get the API base URL
 */
function getApiBaseUrl(): string {
  if (import.meta.env.DEV && import.meta.env.VITE_BACKEND_URL) {
    return import.meta.env.VITE_BACKEND_URL;
  }
  return ``;
}

/**
 * Hook to fetch available log targets from the API
 */
export function useTargets() {
  const [targets, setTargets] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchTargets = async () => {
      try {
        const baseUrl = getApiBaseUrl();
        const url = baseUrl ? `${baseUrl}/api/targets` : `api/targets`;
        const response = await fetch(url);

        if (response.ok) {
          const data = await response.json();
          setTargets(data.targets || []);
        }
      } catch (err) {
        console.error(`Failed to fetch targets:`, err);
      } finally {
        setIsLoading(false);
      }
    };

    fetchTargets();

    // Refresh targets periodically
    const interval = setInterval(fetchTargets, 30000);
    return () => clearInterval(interval);
  }, []);

  return { targets, isLoading };
}
