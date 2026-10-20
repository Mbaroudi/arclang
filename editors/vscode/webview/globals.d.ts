// What the page gets from VS Code and from the viewer's scripts.
import type { FromWebview } from '../src/pure/workbench';

export interface ViewerSelection {
  view: string;
  id: string;
  name: string;
  kind: string;
  isEdge: boolean;
}

export interface ViewerPosition {
  view: string | null;
  selected: string | null;
  position: unknown;
}

export interface ViewerApi {
  ready: Promise<unknown>;
  settled?: () => Promise<unknown> | undefined;
  show?: (view: string) => void;
  goTo?: (view: string, id?: string) => void;
  select?: (id: string | null) => void;
  fit?: () => void;
  current?: () => ViewerPosition;
  restore?: (position: unknown) => void;
  views?: () => { id: string; view: string; title: string }[];
  svgOf: (id: string) => SVGSVGElement | null;
  /** Several boxes moved at once, in sheet units: one redraw, one layout change. */
  moveMany?: (moves: { id: string; dx: number; dy: number }[]) => Promise<void>;
  scale?: () => number;
  routeOf?: (edge: string) => { x: number; y: number }[];
  setRoute?: (edge: string, points: { x: number; y: number }[]) => Promise<void>;
  /** Read the host's history and selection into the toolbar again. */
  refreshTools?: () => void;
}

export interface ToolbarHookItem {
  id: string;
  label: string;
  title?: string;
  /** As 'Mod+Shift+C': written ⇧⌘C on a Mac, Ctrl+Shift+C elsewhere. */
  shortcut?: string;
  /** A theme item: ticked when it is the one in use. */
  checked?: boolean;
  run: () => void;
}

export interface ViewerOptions {
  slots?: Record<string, HTMLElement>;
  onSelect?: (selection: ViewerSelection | null) => void;
  onDrop?: (drop: { view: string; from: string; to: string | null }) => void;
  onOpen?: (id: string) => void;
  onFullscreen?: () => void;
  onConnect?: (fromId: string, toId: string, ends?: { fromKind?: string; toKind?: string; view?: string }) => void;
  connectTargets?: (fromId: string) => string[] | null;
  onDownload?: (blob: Blob, filename: string) => void;
  arrange?: boolean;
  onLayout?: (file: unknown) => void;
  /** Editing from the drawing (ArcVizEdit): see the viewer's `mount`. */
  edit?: Record<string, unknown>;
  /** What the toolbar adds for this host: undo and redo, align, the arrange switch. */
  toolbar?: {
    history?: { undo: () => void; redo: () => void; labels: () => { undo: string | null; redo: string | null } };
    align?: { count: () => number; align: (how: string) => void; distribute: (axis: 'horizontal' | 'vertical') => void };
    arrangeToggle?: boolean;
  };
  /** More entries in Export, above SVG and PNG; a Theme menu when not empty. */
  exportItems?: ToolbarHookItem[] | (() => ToolbarHookItem[]);
  themeItems?: ToolbarHookItem[] | (() => ToolbarHookItem[]);
}

declare global {
  interface Window {
    ArcViz: { mount: (host: HTMLElement, payload: unknown, deps: Record<string, unknown>, options?: ViewerOptions) => ViewerApi };
    ELK: unknown;
    ArcVizRender: unknown;
    ArcVizLinks: unknown;
    ArcVizNavigation: unknown;
    ArcVizCollapse: unknown;
    ArcVizScope: unknown;
    ArcVizExport: unknown;
    ArcVizStage: unknown;
    ArcVizPlace: unknown;
    ArcVizEdit: unknown;
    ArcVizToolbar: {
      icon: (name: string, size?: number) => SVGSVGElement;
      isMac: () => boolean;
      tooltip: (label: string, keys: string, mac: boolean) => string;
      shortcutLabel: (keys: string, mac: boolean) => string;
    };
  }
  function acquireVsCodeApi(): { postMessage: (message: FromWebview) => void; getState: () => unknown; setState: (state: unknown) => void };
}
