/**
 * API utilities for interacting with the tracing backend
 */

/**
 * Get the API base URL
 */
export function getApiBaseUrl(): string {
  // In development mode, use the backend URL from environment
  if (import.meta.env.DEV && import.meta.env.VITE_BACKEND_URL) {
    return import.meta.env.VITE_BACKEND_URL;
  }
  // In production, use relative paths (will use current host)
  return ``;
}
