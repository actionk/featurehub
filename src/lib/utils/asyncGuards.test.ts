import { describe, expect, test, vi } from "vitest";
import { createDebouncedTask, createLatestGuard, createPollingGate } from "./asyncGuards";

describe("createLatestGuard", () => {
  test("only allows the most recent request to apply results", () => {
    const guard = createLatestGuard();
    const first = guard.next();
    const second = guard.next();

    expect(guard.isCurrent(first)).toBe(false);
    expect(guard.isCurrent(second)).toBe(true);
  });

  test("invalidates all previous requests", () => {
    const guard = createLatestGuard();
    const token = guard.next();

    guard.invalidate();

    expect(guard.isCurrent(token)).toBe(false);
  });
});

describe("createPollingGate", () => {
  test("prevents overlapping polling work", async () => {
    const gate = createPollingGate();
    let release!: () => void;
    let firstRun = true;
    const work = vi.fn(
      () => {
        if (!firstRun) return Promise.resolve();
        firstRun = false;
        return new Promise<void>((resolve) => {
          release = resolve;
        });
      },
    );

    const first = gate.run(work);
    const second = gate.run(work);

    expect(work).toHaveBeenCalledTimes(1);
    await second;
    release();
    await first;

    await gate.run(work);

    expect(work).toHaveBeenCalledTimes(2);
  });
});

describe("createDebouncedTask", () => {
  test("runs only the latest scheduled task", async () => {
    vi.useFakeTimers();
    const run = vi.fn();
    const task = createDebouncedTask(200);

    task.schedule(() => run("first"));
    task.schedule(() => run("second"));

    await vi.advanceTimersByTimeAsync(199);
    expect(run).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(run).toHaveBeenCalledTimes(1);
    expect(run).toHaveBeenCalledWith("second");

    vi.useRealTimers();
  });
});
