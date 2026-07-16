/**
 * System Context Diagram Renderer
 * 
 * Renders System Need Analysis diagrams showing:
 * - Central System box (under study)
 * - Surrounding Actors with interactions
 * - System boundary visualization
 * - Actor-to-System exchanges
 * - Professional Capella-style design
 */

import {
  SystemAnalysis,
  ExternalActor,
  SystemComponent,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  SvgElement,
  RenderConfig,
  DiagramOutput,
  CAPELLA_COLORS,
} from '../types/diagram';
import {
  createSvgDocument,
  createRect,
  createCircle,
  createLine,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createRoundedRect,
} from '../utils/svg';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

const SYSTEM_BOX_SIZE = { width: 400, height: 300 };
const ACTOR_SIZE = { width: 140, height: 100 };
const RADIUS = 350; // Distance from center for actors

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render system context diagram
 */
export async function renderSystemContext(
  sa: SystemAnalysis,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // Calculate layout
  const layout = calculateCircularLayout(sa);

  // Render to SVG
  const svg = renderToSvg(sa, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'system-context',
      systemName: sa.name,
      actorCount: sa.external_actors?.length || 0,
    },
  };
}

// ============================================================================
// Layout Calculation
// ============================================================================

interface SystemContextLayout {
  systemBox: { x: number; y: number; width: number; height: number };
  actors: Array<{
    actor: ExternalActor;
    x: number;
    y: number;
    width: number;
    height: number;
    angle: number;
  }>;
  exchanges: Array<{
    from: Point;
    to: Point;
    label: string;
  }>;
  totalSize: { width: number; height: number };
}

function calculateCircularLayout(sa: SystemAnalysis): SystemContextLayout {
  const actors = sa.external_actors || [];
  const centerX = RADIUS + 100;
  const centerY = RADIUS + 100;

  // System box in center
  const systemBox = {
    x: centerX - SYSTEM_BOX_SIZE.width / 2,
    y: centerY - SYSTEM_BOX_SIZE.height / 2,
    width: SYSTEM_BOX_SIZE.width,
    height: SYSTEM_BOX_SIZE.height,
  };

  // Position actors in circle around system
  const actorNodes = actors.map((actor, i) => {
    const angle = (i * 2 * Math.PI) / actors.length - Math.PI / 2;
    const x = centerX + RADIUS * Math.cos(angle) - ACTOR_SIZE.width / 2;
    const y = centerY + RADIUS * Math.sin(angle) - ACTOR_SIZE.height / 2;

    return {
      actor,
      x,
      y,
      width: ACTOR_SIZE.width,
      height: ACTOR_SIZE.height,
      angle,
    };
  });

  // Create exchanges from actors to system
  const exchanges = actorNodes.map((actorNode) => {
    const actorCenterX = actorNode.x + actorNode.width / 2;
    const actorCenterY = actorNode.y + actorNode.height / 2;

    // Find closest point on system box
    const systemCenterX = systemBox.x + systemBox.width / 2;
    const systemCenterY = systemBox.y + systemBox.height / 2;

    // Calculate intersection with system box edges
    const dx = systemCenterX - actorCenterX;
    const dy = systemCenterY - actorCenterY;
    const angle = Math.atan2(dy, dx);

    let toX: number, toY: number;

    // Determine which edge of the system box to connect to
    if (Math.abs(Math.cos(angle)) > Math.abs(Math.sin(angle))) {
      // Left or right edge
      toX = Math.cos(angle) > 0 ? systemBox.x : systemBox.x + systemBox.width;
      toY = systemCenterY + (toX - systemCenterX) * Math.tan(angle);
    } else {
      // Top or bottom edge
      toY = Math.sin(angle) > 0 ? systemBox.y : systemBox.y + systemBox.height;
      toX = systemCenterX + (toY - systemCenterY) / Math.tan(angle);
    }

    return {
      from: { x: actorCenterX, y: actorCenterY },
      to: { x: toX, y: toY },
      label: `interacts with ${actorNode.actor.name}`,
    };
  });

  return {
    systemBox,
    actors: actorNodes,
    exchanges,
    totalSize: {
      width: (RADIUS + 100) * 2,
      height: (RADIUS + 100) * 2,
    },
  };
}

// ============================================================================
// SVG Rendering
// ============================================================================

function renderToSvg(
  sa: SystemAnalysis,
  layout: SystemContextLayout,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  // Create arrow markers
  defs.push(createArrowMarker('arrow-exchange', '#607D8B', 10));

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Render exchanges first (behind everything)
  for (const exchange of layout.exchanges) {
    elements.push(renderExchange(exchange, config));
  }

  // Render system box with boundary
  elements.push(renderSystemBox(sa, layout.systemBox, config));

  // Render actors
  for (const actorNode of layout.actors) {
    elements.push(renderActor(actorNode, config));
  }

  // Title
  elements.push(
    createText(20, 35, `${sa.name} - System Context`, {
      'font-family': 'Arial, sans-serif',
      'font-size': 18,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

function renderSystemBox(
  sa: SystemAnalysis,
  box: { x: number; y: number; width: number; height: number },
  config: RenderConfig
): SvgElement {
  const elements: SvgElement[] = [];

  // CRITICAL: Prominent system boundary per LaTeX spec Section 4 (SAB Layout)
  // Must be visually distinct and clearly centered
  
  // Outer boundary highlight (system boundary) - MORE PROMINENT
  elements.push(
    createRoundedRect(box.x - 40, box.y - 40, box.width + 80, box.height + 80, 20, {
      fill: 'rgba(25, 118, 210, 0.08)',
      stroke: '#1976D2',
      'stroke-width': 6,
      'stroke-dasharray': '20,10',
      'opacity': '0.9',
    })
  );

  // Inner boundary emphasis
  elements.push(
    createRoundedRect(box.x - 20, box.y - 20, box.width + 40, box.height + 40, 16, {
      fill: 'none',
      stroke: '#1976D2',
      'stroke-width': 3,
      'opacity': '0.6',
    })
  );

  // System box with enhanced shadow
  elements.push(
    createRoundedRect(box.x, box.y, box.width, box.height, 12, {
      fill: '#E3F2FD',
      stroke: '#1976D2',
      'stroke-width': 5,
      'filter': 'drop-shadow(0 6px 12px rgba(25,118,210,0.3))',
    })
  );

  // «system» stereotype
  elements.push(
    createText(box.x + box.width / 2, box.y + 30, '«system»', {
      'text-anchor': 'middle',
      'dominant-baseline': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 14,
      'font-style': 'italic',
      fill: '#1976D2',
    })
  );

  // System name
  const systemName = sa.name || 'System Under Study';
  elements.push(
    createText(box.x + box.width / 2, box.y + 60, systemName, {
      'text-anchor': 'middle',
      'dominant-baseline': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 24,
      'font-weight': 'bold',
      fill: '#1976D2',
    })
  );

  // System boundary label - PROMINENT
  elements.push(
    createText(box.x - 35, box.y - 50, 'SYSTEM BOUNDARY', {
      'font-family': 'Arial, sans-serif',
      'font-size': 14,
      'font-weight': 'bold',
      'font-style': 'italic',
      fill: '#1976D2',
    })
  );

  // Functions count (if available)
  const functionCount = sa.functions?.length || 0;
  if (functionCount > 0) {
    elements.push(
      createText(box.x + box.width / 2, box.y + 100, `${functionCount} Functions`, {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        fill: '#666666',
      })
    );
  }

  return createGroup(elements, { id: 'system-box' });
}

function renderActor(
  actorNode: {
    actor: ExternalActor;
    x: number;
    y: number;
    width: number;
    height: number;
  },
  config: RenderConfig
): SvgElement {
  const elements: SvgElement[] = [];

  // Actor box
  elements.push(
    createRoundedRect(
      actorNode.x,
      actorNode.y,
      actorNode.width,
      actorNode.height,
      10,
      {
        fill: '#FFF9C4',
        stroke: '#F57C00',
        'stroke-width': 3,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // «actor» stereotype
  elements.push(
    createText(
      actorNode.x + actorNode.width / 2,
      actorNode.y + 20,
      '«actor»',
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        'font-style': 'italic',
        fill: '#F57C00',
      }
    )
  );

  // Actor name (with line wrapping)
  const name = actorNode.actor.name;
  const lines = name.split(' ');
  const lineHeight = 16;
  const startY = actorNode.y + 45;

  if (lines.length === 1) {
    elements.push(
      createText(
        actorNode.x + actorNode.width / 2,
        startY,
        name,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 13,
          'font-weight': 'bold',
          fill: '#F57C00',
        }
      )
    );
  } else {
    lines.forEach((line, i) => {
      elements.push(
        createText(
          actorNode.x + actorNode.width / 2,
          startY + i * lineHeight,
          line,
          {
            'text-anchor': 'middle',
            'dominant-baseline': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 13,
            'font-weight': 'bold',
            fill: '#F57C00',
          }
        )
      );
    });
  }

  return createGroup(elements, { id: `actor-${actorNode.actor.id}` });
}

function renderExchange(
  exchange: { from: Point; to: Point; label: string },
  config: RenderConfig
): SvgElement {
  const elements: SvgElement[] = [];

  // Exchange line
  elements.push(
    createLine(exchange.from.x, exchange.from.y, exchange.to.x, exchange.to.y, {
      stroke: '#607D8B',
      'stroke-width': 2.5,
      'marker-end': 'url(#arrow-exchange)',
      'stroke-dasharray': '5,5',
    })
  );

  // Label (optional, can be hidden for cleaner look)
  if (config.showLabels) {
    const midX = (exchange.from.x + exchange.to.x) / 2;
    const midY = (exchange.from.y + exchange.to.y) / 2;

    elements.push(
      createRect(midX - 60, midY - 10, 120, 20, {
        fill: '#FFFFFF',
        stroke: '#607D8B',
        'stroke-width': 1,
        rx: 4,
        ry: 4,
        'filter': 'drop-shadow(0 1px 2px rgba(0,0,0,0.1))',
      })
    );

    elements.push(
      createText(midX, midY, 'interacts', {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        'font-style': 'italic',
        fill: '#607D8B',
      })
    );
  }

  return createGroup(elements);
}
