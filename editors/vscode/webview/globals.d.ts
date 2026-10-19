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
  }
  function acquireVsCodeApi(): { postMessage: (message: FromWebview) => void; getState: () => unknown; setState: (state: unknown) => void };
}
