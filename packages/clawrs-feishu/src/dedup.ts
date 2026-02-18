/**
 * Simple in-memory message deduplication.
 * Feishu may push the same event more than once under certain conditions.
 */

const DEDUP_WINDOW_MS = 60_000; // 1 minute
const seen = new Map<string, number>();

let cleanupTimer: ReturnType<typeof setInterval> | undefined;

function ensureCleanup() {
  if (cleanupTimer) return;
  cleanupTimer = setInterval(() => {
    const now = Date.now();
    for (const [key, ts] of seen) {
      if (now - ts > DEDUP_WINDOW_MS) seen.delete(key);
    }
    if (seen.size === 0 && cleanupTimer) {
      clearInterval(cleanupTimer);
      cleanupTimer = undefined;
    }
  }, DEDUP_WINDOW_MS);
  if (cleanupTimer && typeof cleanupTimer === 'object' && 'unref' in cleanupTimer) {
    (cleanupTimer as NodeJS.Timeout).unref();
  }
}

/** Returns true if this messageId was already seen recently. */
export function isDuplicate(messageId: string | undefined): boolean {
  if (!messageId) return false;
  if (seen.has(messageId)) return true;
  seen.set(messageId, Date.now());
  ensureCleanup();
  return false;
}
