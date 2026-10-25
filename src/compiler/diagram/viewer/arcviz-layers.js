/*
 * ArcViz layers — draw.io's Arrange menu on a diagram: the stacking order of
 * the boxes (to the front, to the back, one step forward or backward among
 * the boxes they overlap), layers made of the elements' tags (shown or
 * hidden, locked, drawn above or below each other), groups (a frame around
 * sibling boxes that moves them together, nested, collapsible) and locks.
 *
 * None of it changes the model: it is kept in the layout file, beside the
 * placement (`layers`, `groups`, `locked`), and applied to a laid-out graph
 * before it is drawn, so what is hidden is neither drawn nor exported.
 * `create` is the state and the graph transform (no DOM); `attach` gives a
 * mounted viewer the commands, each one redraw and one change of the
 * layout file (one undo step for the host).
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizLayers = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  /** The layer of the elements that carry no tag. */
  const DEFAULT = '';
  /** Drawing ids of group frames. */
  const GROUP = 'group:';
  const PAD = { side: 12, top: 26, bottom: 12 };
  const COLLAPSED = { width: 150, height: 56 };
  const LIMITS = { layers: 1000, groups: 2000, members: 5000, locked: 50000, name: 200 };

  const isGroupId = (id) => typeof id === 'string' && id.startsWith(GROUP);
  const groupKey = (id) => (isGroupId(id) ? id.slice(GROUP.length) : id);
  const names = (list, limit) => (Array.isArray(list) ? [...new Set(list.filter((n) => typeof n === 'string' && n.length <= LIMITS.name))].slice(0, limit) : []);
  const own = (object, key) => (object && typeof object === 'object' && Object.prototype.hasOwnProperty.call(object, key) ? object[key] : undefined);

  // --- stacking: pure ---------------------------------------------------------

  const overlap = (a, b) => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;

  /**
   * The new order of sibling units (bottom first) once `selected` moved.
   * units: [{ id, rect, layer }] in drawing order; how: 'front' | 'back' |
   * 'forward' | 'backward'. Forward and backward take one step past the
   * nearest unit of the same layer that overlaps the moved one; several
   * selected keep their order among themselves. Returns ids.
   */
  function restacked(units, selected, how) {
    const chosen = new Set(selected);
    const ids = units.map((unit) => unit.id);
    if (how === 'front') return [...ids.filter((id) => !chosen.has(id)), ...ids.filter((id) => chosen.has(id))];
    if (how === 'back') return [...ids.filter((id) => chosen.has(id)), ...ids.filter((id) => !chosen.has(id))];
    const byId = new Map(units.map((unit) => [unit.id, unit]));
    const order = ids.slice();
    const up = how === 'forward';
    const moving = ids.filter((id) => chosen.has(id));
    for (const id of up ? moving.reverse() : moving) {
      const at = order.indexOf(id);
      const unit = byId.get(id);
      const step = up ? 1 : -1;
      let past = -1;
      for (let k = at + step; k >= 0 && k < order.length; k += step) {
        const other = byId.get(order[k]);
        if (chosen.has(other.id) || other.layer !== unit.layer) continue;
        if (overlap(unit.rect, other.rect)) {
          past = k;
          break;
        }
      }
      if (past < 0) continue;
      // Out of the list, then back in beside the one it passes: above it going up, below it going down.
      order.splice(at, 1);
      order.splice(past, 0, id);
    }
    return order;
  }

  /** The z values of `order` (bottom first): 1, 2, 3… — what the layout file keeps under `order`. */
  const stackValues = (order) => Object.fromEntries(order.map((id, index) => [id, index + 1]));

  // --- same size: pure ----------------------------------------------------------

  /** The resizes that give every box the width and/or height of `reference` (draw.io: the last selected). */
  function sameSize(boxes, reference, how) {
    const ref = boxes.find((box) => box.id === reference);
    if (!ref) return [];
    return boxes.filter((box) => box.id !== reference).map((box) => ({
      id: box.id,
      dw: how === 'height' ? 0 : ref.width - box.width,
      dh: how === 'width' ? 0 : ref.height - box.height,
    })).filter((change) => Math.abs(change.dw) > 0.01 || Math.abs(change.dh) > 0.01);
  }

  // --- the state ----------------------------------------------------------------

  /**
   * deps: { tagsOf(view, id, isEdge) -> [tag] } — the tags of a drawn
   * element (its first tag is its layer).
   */
  function create(deps) {
    const tagsOf = (deps && deps.tagsOf) || (() => []);
    let order = []; // layers drawn top first; empty: no layer is drawn above another
    let hidden = new Set();
    let lockedLayers = new Set();
    let locked = new Set(); // drawing ids locked one by one
    let groups = new Map(); // view -> [{ id, members, label, collapsed }]
    const entered = new Map(); // view -> group id, for this visit only

    const layerOf = (view, id, isEdge) => {
      const tags = tagsOf(view, id, Boolean(isEdge)) || [];
      return typeof tags[0] === 'string' ? tags[0] : DEFAULT;
    };
    const rankOf = (layer) => {
      if (order.length === 0) return 0;
      const at = order.indexOf(layer);
      return at < 0 ? order.length : order.length - 1 - at; // a layer not ordered yet is on top, as a new one in draw.io
    };

    function toFile() {
      const out = {};
      if (order.length || hidden.size || lockedLayers.size) {
        out.layers = { order: order.slice(), hidden: [...hidden].sort(), locked: [...lockedLayers].sort() };
      }
      const kept = [...groups.keys()].sort().filter((view) => groups.get(view).length > 0);
      if (kept.length) {
        out.groups = Object.fromEntries(kept.map((view) => [view, groups.get(view).map((g) => ({ id: g.id, members: g.members.slice(), ...(g.label ? { label: g.label } : {}), ...(g.collapsed ? { collapsed: true } : {}) }))]));
      }
      if (locked.size) out.locked = [...locked].sort();
      return out;
    }

    function load(file) {
      const layers = own(file, 'layers');
      order = names(own(layers, 'order'), LIMITS.layers);
      hidden = new Set(names(own(layers, 'hidden'), LIMITS.layers));
      lockedLayers = new Set(names(own(layers, 'locked'), LIMITS.layers));
      locked = new Set(names(own(file, 'locked'), LIMITS.locked));
      groups = new Map();
      const all = own(file, 'groups');
      for (const view of all && typeof all === 'object' ? Object.keys(all) : []) {
        if (view === '__proto__' || !Array.isArray(all[view])) continue;
        const seen = new Set();
        const list = [];
        for (const entry of all[view].slice(0, LIMITS.groups)) {
          const id = own(entry, 'id');
          if (typeof id !== 'string' || !id || id.length > LIMITS.name || seen.has(id)) continue;
          const members = names(own(entry, 'members'), LIMITS.members);
          if (members.length === 0) continue;
          seen.add(id);
          const label = own(entry, 'label');
          list.push({ id, members, label: typeof label === 'string' ? label.slice(0, LIMITS.name) : '', collapsed: own(entry, 'collapsed') === true });
        }
        if (list.length) groups.set(view, list);
      }
      entered.clear();
    }

    const groupsOf = (view) => groups.get(view) || [];
    const setGroups = (view, list) => {
      if (list.length) groups.set(view, list);
      else groups.delete(view);
    };
    const parentGroup = (view, id) => groupsOf(view).find((g) => g.members.includes(id)) || null;

    /** What tells one drawing of a view from another: the layer and group state that changes it. */
    const signature = (view) => `|L:${order.join(',')}/${[...hidden].sort().join(',')}|G:${JSON.stringify(groupsOf(view))}`;

    const isLocked = (view, id, isEdge) => locked.has(id) || lockedLayers.has(layerOf(view, id, isEdge));

    return {
      DEFAULT,
      toFile,
      load,
      signature,
      layerOf,
      rankOf,
      isLocked,
      lockedOne: (id) => locked.has(id),
      state: () => ({ order: order.slice(), hidden: [...hidden], locked: [...lockedLayers] }),
      setHidden(name, on) { hidden = new Set([...hidden].filter((n) => n !== name)); if (on) hidden.add(name); },
      setLayerLocked(name, on) { lockedLayers = new Set([...lockedLayers].filter((n) => n !== name)); if (on) lockedLayers.add(name); },
      /** Layers top first; every layer shown on the panel is written, so the order is the reader's. */
      setOrder(list) { order = names(list, LIMITS.layers); },
      setLocked(ids, on) {
        const next = new Set(locked);
        for (const id of ids) if (on) next.add(id); else next.delete(id);
        locked = next;
      },
      groupsOf: (view) => groupsOf(view).map((g) => ({ ...g, members: g.members.slice() })),
      groupById: (view, id) => groupsOf(view).find((g) => g.id === groupKey(id)) || null,
      parentGroup,
      setGroups,
      enteredOf: (view) => entered.get(view) || null,
      enter(view, id) { if (id) entered.set(view, groupKey(id)); else entered.delete(view); },
      /** The graph as drawn: hidden layers left out, layers stacked, groups framed or collapsed. */
      apply: (graph, view) => arrange(graph, view, { layerOf, rankOf, hidden, ordered: order.length > 0, groups: groupsOf(view) }),
    };
  }

  // --- the graph transform --------------------------------------------------------

  function survey(graph) {
    const nodes = new Map(); // id -> { node, holder, x, y } (x, y absolute)
    const ports = new Map(); // port id -> owner id
    const walk = (node, holder, ox, oy) => {
      const entry = { node, holder, x: ox + node.x, y: oy + node.y };
      nodes.set(node.id, entry);
      for (const port of node.ports || []) ports.set(port.id, node.id);
      for (const child of node.children || []) walk(child, node, entry.x, entry.y);
    };
    for (const child of graph.children || []) walk(child, graph, 0, 0);
    return { nodes, ports };
  }

  /** The point where the line from the middle of `rect` towards `to` leaves it. */
  function border(rect, to) {
    const cx = rect.x + rect.width / 2;
    const cy = rect.y + rect.height / 2;
    const dx = to.x - cx;
    const dy = to.y - cy;
    if (!dx && !dy) return { x: cx, y: rect.y };
    const k = Math.min(Math.abs(dx) > 0 ? rect.width / 2 / Math.abs(dx) : Infinity, Math.abs(dy) > 0 ? rect.height / 2 / Math.abs(dy) : Infinity);
    return { x: cx + dx * k, y: cy + dy * k };
  }

  function arrange(source, view, s) {
    const anyHidden = s.hidden.size > 0;
    if (!anyHidden && !s.ordered && s.groups.length === 0) return source;
    const graph = JSON.parse(JSON.stringify(source));
    const ends = (edge) => [...(edge.sources || []), ...(edge.targets || [])];

    // 1. Hidden layers: their boxes (with what they hold) and the lines that touch them are not drawn.
    if (anyHidden) {
      const gone = new Set();
      const drop = (node) => {
        gone.add(node.id);
        for (const port of node.ports || []) gone.add(port.id);
        (node.children || []).forEach(drop);
      };
      const prune = (holder) => {
        holder.children = (holder.children || []).filter((node) => {
          if (s.hidden.has(s.layerOf(view, node.id, false))) {
            drop(node);
            return false;
          }
          prune(node);
          return true;
        });
      };
      prune(graph);
      graph.edges = (graph.edges || []).filter((edge) => !gone.has(edge.container) && !ends(edge).some((end) => gone.has(end)) && !s.hidden.has(s.layerOf(view, edge.id, true)));
    }

    // 2. Layers stacked: among siblings, a higher layer is drawn later (on top); stable within a layer.
    if (s.ordered) {
      const stack = (holder) => {
        if (!holder.children) return;
        holder.children = holder.children.map((node, index) => ({ node, index, rank: s.rankOf(s.layerOf(view, node.id, false)) }))
          .sort((a, b) => a.rank - b.rank || a.index - b.index).map(({ node }) => node);
        holder.children.forEach(stack);
      };
      stack(graph);
    }

    // 3. Groups: a frame around sibling boxes, inner groups first.
    if (s.groups.length) frame(graph, s.groups);
    return graph;
  }

  /** Frame the groups whose members are siblings in what is drawn; collapse the collapsed ones. */
  function frame(graph, groups) {
    let map = survey(graph);
    const done = new Map(); // group id -> frame node
    const blocks = new Map(); // frame node -> the frame and what it draws after it
    const pending = groups.slice();
    const collapsedInto = new Map(); // drawn id (node or port) -> collapsed frame id
    let progress = true;
    while (pending.length && progress) {
      progress = false;
      for (let i = 0; i < pending.length; i += 1) {
        const group = pending[i];
        const inner = group.members.filter(isGroupId);
        if (inner.some((id) => pending.some((g) => g.id === groupKey(id)))) continue; // an inner group first
        pending.splice(i, 1);
        i -= 1;
        progress = true;
        const members = group.members.map((id) => (isGroupId(id) ? done.get(groupKey(id)) : map.nodes.get(id) && map.nodes.get(id).node)).filter(Boolean);
        if (members.length === 0) continue;
        const holderOf = (node) => (map.nodes.get(node.id) || {}).holder;
        const holder = holderOf(members[0]);
        if (!holder || members.some((node) => holderOf(node) !== holder)) continue; // no longer siblings here: not drawn
        const left = Math.min(...members.map((n) => n.x));
        const top = Math.min(...members.map((n) => n.y));
        const right = Math.max(...members.map((n) => n.x + n.width));
        const bottom = Math.max(...members.map((n) => n.y + n.height));
        const x = Math.max(0, left - PAD.side);
        const y = Math.max(0, top - PAD.top);
        const id = GROUP + group.id;
        const node = {
          id,
          x,
          y,
          width: right + PAD.side - x,
          height: bottom + PAD.bottom - y,
          labels: [{ text: group.label || '' }],
          ports: [],
          arc: { kind: 'diagram_group', group: group.id, members: group.members.slice() },
        };
        if (group.collapsed) {
          node.width = COLLAPSED.width;
          node.height = COLLAPSED.height;
          node.labels = [{ text: group.label || 'Group' }];
          node.arc.collapsed = true;
          node.arc.folded = members.length;
        }
        // The frame takes the place of its topmost member; its members follow it, in their order,
        // an inner group with its own members (its block).
        const blockOf = (m) => blocks.get(m) || [m];
        const kept = members.slice().sort((p, q) => holder.children.indexOf(p) - holder.children.indexOf(q));
        const block = kept.flatMap(blockOf);
        const inGroup = new Set(block);
        const at = Math.max(...block.map((m) => holder.children.indexOf(m)));
        const before = holder.children.slice(0, at + 1).filter((n) => !inGroup.has(n));
        const after = holder.children.slice(at + 1).filter((n) => !inGroup.has(n));
        holder.children = [...before, node, ...(group.collapsed ? [] : block), ...after];
        blocks.set(node, group.collapsed ? [node] : [node, ...block]);
        if (group.collapsed) {
          const swallow = (n) => {
            collapsedInto.set(n.id, id);
            for (const port of n.ports || []) collapsedInto.set(port.id, id);
            (n.children || []).forEach(swallow);
          };
          block.forEach(swallow);
          // Inner frames already swallowed: what they took now ends on this one.
          for (const [drawn, into] of collapsedInto) if (inGroup.has(done.get(groupKey(into)))) collapsedInto.set(drawn, id);
        }
        done.set(group.id, node);
        map = survey(graph);
      }
    }
    if (collapsedInto.size === 0) return;
    // The lines to what a collapsed group holds end on its border.
    map = survey(graph);
    const originOf = (id) => {
      const entry = id && map.nodes.get(id);
      return entry ? { x: entry.x, y: entry.y } : { x: 0, y: 0 };
    };
    const rectOf = (id) => {
      const entry = map.nodes.get(id);
      return { x: entry.x, y: entry.y, width: entry.node.width, height: entry.node.height };
    };
    graph.edges = (graph.edges || []).filter((edge) => {
      const from = (edge.sources || [])[0];
      const to = (edge.targets || [])[0];
      const a = collapsedInto.get(from);
      const b = collapsedInto.get(to);
      if (!a && !b) return true;
      if ((a && a === b) || collapsedInto.has(edge.container)) return false; // inside the collapsed group
      const origin = edge.container && edge.container !== graph.id ? originOf(edge.container) : { x: 0, y: 0 };
      const section = (edge.sections || [])[0];
      if (!section) return false;
      const abs = (p) => ({ x: p.x + origin.x, y: p.y + origin.y });
      let start = abs(section.startPoint);
      let end = abs(section.endPoint);
      if (a) start = border(rectOf(a), b ? { x: rectOf(b).x + rectOf(b).width / 2, y: rectOf(b).y + rectOf(b).height / 2 } : end);
      if (b) end = border(rectOf(b), start);
      const rel = (p) => ({ x: p.x - origin.x, y: p.y - origin.y });
      edge.sections = [{ id: section.id, startPoint: rel(start), endPoint: rel(end), bendPoints: [] }];
      for (const label of edge.labels || []) {
        if (label.x === undefined) continue;
        label.x = (start.x + end.x) / 2 - origin.x - (label.width || 0) / 2;
        label.y = (start.y + end.y) / 2 - origin.y - 12;
      }
      return true;
    });
  }

  // --- the commands on a mounted viewer ----------------------------------------------

  /**
   * viewer: { store, scope, replace, view(), shown(), announce } — the
   * store of `create`, the viewer's scope, its redraw-and-report, the view
   * on screen and its drawn graph (a promise).
   */
  function attach(viewer) {
    const { store, scope, replace, view, shown, announce } = viewer;
    const refuse = (reason) => Promise.resolve({ ok: false, reason });
    const ok = (text) => {
      if (text) announce(text);
      return { ok: true };
    };

    /** The sibling list the drawn ids sit in. */
    async function siblingsOf(ids) {
      const drawn = await shown();
      if (!drawn) return null;
      const map = survey(drawn.graph);
      const holders = new Set(ids.map((id) => (map.nodes.get(id) || {}).holder));
      if (holders.has(undefined) || holders.size !== 1) return null;
      return { holder: [...holders][0], map };
    }

    /** Every box a group holds, inner groups opened. */
    function membersDeep(v, id) {
      const group = store.groupById(v, id);
      if (!group) return [];
      return group.members.flatMap((member) => (isGroupId(member) ? membersDeep(v, member) : [member]));
    }

    /** The units of a sibling list: a group frame with its members is one unit. */
    function unitsOf(holder, v) {
      const units = [];
      const groupFor = new Map();
      const walkGroup = (frameNode, unit) => {
        for (const member of frameNode.arc.members) {
          groupFor.set(member, unit);
          const inner = holder.children.find((n) => n.id === member);
          if (inner && inner.arc && inner.arc.kind === 'diagram_group') walkGroup(inner, unit);
        }
      };
      for (const node of holder.children || []) {
        if (groupFor.has(node.id)) {
          const unit = groupFor.get(node.id);
          if (!(node.arc && node.arc.kind === 'diagram_group')) unit.nodes.push(node.id);
          continue;
        }
        const isFrame = node.arc && node.arc.kind === 'diagram_group';
        const unit = { id: node.id, rect: { x: node.x, y: node.y, width: node.width, height: node.height }, layer: store.rankOf(store.layerOf(v, node.id, false)), nodes: isFrame ? [] : [node.id] };
        if (isFrame) {
          walkGroup(node, unit);
          // Collapsed: its members are not drawn, but they keep their place among the others.
          if (node.arc.collapsed) unit.nodes.push(...membersDeep(v, node.arc.group));
        }
        units.push(unit);
      }
      return units;
    }

    /** Front, back, forward, backward: the selection among its siblings, kept as `order` in the layout file. */
    async function restack(ids, how) {
      const list = (Array.isArray(ids) ? ids : [ids]).filter(Boolean);
      if (!list.length) return refuse('Select a box first.');
      const v = view();
      const at = await siblingsOf(list);
      if (!at) return refuse('Only boxes side by side in one container are stacked together.');
      const units = unitsOf(at.holder, v);
      const chosen = list.map((id) => (units.find((u) => u.id === id || u.nodes.includes(id)) || {}).id).filter(Boolean);
      const next = restacked(units, [...new Set(chosen)], how);
      if (next.join() === units.map((u) => u.id).join()) {
        return refuse(how === 'forward' || how === 'backward' ? `Nothing it overlaps is ${how === 'forward' ? 'in front of' : 'behind'} it.` : `Already at the ${how}.`);
      }
      return write(next.flatMap((id) => units.find((u) => u.id === id).nodes), how);
    }

    async function write(order, how) {
      const v = view();
      await replace(() => scope.setStacking(v, stackValues(order)));
      const said = { front: 'Brought to the front.', back: 'Sent to the back.', forward: 'Brought forward.', backward: 'Sent backward.' }[how];
      return ok(said);
    }

    /** Group sibling boxes (or groups) of the view on screen. */
    async function group(ids, label) {
      const list = [...new Set(ids)];
      if (list.length < 2) return refuse('Select two or more boxes to group them.');
      const v = view();
      const at = await siblingsOf(list);
      if (!at) return refuse('The boxes of a group must sit side by side in one container: these are in different ones.');
      const taken = new Set(store.groupsOf(v).map((g) => g.id));
      let n = 1;
      while (taken.has(`g${n}`)) n += 1;
      const id = `g${n}`;
      const parent = store.parentGroup(v, list[0]);
      const others = store.groupsOf(v).map((g) => (parent && g.id === parent.id ? { ...g, members: [...g.members.filter((m) => !list.includes(m)), GROUP + id] } : { ...g, members: g.members.filter((m) => !list.includes(m)) })).filter((g) => g.members.length > 0);
      await replace(() => store.setGroups(v, [...others, { id, members: list, label: label || '', collapsed: false }]));
      return { ok: true, id: GROUP + id };
    }

    async function ungroup(id) {
      const v = view();
      const found = store.groupById(v, id);
      if (!found) return refuse('Select a group to ungroup it.');
      const key = GROUP + found.id;
      const rest = store.groupsOf(v).filter((g) => g.id !== found.id).map((g) => ({ ...g, members: g.members.flatMap((m) => (m === key ? found.members : [m])) }));
      if (store.enteredOf(v) === found.id) store.enter(v, null);
      await replace(() => store.setGroups(v, rest));
      return ok('Ungrouped.');
    }

    async function removeFromGroup(ids) {
      const v = view();
      const list = new Set(ids);
      if (!store.groupsOf(v).some((g) => g.members.some((m) => list.has(m)))) return refuse('The selection is in no group.');
      await replace(() => store.setGroups(v, store.groupsOf(v).map((g) => ({ ...g, members: g.members.filter((m) => !list.has(m)) })).filter((g) => g.members.length > 0)));
      return ok('Removed from the group.');
    }

    async function changeGroup(id, change, text) {
      const v = view();
      const found = store.groupById(v, id);
      if (!found) return refuse('Select a group first.');
      await replace(() => store.setGroups(v, store.groupsOf(v).map((g) => (g.id === found.id ? { ...g, ...change } : g))));
      return ok(text);
    }

    /** Lock or unlock elements one by one (moves, resizes, reconnections, deletes, renames refused). */
    async function lock(ids, on) {
      await replace(() => store.setLocked(ids, on));
      return ok(on ? 'Locked.' : 'Unlocked.');
    }

    async function layerChange(change, text) {
      await replace(change);
      return ok(text);
    }

    /** Several boxes resized at once (same size): one redraw, one layout change. */
    async function resizeMany(changes) {
      const v = view();
      const free = changes.filter((c) => !store.isLocked(v, c.id, false));
      if (!free.length) return refuse('Nothing to resize.');
      await replace(() => free.forEach((c) => scope.resizeBy(v, c.id, c.dw, c.dh)));
      return ok('Resized.');
    }

    async function clearRoutes(edges) {
      const v = view();
      const bent = edges.filter((e) => scope.routeOf(v, e).length > 0 && !store.isLocked(v, e, true));
      if (!bent.length) return refuse('The selected lines have no waypoints.');
      await replace(() => bent.forEach((e) => scope.setRoute(v, e, [])));
      return ok(`Waypoints cleared on ${bent.length} line${bent.length > 1 ? 's' : ''}.`);
    }

    /** What is drawn in the view on screen: boxes (frames left out) and lines. */
    async function drawn() {
      const now = await shown();
      if (!now) return { nodes: [], edges: [] };
      const nodes = [...survey(now.graph).nodes.values()].map((e) => e.node).filter((n) => !(n.arc && n.arc.kind === 'diagram_group')).map((n) => n.id);
      return { nodes, edges: (now.graph.edges || []).map((e) => e.id) };
    }

    return {
      restack,
      bringToFront: (ids) => restack(ids, 'front'),
      sendToBack: (ids) => restack(ids, 'back'),
      bringForward: (ids) => restack(ids, 'forward'),
      sendBackward: (ids) => restack(ids, 'backward'),
      group,
      ungroup,
      removeFromGroup,
      collapseGroup: (id, on) => changeGroup(id, { collapsed: Boolean(on) }, on ? 'Group collapsed.' : 'Group expanded.'),
      labelGroup: (id, label) => changeGroup(id, { label: String(label || '').slice(0, LIMITS.name) }, 'Group renamed.'),
      enterGroup: (id) => {
        const v = view();
        if (!store.groupById(v, id)) return false;
        store.enter(v, id);
        return true;
      },
      exitGroup: () => store.enter(view(), null),
      enteredGroup: () => store.enteredOf(view()),
      groupOf: (id) => {
        const g = store.parentGroup(view(), id);
        return g ? GROUP + g.id : null;
      },
      groupInfo: (id) => store.groupById(view(), id),
      lock,
      isLocked: (id, isEdge) => store.isLocked(view(), id, isEdge),
      lockedOne: (id) => store.lockedOne(id),
      layerState: () => store.state(),
      layerOf: (id, isEdge) => store.layerOf(view(), id, isEdge),
      setLayerHidden: (name, on) => layerChange(() => store.setHidden(name, on), on ? 'Layer hidden.' : 'Layer shown.'),
      setLayerLocked: (name, on) => layerChange(() => store.setLayerLocked(name, on), on ? 'Layer locked.' : 'Layer unlocked.'),
      setLayerOrder: (list) => layerChange(() => store.setOrder(list), 'Layers reordered.'),
      resizeMany,
      clearRoutes,
      drawn,
      membersDeep: (id) => membersDeep(view(), id),
    };
  }

  /**
   * Draw the marks the drawing gets once on screen: a padlock on what is
   * locked, the group open for editing set apart. `canvas` holds the SVG.
   */
  function decorate(canvas, store, v) {
    const SVG = 'http://www.w3.org/2000/svg';
    for (const node of canvas.querySelectorAll('.av-node, .av-edge')) {
      const isEdge = node.classList.contains('av-edge');
      if (!store.isLocked(v, node.dataset.id, isEdge)) continue;
      node.classList.add('is-locked');
      const shape = node.querySelector(':scope > .av-shape');
      if (!shape || isEdge) continue;
      const x = Number(shape.getAttribute('x')) + Number(shape.getAttribute('width')) - 13;
      const y = Number(shape.getAttribute('y')) + 3;
      const badge = document.createElementNS(SVG, 'g');
      badge.setAttribute('class', 'av-lock');
      badge.setAttribute('transform', `translate(${x} ${y})`);
      const title = document.createElementNS(SVG, 'title');
      title.textContent = store.lockedOne(node.dataset.id) ? 'Locked (⌘L unlocks)' : 'On a locked layer';
      const body = document.createElementNS(SVG, 'rect');
      for (const [k, val] of Object.entries({ x: 1, y: 4.5, width: 8, height: 6, rx: 1 })) body.setAttribute(k, String(val));
      const hoop = document.createElementNS(SVG, 'path');
      hoop.setAttribute('d', 'M3 4.5V3a2 2 0 0 1 4 0v1.5');
      badge.append(title, body, hoop);
      node.appendChild(badge);
    }
    const entered = store.enteredOf(v);
    canvas.classList.toggle('is-in-group', Boolean(entered));
    const frameNode = entered && [...canvas.querySelectorAll('.av-node')].find((node) => node.dataset.id === GROUP + entered);
    if (frameNode) frameNode.classList.add('is-entered');
  }

  /** A click on a member of a group selects the group, unless that group is entered. */
  function redirect(store, v, id) {
    let at = id;
    const entered = store.enteredOf(v);
    for (let guard = 0; guard < 64; guard += 1) {
      const parent = store.parentGroup(v, at);
      if (!parent || parent.id === entered) return at;
      at = GROUP + parent.id;
    }
    return at;
  }

  return { create, attach, decorate, redirect, restacked, stackValues, sameSize, arrange, isGroupId, GROUP, DEFAULT };
});
