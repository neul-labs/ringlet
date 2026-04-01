const STORAGE_KEY_NOTIFY = 'ringlet_notify_sessions'
const STORAGE_KEY_SOUND = 'ringlet_notify_sound'

export function getNotifyEnabled(): boolean {
  return localStorage.getItem(STORAGE_KEY_NOTIFY) !== 'false'
}

export function setNotifyEnabled(enabled: boolean) {
  localStorage.setItem(STORAGE_KEY_NOTIFY, String(enabled))
}

export function getSoundEnabled(): boolean {
  return localStorage.getItem(STORAGE_KEY_SOUND) !== 'false'
}

export function setSoundEnabled(enabled: boolean) {
  localStorage.setItem(STORAGE_KEY_SOUND, String(enabled))
}

export async function requestNotificationPermission(): Promise<boolean> {
  if (!('Notification' in window)) return false
  if (Notification.permission === 'granted') return true
  if (Notification.permission === 'denied') return false
  const result = await Notification.requestPermission()
  return result === 'granted'
}

export function playNotificationSound() {
  try {
    const ctx = new AudioContext()
    const osc = ctx.createOscillator()
    const gain = ctx.createGain()
    osc.connect(gain)
    gain.connect(ctx.destination)
    osc.frequency.value = 880
    gain.gain.value = 0.3
    osc.start()
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.3)
    osc.stop(ctx.currentTime + 0.3)
  } catch {
    // AudioContext may not be available
  }
}

export function notifySessionComplete(profileAlias: string, exitCode: number | null) {
  if (!getNotifyEnabled()) return
  if (document.hasFocus()) return

  const success = exitCode === 0
  const title = 'Session Complete'
  const body = success
    ? `${profileAlias} finished successfully`
    : `${profileAlias} finished (exit code: ${exitCode ?? 'unknown'})`

  if ('Notification' in window && Notification.permission === 'granted') {
    new Notification(title, { body, silent: false })
  }

  if (getSoundEnabled()) {
    playNotificationSound()
  }
}
