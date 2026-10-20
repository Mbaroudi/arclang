/*
 * ArcViz renderer — draws a laid-out ELK graph (see `diagram::elk`) as an
 * SVG string in Arcadia notation.
 *
 * Pure: no DOM, no globals, same output in a browser and under Node, so the
 * drawing can be checked headless (tools/diagram_render). Every dynamic
 * string goes through `esc` before it reaches the markup.
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.ArcVizRender = factory();
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  // Arial metrics are what the Rust size estimates assume; keep in step
  // with CHAR_WIDTH in diagram/elk.rs.
  const FONT = 'Arial, Helvetica, sans-serif';
  const MARGIN = 14;
  // Boxes listing lines (class fields, enumeration values); keep in step
  // with the COMPARTMENT_* constants in diagram/elk.rs.
  const COMPARTMENT = { top: 34, line: 14, inset: 10 };
  const CORNER = 5;

  // Capella's colour code: blue structure, green functions, yellow hardware.
  const NODE_STYLE = {
    operational_actor: { fill: '#DCEAF8', stroke: '#3F6FA6', label: 'Operational actor', glyph: 'actor' },
    operational_entity: { fill: '#EEF0F2', stroke: '#66717D', label: 'Operational entity', glyph: 'box' },
    operational_activity: { fill: '#FFE3AE', stroke: '#B0730C', label: 'Operational activity', glyph: 'function', round: 9 },
    operational_process: { fill: '#FFEFCF', stroke: '#B0730C', label: 'Operational process', glyph: 'chain', round: 9 },
    system: { fill: '#E3EEFB', stroke: '#2A62A8', label: 'System', glyph: 'component' },
    system_actor: { fill: '#DCEAF8', stroke: '#3F6FA6', label: 'Actor', glyph: 'actor' },
    system_component: { fill: '#D3E4F8', stroke: '#2A62A8', label: 'System component', glyph: 'component' },
    function: { fill: '#CDEDBE', stroke: '#3C8A3F', label: 'Function', glyph: 'function', round: 9 },
    logical_component: { fill: '#D3E4F8', stroke: '#2A62A8', label: 'Logical component', glyph: 'component' },
    physical_node: { fill: '#FFF4AE', stroke: '#9C8500', label: 'Node', glyph: 'node' },
    behavior_component: { fill: '#D3E4F8', stroke: '#2A62A8', label: 'Behaviour component', glyph: 'component' },
    hardware_component: { fill: '#FFE98A', stroke: '#9C8500', label: 'Hardware component', glyph: 'node' },
    deployed_component: { fill: '#D3E4F8', stroke: '#2A62A8', label: 'Deployed component', glyph: 'component' },
    mode: { fill: '#E3EEFB', stroke: '#2A62A8', label: 'Mode', glyph: 'none', round: 14 },
    state: { fill: '#EEF0F2', stroke: '#4A545E', label: 'State', glyph: 'none', round: 14 },
    initial_state: { fill: '#18222D', stroke: '#18222D', label: 'Initial state', glyph: 'none' },
    junction: { fill: '#1F4E8C', stroke: '#1F4E8C', label: 'Junction', glyph: 'none' },
    lifeline: { fill: '#EEF0F2', stroke: '#66717D', label: 'Participant', glyph: 'box' },
    mission: { fill: '#FFE9B8', stroke: '#9A6400', label: 'Mission', glyph: 'mission', round: 16 },
    operational_capability: { fill: '#F3E8D3', stroke: '#8A6A2E', label: 'Operational capability', glyph: 'capability', round: 16 },
    capability: { fill: '#E3EEFB', stroke: '#2A62A8', label: 'Capability', glyph: 'capability', round: 16 },
    capability_realization: { fill: '#D3E4F8', stroke: '#2A62A8', label: 'Capability realization', glyph: 'capability', round: 16 },
    functional_chain: { fill: '#FCE3D4', stroke: '#B4500F', label: 'Functional chain', glyph: 'chain', round: 9 },
    class: { fill: '#F7F1DC', stroke: '#7A6A2A', label: 'Class', glyph: 'none' },
    enumeration: { fill: '#EEF0F2', stroke: '#4A545E', label: 'Enumeration', glyph: 'none' },
    data_type: { fill: '#EEF0F2', stroke: '#4A545E', label: 'Data type', glyph: 'none' },
    exchange_item: { fill: '#E4F3E0', stroke: '#3C8A3F', label: 'Exchange item', glyph: 'none' },
    configuration_item: { fill: '#E9ECEF', stroke: '#3D4852', label: 'Configuration item', glyph: 'node' },
  };
  const FALLBACK_NODE = { fill: '#FFFFFF', stroke: '#444B52', label: 'Element', glyph: 'box' };

  const EDGE_STYLE = {
    interaction: { stroke: '#7A5200', width: 1.4, label: 'Interaction', arrow: true },
    communication_mean: { stroke: '#5A6672', width: 1.4, dash: '5 3', label: 'Communication mean', arrow: false },
    functional_exchange: { stroke: '#1F7A33', width: 1.5, label: 'Functional exchange', arrow: true },
    component_exchange: { stroke: '#1D4E96', width: 1.7, label: 'Component exchange', arrow: true },
    physical_link: { stroke: '#C92A2A', width: 2.8, label: 'Physical link', arrow: false },
    physical_exchange: { stroke: '#1D4E96', width: 1.5, dash: '6 3', label: 'Physical exchange', arrow: true },
    transition: { stroke: '#343A40', width: 1.4, label: 'Transition', arrow: true },
    message: { stroke: '#18222D', width: 1.4, label: 'Message', arrow: true },
    exploitation: { stroke: '#9A6400', width: 1.5, label: 'Mission exploits capability', arrow: true },
    realization: { stroke: '#2A62A8', width: 1.4, dash: '6 3', label: 'Realizes', arrow: true },
    involvement: { stroke: '#55616D', width: 1.2, label: 'Involves', arrow: true },
    capability_association: { stroke: '#8A6A2E', width: 1.3, dash: '4 3', label: 'Includes or extends', arrow: true },
    generalization: { stroke: '#4A545E', width: 1.3, label: 'Specializes', arrow: true, hollow: true },
    association: { stroke: '#7A6A2A', width: 1.3, label: 'Typed by', arrow: true },
    item_element: { stroke: '#3C8A3F', width: 1.3, dash: '5 3', label: 'Groups', arrow: true },
    breakdown: { stroke: '#3D4852', width: 1.4, label: 'Made of', arrow: false },
  };
  const FALLBACK_EDGE = { stroke: '#444B52', width: 1.4, label: 'Link', arrow: true };

  const SAFETY_COLOR = [
    [/ASIL[-_ ]?D|SIL[-_ ]?4|DAL[-_ ]?A/i, '#B42318'],
    [/ASIL[-_ ]?C|SIL[-_ ]?3|DAL[-_ ]?B/i, '#C4500A'],
    [/ASIL[-_ ]?B|SIL[-_ ]?2|DAL[-_ ]?C/i, '#A66A00'],
    [/ASIL[-_ ]?A|SIL[-_ ]?1|DAL[-_ ]?D/i, '#5C7A12'],
  ];

  function esc(value) {
    return String(value == null ? '' : value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }

  function num(value) {
    return Math.round(value * 100) / 100;
  }

  function nodeStyle(kind) {
    return NODE_STYLE[kind] || FALLBACK_NODE;
  }

  function edgeStyle(kind) {
    return EDGE_STYLE[kind] || FALLBACK_EDGE;
  }

  function safetyColor(level) {
    const hit = SAFETY_COLOR.find(([pattern]) => pattern.test(level));
    return hit ? hit[1] : '#5A6672';
  }

  function safetyOf(node) {
    const properties = (node.arc && node.arc.properties) || {};
    return properties.safety_level || properties.asil || null;
  }

  /** Small kind marker in a node's top-left corner (12x12 box at x,y). */
  function glyph(kind, x, y, stroke) {
    const s = `fill="none" stroke="${stroke}" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round"`;
    switch (kind) {
      case 'actor':
        return `<g ${s}><circle cx="${x + 6}" cy="${y + 2.5}" r="2"/><path d="M${x + 6} ${y + 4.5}v4m-3.5-2.5h7m-3.5 2.5l-3 3.5m3-3.5l3 3.5"/></g>`;
      case 'function':
        return `<g ${s}><circle cx="${x + 6}" cy="${y + 6}" r="5.2"/><path d="M${x + 7.6} ${y + 3.4}c-1.6-.4-2.1.5-2.1 1.6v4.4m-1.4-3h3"/></g>`;
      case 'node':
        return `<g ${s}><path d="M${x + 1} ${y + 4}h7.5v7H${x + 1}zm0 0l2.5-2.5h7.5L${x + 8.5} ${y + 4}m2.5-2.5v7l-2.5 2.5"/></g>`;
      case 'none':
        return '';
      case 'capability':
        return `<g ${s}><ellipse cx="${x + 6}" cy="${y + 6}" rx="5.4" ry="3.6"/></g>`;
      case 'mission':
        return `<g ${s}><path d="M${x + 2.5} ${y + 11.5}v-10m0 .8h7l-2 2.6 2 2.6h-7"/></g>`;
      case 'chain':
        return `<g ${s}><circle cx="${x + 2.5}" cy="${y + 6}" r="1.8"/><circle cx="${x + 9.5}" cy="${y + 6}" r="1.8"/><path d="M${x + 4.3} ${y + 6}h3.4"/></g>`;
      case 'component':
        return `<g ${s}><rect x="${x + 3}" y="${y + 1}" width="8.5" height="10" rx="1"/><rect x="${x + 0.8}" y="${y + 3}" width="4.4" height="2" fill="#fff"/><rect x="${x + 0.8}" y="${y + 7}" width="4.4" height="2" fill="#fff"/></g>`;
      default:
        return `<g ${s}><rect x="${x + 1.5}" y="${y + 2}" width="9" height="8" rx="1"/></g>`;
    }
  }

  function portSide(port, node) {
    if (port.x <= 0) return 'W';
    if (port.x + port.width >= node.width) return 'E';
    if (port.y <= 0) return 'N';
    return 'S';
  }

  /** Triangle inside a component port showing which way data flows. */
  function flowArrow(x, y, size, side, direction) {
    const inward = { W: 'right', E: 'left', N: 'down', S: 'up' }[side];
    const outward = { W: 'left', E: 'right', N: 'up', S: 'down' }[side];
    const heading = direction === 'in' ? inward : outward;
    const c = size / 2;
    const r = size * 0.28;
    const shapes = {
      right: [[c - r, c - r], [c + r, c], [c - r, c + r]],
      left: [[c + r, c - r], [c - r, c], [c + r, c + r]],
      down: [[c - r, c - r], [c, c + r], [c + r, c - r]],
      up: [[c - r, c + r], [c, c - r], [c + r, c + r]],
    };
    const points = shapes[heading].map(([px, py]) => `${num(x + px)},${num(y + py)}`).join(' ');
    return `<polygon points="${points}" fill="#1D4E96"/>`;
  }

  function drawPort(port, node, nx, ny) {
    const arc = port.arc || {};
    const x = nx + port.x;
    const y = ny + port.y;
    const onFunction = node.arc.kind === 'function';
    const dash = arc.synthesized ? ' stroke-dasharray="2 1.5"' : '';
    const tip = [
      port.labels && port.labels[0] ? port.labels[0].text : arc.name,
      arc.interface ? `carries ${arc.interface}` : null,
      arc.protocol ? `over ${arc.protocol}` : null,
      arc.synthesized ? 'derived from an exchange, not declared' : null,
    ].filter(Boolean).join(' — ');

    let body;
    if (onFunction) {
      const fill = { in: '#2F9E44', out: '#E8590C' }[arc.direction] || '#868E96';
      body = `<rect x="${num(x)}" y="${num(y)}" width="${port.width}" height="${port.height}" fill="${fill}" stroke="#1B1F23" stroke-width=".8"${dash}/>`;
    } else if (arc.direction === 'undirected') {
      body = `<rect x="${num(x)}" y="${num(y)}" width="${port.width}" height="${port.height}" fill="#FFE066" stroke="#7A6800" stroke-width="1.1"/>`;
    } else {
      body = `<rect x="${num(x)}" y="${num(y)}" width="${port.width}" height="${port.height}" fill="#FFFFFF" stroke="#1D4E96" stroke-width="1.2"${dash}/>`;
      if (arc.direction === 'in' || arc.direction === 'out') {
        body += flowArrow(x, y, port.width, portSide(port, node), arc.direction);
      }
    }
    return `<g class="av-port" data-id="${esc(port.id)}"><title>${esc(tip)}</title>${body}</g>`;
  }

  function drawNode(node, ox, oy, out, origins) {
    const arc = node.arc || {};
    const style = nodeStyle(arc.kind);
    const x = ox + node.x;
    const y = oy + node.y;
    origins.set(node.id, { x, y });
    const container = Array.isArray(node.children) && node.children.length > 0;
    const name = node.labels && node.labels[0] ? node.labels[0].text : node.id;
    const description = arc.properties && arc.properties.description;
    const realizes = (arc.realizes || []).join(', ');
    const safety = safetyOf(node);

    // A notation mark: a dot where a machine starts or an exchange meets.
    if (arc.kind === 'initial_state' || arc.kind === 'junction') {
      const r = node.width / 2;
      out.push(`<g class="av-node" data-id="${esc(node.id)}" data-kind="${arc.kind}"><title>${esc(style.label)}${arc.kind === 'junction' ? ': ' + esc(name) : ''}</title><circle class="av-shape" cx="${num(x + r)}" cy="${num(y + r)}" r="${num(r)}" fill="${style.fill}" stroke="${style.stroke}"/></g>`);
      return;
    }
    let body = `<title>${esc(style.label)}: ${esc(name)}${description ? ' — ' + esc(description) : ''}</title>`;
    // A folded container reads as a stack: a second sheet shows behind it.
    if (arc.folded) {
      body += `<rect x="${num(x + 4)}" y="${num(y + 4)}" width="${num(node.width)}" height="${num(node.height)}" rx="${style.round || 3}" fill="${style.fill}" stroke="${style.stroke}" stroke-width="1"/>`;
    }
    // A variant of a product line is one choice among others: dashed.
    const properties = arc.properties || {};
    const variant = properties.variant === 'true';
    const dashed = arc.context || variant;
    body += `<rect class="av-shape" x="${num(x)}" y="${num(y)}" width="${num(node.width)}" height="${num(node.height)}" rx="${style.round || 3}" fill="${style.fill}" stroke="${style.stroke}" stroke-width="${container ? 1.6 : 1.2}"${dashed ? ' stroke-dasharray="5 3"' : ''}/>`;
    body += glyph(style.glyph, x + 6, y + 6, style.stroke);
    if (properties.variation === 'true' || variant) {
      body += `<text x="${num(x + node.width - 6)}" y="${num(y + 11)}" text-anchor="end" font-size="8.5" font-style="italic" fill="#3D4852">${variant ? 'variant' : 'variation'}</text>`;
    }

    const centerX = x + node.width / 2;
    const lines = arc.compartment || [];
    if (lines.length > 0) {
      body += `<text x="${num(centerX)}" y="${num(y + 19)}" text-anchor="middle" font-size="12" font-weight="700" fill="#18222D">${esc(name)}</text>`;
      body += `<line x1="${num(x)}" y1="${num(y + COMPARTMENT.top - 8)}" x2="${num(x + node.width)}" y2="${num(y + COMPARTMENT.top - 8)}" stroke="${style.stroke}" stroke-width="1"/>`;
      lines.forEach((line, index) => {
        body += `<text x="${num(x + COMPARTMENT.inset)}" y="${num(y + COMPARTMENT.top + 4 + index * COMPARTMENT.line)}" font-size="10.5" fill="#26313C">${esc(line)}</text>`;
      });
    } else if (container) {
      body += `<text x="${num(centerX)}" y="${num(y + 20)}" text-anchor="middle" font-size="12" font-weight="700" fill="#18222D">${esc(name)}</text>`;
      if (realizes) {
        body += `<text x="${num(centerX)}" y="${num(y + 33)}" text-anchor="middle" font-size="9.5" font-style="italic" fill="#3D4852">realizes ${esc(realizes)}</text>`;
      }
    } else {
      // Under the name: what the element realizes, and what a folded
      // container hides.
      const notes = [realizes ? `realizes ${realizes}` : '', arc.folded ? `${arc.folded} inside` : ''].filter(Boolean).join(' · ');
      const baseline = y + node.height / 2 + (notes ? -2 : 4);
      body += `<text x="${num(centerX)}" y="${num(baseline)}" text-anchor="middle" font-size="12" font-weight="600" fill="#18222D">${esc(name)}</text>`;
      if (notes) {
        body += `<text x="${num(centerX)}" y="${num(baseline + 13)}" text-anchor="middle" font-size="9.5" font-style="italic" fill="#3D4852">${esc(notes)}</text>`;
      }
    }
    if (safety) {
      const width = safety.length * 5.2 + 8;
      const bx = x + node.width - width - 4;
      const by = y + node.height - 15;
      body += `<rect x="${num(bx)}" y="${num(by)}" width="${num(width)}" height="11" rx="2" fill="#FFFFFF" stroke="${safetyColor(safety)}" stroke-width=".8"/>`;
      body += `<text x="${num(bx + width / 2)}" y="${num(by + 8.3)}" text-anchor="middle" font-size="8" font-weight="700" fill="${safetyColor(safety)}">${esc(safety)}</text>`;
    }
    for (const port of node.ports || []) body += drawPort(port, node, x, y);

    // A context element stands beside the subject of the diagram: dashed.
    out.push(`<g class="av-node${arc.context ? ' is-context' : ''}" data-id="${esc(node.id)}" data-kind="${esc(arc.kind)}">${body}</g>`);
    for (const child of node.children || []) drawNode(child, x, y, out, origins);
  }

  /** Orthogonal polyline with softened corners. */
  function roundedPath(points) {
    if (points.length < 2) return '';
    let d = `M${num(points[0].x)} ${num(points[0].y)}`;
    for (let i = 1; i < points.length - 1; i += 1) {
      const prev = points[i - 1];
      const here = points[i];
      const next = points[i + 1];
      const inLen = Math.hypot(here.x - prev.x, here.y - prev.y);
      const outLen = Math.hypot(next.x - here.x, next.y - here.y);
      const r = Math.min(CORNER, inLen / 2, outLen / 2);
      if (r < 0.5) {
        d += `L${num(here.x)} ${num(here.y)}`;
        continue;
      }
      const ax = here.x - ((here.x - prev.x) / inLen) * r;
      const ay = here.y - ((here.y - prev.y) / inLen) * r;
      const bx = here.x + ((next.x - here.x) / outLen) * r;
      const by = here.y + ((next.y - here.y) / outLen) * r;
      d += `L${num(ax)} ${num(ay)}Q${num(here.x)} ${num(here.y)} ${num(bx)} ${num(by)}`;
    }
    const last = points[points.length - 1];
    return `${d}L${num(last.x)} ${num(last.y)}`;
  }

  /** A smooth line whose corners pull it along (draw.io's curved style): each corner a control point. */
  function curvedPath(points) {
    if (points.length < 3) return roundedPath(points);
    let d = `M${num(points[0].x)} ${num(points[0].y)}`;
    for (let i = 1; i < points.length - 1; i += 1) {
      const here = points[i];
      const next = points[i + 1];
      const end = i === points.length - 2 ? next : { x: (here.x + next.x) / 2, y: (here.y + next.y) / 2 };
      d += `Q${num(here.x)} ${num(here.y)} ${num(end.x)} ${num(end.y)}`;
    }
    return d;
  }

  // ELK reports an edge's route relative to the node that contains it (the
  // lowest common ancestor of its ends), named in `edge.container`.
  function drawEdge(edge, origins, rootId) {
    // The root graph's id can equal a node's id; the root is always at 0,0.
    const origin = (edge.container !== rootId && origins.get(edge.container)) || { x: 0, y: 0 };
    const at = (point) => ({ x: point.x + origin.x, y: point.y + origin.y });
    const arc = edge.arc || {};
    const style = edgeStyle(arc.kind);
    const name = edge.labels && edge.labels[0] ? edge.labels[0].text : edge.id;
    const tip = `${style.label}: ${name}${arc.exchange_item ? ' — carries ' + arc.exchange_item : ''}`;
    let body = `<title>${esc(tip)}</title>`;
    for (const section of edge.sections || []) {
      const points = [section.startPoint, ...(section.bendPoints || []), section.endPoint].map(at);
      // A line the reader styled: curved through its corners, or straight from end to end.
      const d = edge.lineStyle === 'curved' ? curvedPath(points) : roundedPath(points);
      const dash = style.dash ? ` stroke-dasharray="${style.dash}"` : '';
      // A leg of an exchange of more than two ends has no direction.
      const directed = style.arrow && !(arc.properties && arc.properties.undirected === 'true');
      const marker = directed ? ` marker-end="url(#av-arrow-${esc(arc.kind)})"` : '';
      body += `<path class="av-hit" d="${d}" fill="none" stroke="#000" stroke-opacity="0" stroke-width="10"/>`;
      body += `<path class="av-line" d="${d}" fill="none" stroke="${style.stroke}" stroke-width="${style.width}" stroke-linejoin="round"${dash}${marker}/>`;
    }
    for (const label of edge.labels || []) {
      if (label.x === undefined || !label.text) continue;
      body += `<text class="av-edge-label" x="${num(label.x + origin.x)}" y="${num(label.y + origin.y + 9.5)}" font-size="10" fill="${style.stroke}" stroke="#FFFFFF" stroke-width="3" paint-order="stroke" stroke-linejoin="round">${esc(label.text)}</text>`;
    }
    const styled = edge.lineStyle === 'curved' || edge.lineStyle === 'straight' ? ` data-style="${edge.lineStyle}"` : '';
    return `<g class="av-edge" data-id="${esc(edge.id)}"${styled} data-kind="${esc(arc.kind)}" data-source="${esc(arc.source_node)}" data-target="${esc(arc.target_node)}">${body}</g>`;
  }

  function arrowMarkers(edges) {
    const kinds = [...new Set(edges.map((edge) => (edge.arc || {}).kind))].filter((kind) => edgeStyle(kind).arrow);
    return kinds
      .map((kind) => {
        const style = edgeStyle(kind);
        // UML generalization: a hollow triangle on the general element.
        const head = style.hollow
          ? `<path d="M1 1L10 5L1 9z" fill="#FFFFFF" stroke="${style.stroke}" stroke-width="1.1" stroke-linejoin="round"/>`
          : `<path d="M0 1L10 5L0 9z" fill="${style.stroke}"/>`;
        const size = style.hollow ? 12 : 9;
        return `<marker id="av-arrow-${esc(kind)}" viewBox="0 0 11 10" refX="${style.hollow ? 10 : 9}" refY="5" markerUnits="userSpaceOnUse" markerWidth="${size}" markerHeight="${size}" orient="auto">${head}</marker>`;
      })
      .join('');
  }

  // --- sequence diagrams ---------------------------------------------------
  // A scenario needs no layout engine: lifelines are columns in declared
  // order, messages are rows in declared order (time runs down).
  const SEQ = { head: 40, first: 78, row: 40, minSpacing: 170, selfReach: 34, selfDrop: 16, tail: 26 };

  function messageText(edge, index) {
    const timing = edge.arc.properties && edge.arc.properties.timing;
    const label = edge.labels && edge.labels[0] ? edge.labels[0].text : '';
    return `${index + 1}: ${label}${timing ? ` {${timing}}` : ''}`;
  }

  function renderSequence(graph) {
    const lifelines = graph.children || [];
    const messages = graph.edges || [];
    const column = new Map(lifelines.map((lifeline, index) => [lifeline.id, index]));
    const textWidth = (text) => text.length * 5.6;
    const headWidth = Math.max(112, ...lifelines.map((lifeline) => lifeline.width || 112));

    let spacing = Math.max(SEQ.minSpacing, headWidth + 36);
    let selfRoom = 0;
    messages.forEach((message, index) => {
      const from = column.get(message.arc.source_node);
      const to = column.get(message.arc.target_node);
      const width = textWidth(messageText(message, index));
      if (from === to) {
        if (from === lifelines.length - 1) selfRoom = Math.max(selfRoom, SEQ.selfReach + 10 + width - headWidth / 2);
        else spacing = Math.max(spacing, SEQ.selfReach + 24 + width);
      } else {
        spacing = Math.max(spacing, (width + 48) / Math.abs(to - from));
      }
    });

    const centre = (index) => headWidth / 2 + index * spacing;
    // Combined fragments take room above their first row (the frame's
    // label), between operands, and below their last row.
    const fragments = (graph.arc && graph.arc.fragments) || [];
    const extra = new Array(messages.length + 1).fill(0);
    const FRAME_TOP = 34;
    const FRAME_BOTTOM = 22;
    const SEPARATOR = 14;
    const reserve = (fragment) => {
      if (fragment.end < fragment.start || fragment.start >= extra.length) return;
      extra[fragment.start] += FRAME_TOP;
      if (fragment.end + 1 < extra.length) extra[fragment.end + 1] += FRAME_BOTTOM;
      fragment.operands.forEach((operand, index) => {
        if (index > 0 && operand.start < extra.length && operand.end >= operand.start) extra[operand.start] += SEPARATOR;
        (operand.fragments || []).forEach(reserve);
      });
    };
    fragments.forEach(reserve);
    // A state or function asserted on a lifeline takes a row of its own
    // above its first message (or at its instant).
    const invariants = (graph.arc && graph.arc.invariants) || [];
    const INVARIANT = 30;
    invariants.forEach((invariant) => {
      if (invariant.start < extra.length) extra[invariant.start] += INVARIANT;
    });
    const offsets = [];
    extra.reduce((sum, value, index) => { offsets[index] = sum + value; return sum + value; }, 0);
    const rowY = (index) => SEQ.first + index * SEQ.row + (offsets[Math.min(index, offsets.length - 1)] || 0);
    const bottom = rowY(Math.max(messages.length - 1, 0)) + (offsets[messages.length] || 0) - (offsets[Math.max(messages.length - 1, 0)] || 0) + SEQ.tail + SEQ.selfDrop;
    const contentWidth = centre(Math.max(lifelines.length - 1, 0)) + headWidth / 2 + Math.max(0, selfRoom);

    const nodes = lifelines.map((lifeline, index) => {
      const represents = lifeline.arc.properties && lifeline.arc.properties.represents;
      const style = represents ? nodeStyle(represents) : nodeStyle('lifeline');
      const name = lifeline.labels[0].text;
      const cx = centre(index);
      const x = cx - headWidth / 2;
      return (
        `<g class="av-node" data-id="${esc(lifeline.id)}" data-kind="lifeline">` +
        `<title>${esc(represents ? style.label : 'Participant')}: ${esc(name)}</title>` +
        `<line x1="${num(cx)}" y1="${SEQ.head}" x2="${num(cx)}" y2="${num(bottom)}" stroke="#66717D" stroke-width="1" stroke-dasharray="5 4"/>` +
        `<rect class="av-shape" x="${num(x)}" y="0" width="${num(headWidth)}" height="${SEQ.head}" rx="3" fill="${style.fill}" stroke="${style.stroke}" stroke-width="1.3"/>` +
        glyph(style.glyph, x + 6, 6, style.stroke) +
        `<text x="${num(cx)}" y="${SEQ.head / 2 + 4}" text-anchor="middle" font-size="12" font-weight="600" fill="#18222D">${esc(name)}</text>` +
        '</g>'
      );
    });

    // The frames of the fragments, drawn under the messages; a frame spans
    // the lifelines its messages touch, and its operands are parted by a
    // dashed line.
    const frames = [];
    const drawFragment = (fragment, depth) => {
      if (fragment.end < fragment.start || messages.length === 0) return;
      const columns = fragment.lifelines.map((id) => column.get(id)).filter((c) => c !== undefined);
      // A nested frame sits inside its parent: a little narrower, a little lower.
      const left = centre(Math.min(...columns)) - 40 + depth * 10;
      const right = centre(Math.max(...columns)) + 40 - depth * 10;
      const top = rowY(fragment.start) - FRAME_TOP + 2 + depth * 10;
      const lastSelf = messages[fragment.end] && messages[fragment.end].arc.source_node === messages[fragment.end].arc.target_node;
      const bottomY = rowY(fragment.end) + (lastSelf ? SEQ.selfDrop : 0) + FRAME_BOTTOM - 6 - depth * 6;
      const title = fragment.label ? `${fragment.kind} [${fragment.label}]` : fragment.kind;
      const tagWidth = Math.max(36, title.length * 6.2 + 14);
      let body =
        `<rect x="${num(left)}" y="${num(top)}" width="${num(right - left)}" height="${num(bottomY - top)}" fill="none" stroke="#5A6672" stroke-width="1.2"/>` +
        `<path d="M${num(left)} ${num(top)}H${num(left + tagWidth)}V${num(top + 12)}L${num(left + tagWidth - 6)} ${num(top + 18)}H${num(left)}Z" fill="#E9EDF1" stroke="#5A6672" stroke-width="1.2"/>` +
        `<text x="${num(left + 6)}" y="${num(top + 13)}" font-size="10.5" font-weight="700" fill="#18222D">${esc(title)}</text>`;
      fragment.operands.forEach((operand, index) => {
        if (index > 0 && operand.end >= operand.start) {
          const y = rowY(operand.start) - SEPARATOR - 10;
          body += `<line x1="${num(left)}" y1="${num(y)}" x2="${num(right)}" y2="${num(y)}" stroke="#5A6672" stroke-width="1" stroke-dasharray="6 4"/>`;
        }
        if (operand.label && operand.end >= operand.start) {
          // The guard: right of the tag for the first operand, under the separator for the others.
          const y = index > 0 ? rowY(operand.start) - SEPARATOR + 2 : top + 13;
          const x = index > 0 ? left + 8 : left + tagWidth + 8;
          body += `<text x="${num(x)}" y="${num(y)}" font-size="10" font-style="italic" fill="#3D4852">[${esc(operand.label)}]</text>`;
        }
        (operand.fragments || []).forEach((nested) => drawFragment(nested, depth + 1));
      });
      frames.push(`<g class="av-fragment" data-kind="${esc(fragment.kind)}"><title>${esc(title)}</title>${body}</g>`);
    };
    fragments.forEach((fragment) => drawFragment(fragment, 0));

    // The invariants: a rounded box on the lifeline, spanning its messages.
    const boxes = invariants.map((invariant) => {
      const col = column.get(invariant.lifeline);
      if (col === undefined) return '';
      const cx = centre(col);
      const instant = invariant.end < invariant.start || messages.length === 0;
      const at = Math.min(invariant.start, Math.max(messages.length - 1, 0));
      const top = messages.length === 0 ? SEQ.first - INVARIANT : rowY(at) - INVARIANT + 4;
      const bottomY = instant ? top + 22 : rowY(invariant.end) + 10;
      const width = Math.max(72, invariant.label.length * 6.2 + 18);
      const fill = invariant.kind === 'function' ? '#E3F1E6' : '#FFF4D6';
      const stroke = invariant.kind === 'function' ? '#2E7D4F' : '#B07A00';
      const title = `${invariant.kind === 'function' ? 'Function' : 'State'} on the lifeline: ${invariant.label}`;
      return (
        `<g class="av-invariant" data-id="${esc(invariant.id)}" data-kind="${esc(invariant.kind)}"><title>${esc(title)}</title>` +
        `<rect x="${num(cx - width / 2)}" y="${num(top)}" width="${num(width)}" height="${num(bottomY - top)}" rx="9" fill="${fill}" fill-opacity="0.85" stroke="${stroke}" stroke-width="1.2"/>` +
        `<text x="${num(cx)}" y="${num(top + 14)}" text-anchor="middle" font-size="10.5" font-weight="600" fill="#18222D">${esc(invariant.label)}</text>` +
        '</g>'
      );
    });

    const style = edgeStyle('message');
    const edges = messages.map((message, index) => {
      const arc = message.arc;
      const kind = (arc.properties && arc.properties.type) || 'sync';
      const y = rowY(index);
      const x1 = centre(column.get(arc.source_node));
      const x2 = centre(column.get(arc.target_node));
      const text = messageText(message, index);
      const dash = kind === 'return' ? ' stroke-dasharray="6 3"' : '';
      const head = (tipX, tipY, dir) => {
        const back = tipX - dir * 9;
        return kind === 'sync'
          ? `<polygon points="${num(tipX)},${num(tipY)} ${num(back)},${num(tipY - 4.5)} ${num(back)},${num(tipY + 4.5)}" fill="${style.stroke}"/>`
          : `<polyline points="${num(back)},${num(tipY - 4.5)} ${num(tipX)},${num(tipY)} ${num(back)},${num(tipY + 4.5)}" fill="none" stroke="${style.stroke}" stroke-width="${style.width}"/>`;
      };
      let d;
      let label;
      let arrow;
      if (x1 === x2) {
        d = `M${num(x1)} ${num(y)}H${num(x1 + SEQ.selfReach)}V${num(y + SEQ.selfDrop)}H${num(x1)}`;
        arrow = head(x1, y + SEQ.selfDrop, -1);
        label = `<text class="av-edge-label" x="${num(x1 + SEQ.selfReach + 8)}" y="${num(y + SEQ.selfDrop / 2 + 3.5)}" font-size="10" fill="${style.stroke}">${esc(text)}</text>`;
      } else {
        const dir = x2 > x1 ? 1 : -1;
        d = `M${num(x1)} ${num(y)}H${num(x2)}`;
        arrow = head(x2, y, dir);
        label = `<text class="av-edge-label" x="${num((x1 + x2) / 2)}" y="${num(y - 6)}" text-anchor="middle" font-size="10" fill="${style.stroke}" stroke="#FFFFFF" stroke-width="3" paint-order="stroke" stroke-linejoin="round">${esc(text)}</text>`;
      }
      const tip = `Message ${index + 1}: ${message.labels[0].text} — ${kind === 'async' ? 'asynchronous' : kind === 'return' ? 'return' : 'synchronous'}`;
      return (
        `<g class="av-edge" data-id="${esc(message.id)}" data-kind="message" data-source="${esc(arc.source_node)}" data-target="${esc(arc.target_node)}">` +
        `<title>${esc(tip)}</title>` +
        `<path class="av-hit" d="${d}" fill="none" stroke="#000" stroke-opacity="0" stroke-width="12"/>` +
        `<path class="av-line" d="${d}" fill="none" stroke="${style.stroke}" stroke-width="${style.width}"${dash}/>` +
        arrow + label + '</g>'
      );
    });

    return frame(graph, nodes, [...frames, ...boxes, ...edges], '', contentWidth, bottom);
  }

  function frame(graph, nodes, edges, defs, contentWidth, contentHeight) {
    const width = Math.ceil(contentWidth + MARGIN * 2);
    const height = Math.ceil(contentHeight + MARGIN * 2);
    const title = graph.arc && graph.arc.title ? graph.arc.title : graph.id;
    const svg =
      `<svg xmlns="http://www.w3.org/2000/svg" class="av-svg" role="img" aria-label="${esc(title)}" ` +
      `width="${width}" height="${height}" viewBox="${-MARGIN} ${-MARGIN} ${width} ${height}" font-family="${FONT}">` +
      `<defs>${defs}</defs>` +
      `<rect x="${-MARGIN}" y="${-MARGIN}" width="${width}" height="${height}" fill="#FFFFFF"/>` +
      `<g class="av-nodes">${nodes.join('')}</g><g class="av-edges">${edges.join('')}</g></svg>`;
    return { svg, width, height };
  }

  /** True when the graph is drawn as-is, without a layout engine. */
  function isSequence(graph) {
    return Boolean(graph.arc && graph.arc.layout === 'sequence');
  }

  /**
   * Draw a graph: a laid-out ELK graph, or a scenario (see `isSequence`).
   * Returns the standalone SVG markup and its size.
   */
  function renderSvg(graph) {
    if (isSequence(graph)) return renderSequence(graph);
    const nodes = [];
    const origins = new Map();
    for (const child of graph.children || []) drawNode(child, 0, 0, nodes, origins);
    const edges = (graph.edges || []).map((edge) => drawEdge(edge, origins, graph.id));
    return frame(graph, nodes, edges, arrowMarkers(graph.edges || []), graph.width || 0, graph.height || 0);
  }

  /**
   * Lay a graph out with ELK, working around two failures of elkjs 0.9.3 on
   * deep graphs: it throws (`Cannot read properties of null (reading 'p')`)
   * when containers fix the sides of their own ports, and edge wrapping
   * (`MULTI_EDGE`) may drop an edge's route. The layout is then redone with
   * those containers' ports free (leaves keep their sides), and without
   * wrapping when an edge is left unrouted. Any other failure is the caller's.
   */
  async function layOut(elk, graph) {
    const pristine = JSON.stringify(graph);
    let attempt = graph;
    let laidOut;
    try {
      laidOut = await elk.layout(graph);
    } catch (error) {
      attempt = JSON.parse(pristine);
      if (!freeContainerPorts(attempt)) throw error;
      const copy = JSON.stringify(attempt);
      laidOut = await elk.layout(attempt);
      attempt = JSON.parse(copy);
    }
    const wrapping = 'elk.layered.wrapping.strategy';
    if (!hasUnroutedEdge(laidOut) || !(attempt.layoutOptions || {})[wrapping]) return laidOut;
    const unwrapped = attempt === graph ? JSON.parse(pristine) : attempt;
    delete unwrapped.layoutOptions[wrapping];
    return elk.layout(unwrapped);
  }

  /** Free the ports of every container that fixes their sides; how many. */
  function freeContainerPorts(node) {
    let freed = 0;
    for (const child of node.children || []) {
      const options = child.layoutOptions || {};
      if (child.children?.length && child.ports?.length && options['elk.portConstraints'] === 'FIXED_SIDE') {
        child.layoutOptions = { ...options, 'elk.portConstraints': 'FREE' };
        freed += 1;
      }
      freed += freeContainerPorts(child);
    }
    return freed;
  }

  function hasUnroutedEdge(node) {
    return (node.edges || []).some((edge) => !edge.sections?.length) || (node.children || []).some(hasUnroutedEdge);
  }

  /** Ids of everything drawn: used to check a rendering is complete. */
  function drawnIds(graph) {
    const ids = [];
    const walk = (node) => {
      ids.push(node.id);
      for (const port of node.ports || []) ids.push(port.id);
      for (const child of node.children || []) walk(child);
    };
    for (const child of graph.children || []) walk(child);
    for (const edge of graph.edges || []) ids.push(edge.id);
    return ids;
  }

  return { renderSvg, isSequence, layOut, drawnIds, nodeStyle, edgeStyle, esc };
});
