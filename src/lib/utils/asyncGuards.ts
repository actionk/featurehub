export interface LatestGuard {
  next(): number;
  invalidate(): void;
  isCurrent(token: number): boolean;
}

export function createLatestGuard(): LatestGuard {
  let current = 0;
  return {
    next() {
      current += 1;
      return current;
    },
    invalidate() {
      current += 1;
    },
    isCurrent(token: number) {
      return token === current;
    },
  };
}

export interface PollingGate {
  run(work: () => Promise<void>): Promise<void>;
  isRunning(): boolean;
}

export function createPollingGate(): PollingGate {
  let running = false;
  return {
    async run(work: () => Promise<void>) {
      if (running) return;
      running = true;
      try {
        await work();
      } finally {
        running = false;
      }
    },
    isRunning() {
      return running;
    },
  };
}

export interface DebouncedTask {
  schedule(work: () => void): void;
  cancel(): void;
}

export function createDebouncedTask(delayMs: number): DebouncedTask {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return {
    schedule(work: () => void) {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        work();
      }, delayMs);
    },
    cancel() {
      if (timer) clearTimeout(timer);
      timer = null;
    },
  };
}
