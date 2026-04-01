/**
 * Runtime environment detection and API configuration.
 *
 * Detects whether the app is running inside a Tauri webview or
 * as a standalone web app served by the daemon.
 */

let _baseUrl = '/api'
let _authToken = ''

/**
 * Check if the app is running inside a Tauri desktop application.
 */
export function isTauri(): boolean {
  return '__TAURI_INTERNALS__' in window
}

/**
 * Get the current API base URL.
 * Defaults to '/api' for the embedded web UI.
 */
export function getBaseUrl(): string {
  return _baseUrl
}

/**
 * Set the API base URL (used for non-Tauri remote connections).
 */
export function setBaseUrl(url: string): void {
  _baseUrl = url
}

/**
 * Get the current auth token (for non-Tauri fetch fallback).
 */
export function getAuthToken(): string {
  return _authToken
}

/**
 * Set the auth token.
 */
export function setAuthToken(token: string): void {
  _authToken = token
}

/**
 * Initialize API configuration.
 * Called once before the app mounts.
 */
export function initApiConfig(): void {
  // In Tauri mode, all API calls go through IPC — no base URL needed.
  // In browser mode, keep defaults (relative '/api' path).
}
