/**
 * Lightweight in-app event bus for instant cross-component updates.
 *
 * Usage:
 *   // Emit from anywhere after a mutation:
 *   emit("session:finished", { featureId, sessionId });
 *
 *   // Subscribe in a component (auto-cleanup via $effect):
 *   $effect(() => subscribe("session:finished", (data) => { ... }));
 */

type EventCallback = (data?: any) => void;

const listeners = new Map<string, Set<EventCallback>>();

export function emit(event: string, data?: any) {
  const set = listeners.get(event);
  if (!set) return;
  for (const cb of set) {
    try {
      cb(data);
    } catch (e) {
      console.error(`[event-bus] error in "${event}" handler:`, e);
    }
  }
}

/**
 * Subscribe to an event. Returns an unsubscribe function.
 * When called inside $effect(), Svelte auto-calls the returned cleanup.
 */
export function subscribe(event: string, cb: EventCallback): () => void {
  let set = listeners.get(event);
  if (!set) {
    set = new Set();
    listeners.set(event, set);
  }
  set.add(cb);
  return () => {
    set!.delete(cb);
    if (set!.size === 0) listeners.delete(event);
  };
}
