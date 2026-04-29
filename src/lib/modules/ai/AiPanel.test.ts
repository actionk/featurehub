import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, render, waitFor } from '@testing-library/svelte';
import AiPanel from './AiPanel.svelte';
import type { TabContext } from '../registry';
import { ptyResumeSession } from '../../api/tauri';

// Mock Chart.js so canvas effects don't blow up in jsdom
vi.mock('chart.js', () => {
  class ChartMock {
    data = { labels: [] as string[], datasets: [{ data: [] as number[] }] };
    update() {}
    destroy() {}
    static register() {}
  }
  return { Chart: ChartMock, registerables: [] };
});

// Mock canvas getContext so the $effect using ctx.createLinearGradient doesn't throw
const mockGradient = { addColorStop: vi.fn() };
const mockCtx2d = {
  createLinearGradient: vi.fn().mockReturnValue(mockGradient),
  createRadialGradient: vi.fn().mockReturnValue(mockGradient),
  createConicGradient: vi.fn().mockReturnValue(mockGradient),
  createPattern: vi.fn().mockReturnValue(null),
  clearRect: vi.fn(), fillRect: vi.fn(), strokeRect: vi.fn(),
  beginPath: vi.fn(), closePath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(),
  arc: vi.fn(), arcTo: vi.fn(), ellipse: vi.fn(), rect: vi.fn(),
  bezierCurveTo: vi.fn(), quadraticCurveTo: vi.fn(),
  fill: vi.fn(), stroke: vi.fn(), clip: vi.fn(),
  save: vi.fn(), restore: vi.fn(),
  translate: vi.fn(), scale: vi.fn(), rotate: vi.fn(),
  setTransform: vi.fn(), resetTransform: vi.fn(),
  getTransform: vi.fn().mockReturnValue({ a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 }),
  drawImage: vi.fn(),
  fillText: vi.fn(), strokeText: vi.fn(),
  measureText: vi.fn().mockReturnValue({ width: 0, actualBoundingBoxAscent: 0, actualBoundingBoxDescent: 0, fontBoundingBoxAscent: 0, fontBoundingBoxDescent: 0, actualBoundingBoxLeft: 0, actualBoundingBoxRight: 0 }),
  getImageData: vi.fn().mockReturnValue({ data: new Uint8ClampedArray(4), width: 1, height: 1, colorSpace: 'srgb' }),
  putImageData: vi.fn(), createImageData: vi.fn(),
  setLineDash: vi.fn(), getLineDash: vi.fn().mockReturnValue([]),
  isPointInPath: vi.fn().mockReturnValue(false), isPointInStroke: vi.fn().mockReturnValue(false),
  canvas: document.createElement('canvas'),
};
HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue(mockCtx2d) as typeof HTMLCanvasElement.prototype.getContext;

// Mock all Tauri IPC
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));
vi.mock('../../stores/settings.svelte', () => ({
  getCachedSettings: vi.fn().mockResolvedValue({ mcp_servers: [], skills: [], extensions: [] }),
}));
vi.mock('../../stores/sessionActivity.svelte', () => ({
  getPanelSessions: vi.fn().mockReturnValue([]),
  isSessionActive: vi.fn().mockReturnValue(false),
}));
vi.mock('../../stores/terminals.svelte', () => ({
  getTerminalsForFeature: vi.fn().mockReturnValue([]),
  getActiveTerminals: vi.fn().mockReturnValue([]),
  addTerminal: vi.fn(),
  removeTerminal: vi.fn(),
  markExited: vi.fn(),
  getPendingViewRequest: vi.fn().mockReturnValue({ version: -1, terminalId: null }),
  getPendingResumeRequest: vi.fn().mockReturnValue({ version: -1, sessionDbId: null }),
  setViewingTerminal: vi.fn(),
}));
vi.mock('../../stores/tabToolbar.svelte', () => ({
  setToolbarActions: vi.fn(),
  clearToolbarActions: vi.fn(),
}));
vi.mock('../../api/tauri', () => ({
  ptySpawnSession: vi.fn(),
  ptyResumeSession: vi.fn(),
  ptyKill: vi.fn(),
  finishEmbeddedSession: vi.fn(),
  detectIdes: vi.fn().mockResolvedValue([]),
  openInIde: vi.fn(),
  getFhCliPath: vi.fn().mockResolvedValue('fh'),
  scanSessions: vi.fn(),
  getFeatureMcpServers: vi.fn().mockResolvedValue([]),
  setFeatureMcpServer: vi.fn(),
  getFeatureSkills: vi.fn().mockResolvedValue([]),
  setFeatureSkill: vi.fn(),
}));

function makeContext(overrides: Partial<TabContext> = {}): TabContext {
  return {
    featureId: 'feat-1',
    feature: {
      id: 'feat-1',
      title: 'Test Feature',
      description: null,
      ticket_id: null,
      status: 'active',
      pinned: false,
      archived: false,
      parent_id: null,
      group_id: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      sort_order: 0,
      directories: [],
      links: [],
      tags: [],
    },
    sessions: [],
    plans: [],
    tasks: [],
    note: null,
    allTags: [],
    activeSessionCount: 0,
    pendingPlanId: null,
    onPendingPlanHandled: vi.fn(),
    onSessionsChanged: vi.fn(),
    onRefresh: vi.fn(),
    ...overrides,
  };
}

describe('AiPanel sessions card', () => {
  beforeEach(() => {
    vi.mocked(ptyResumeSession).mockResolvedValue({
      terminalId: 'term-1',
      sessionDbId: 's1',
      claudeSessionId: 'claude-1',
    });
  });

  it('shows Start Session CTA when no active sessions', () => {
    const { container } = render(AiPanel, { props: makeContext({ sessions: [] }) });
    expect(container.querySelector('.sc-start-cta')).toBeTruthy();
    expect(container.querySelector('.sc-start-btn')).toBeTruthy();
    expect(container.querySelector('.sc-active')).toBeFalsy();
  });

  it('shows expanded active card when one session is running', () => {
    const ctx = makeContext({
      sessions: [{
        id: 's1',
        feature_id: 'feat-1',
        claude_session_id: null,
        title: 'Working on auth',
        summary: null,
        started_at: new Date().toISOString(),
        ended_at: null,
        duration_mins: null,
        project_path: null,
        branch: null,
        turns: null,
      }],
    });
    const { container } = render(AiPanel, { props: ctx });
    expect(container.querySelector('.sc-active')).toBeTruthy();
    expect(container.querySelector('.sc-live-pill')).toBeTruthy();
    expect(container.querySelector('.sc-start-cta')).toBeFalsy();
  });

  it('shows compact row for second active session', () => {
    const now = new Date().toISOString();
    const ctx = makeContext({
      sessions: [
        { id: 's1', feature_id: 'feat-1', claude_session_id: null, title: 'First', summary: null, started_at: now, ended_at: null, duration_mins: null, project_path: null, branch: null, turns: null },
        { id: 's2', feature_id: 'feat-1', claude_session_id: null, title: 'Second', summary: null, started_at: now, ended_at: null, duration_mins: null, project_path: null, branch: null, turns: null },
      ],
    });
    const { container } = render(AiPanel, { props: ctx });
    expect(container.querySelector('.sc-active')).toBeTruthy();
    expect(container.querySelector('.sc-active-compact')).toBeTruthy();
  });

  it('opens the active session when the active row is clicked', async () => {
    const now = new Date().toISOString();
    const ctx = makeContext({
      sessions: [
        { id: 's1', feature_id: 'feat-1', claude_session_id: 'claude-1', title: 'Active work', summary: null, started_at: now, ended_at: null, duration_mins: null, project_path: null, branch: null, turns: null },
      ],
    });
    const { container } = render(AiPanel, { props: ctx });

    await fireEvent.click(container.querySelector('.sc-active')!);

    await waitFor(() => {
      expect(ptyResumeSession).toHaveBeenCalledWith('s1', 80, 24, false);
    });
  });

  it('can resume a past session with full access from its danger action', async () => {
    const ctx = makeContext({
      sessions: [
        { id: 's1', feature_id: 'feat-1', claude_session_id: 'claude-1', title: 'Past work', summary: null, started_at: new Date().toISOString(), ended_at: new Date().toISOString(), duration_mins: 5, project_path: null, branch: null, turns: null },
      ],
    });
    const { container } = render(AiPanel, { props: ctx });

    await fireEvent.click(container.querySelector('.sc-session-danger-open')!);

    await waitFor(() => {
      expect(ptyResumeSession).toHaveBeenCalledWith('s1', 80, 24, true);
    });
  });
});
