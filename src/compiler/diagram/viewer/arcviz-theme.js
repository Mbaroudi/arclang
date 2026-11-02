/*
 * ArcViz themes — the color code of the diagrams as a file
 * (`arclang-theme.json`, version 1), the built-in presets, and the rules a
 * theme must pass before it colors anything.
 *
 * A theme gives each Arcadia concept family its fill, stroke and text, each
 * kind of exchange its line, width and style, and the markings (variant,
 * tag). One theme colors every layer at once. The validator holds a theme
 * to what a systems engineer expects of a color code:
 *   - one concept, one color on every layer (a function is a function in
 *     System, Logical and Physical analysis);
 *   - the colors that mean something to the tool (error red, warning amber,
 *     the selection accent, the AI purple) never color an element kind;
 *   - text on its fill reaches WCAG AA (4.5:1);
 *   - what one view draws together stays distinguishable: CIEDE2000 >= 10,
 *     and still apart for protanopia, deuteranopia and a grayscale print,
 *     unless the two differ by shape or line style;
 *   - a bounded palette.
 * A theme that breaks a rule is refused with the reasons; it is never
 * applied in part. `src/compiler/diagram/theme.rs` mirrors these rules for
 * the CLI; tools/diagram_render/theme.test.mjs and the Rust tests check both
 * against the same fixtures.
 *
 * Pure: no DOM. The presets are the JSON files of viewer/themes, read here
 * under Node and from `ArcVizThemePresets` in a page (the CLI and the
 * workbench put them there).
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) {
    const presets = ['arclang-default', 'capella-classic', 'colorblind-safe', 'print-grayscale', 'high-contrast', 'monochrome-document']
      .map((id) => require(`./themes/${id}.json`));
    module.exports = factory(require('./arcviz-color.js'), presets);
  } else {
    root.ArcVizTheme = factory(root.ArcVizColor, root.ArcVizThemePresets || []);
  }
})(typeof globalThis !== 'undefined' ? globalThis : this, function (Color, PRESETS) {
  'use strict';

  const VERSION = 1;

  // The concept families a theme colors: the kinds of box each one draws,
  // and the silhouette the notation gives them: a rectangle (square or
  // rounded corners: too alike to tell concepts apart) or a pill (states,
  // capabilities, missions). Two concepts of different silhouettes are told
  // apart without their color. `function` is one kind on three layers:
  // its family is the layer's.
  const CONCEPTS = [
    { key: 'operational_entity', label: 'operational entity or actor', shape: 'rect', kinds: ['operational_entity', 'operational_actor', 'lifeline'] },
    { key: 'operational_activity', label: 'operational activity', shape: 'rect', kinds: ['operational_activity', 'operational_process'] },
    { key: 'operational_capability', label: 'operational capability', shape: 'pill', kinds: ['operational_capability'] },
    { key: 'system_actor', label: 'actor', shape: 'rect', kinds: ['system_actor'] },
    { key: 'system_component', label: 'system component', shape: 'rect', kinds: ['system', 'system_component', 'sysml_part'] },
    { key: 'system_function', label: 'system function', shape: 'rect', kinds: ['sysml_action'] },
    { key: 'logical_component', label: 'logical component', shape: 'rect', kinds: ['logical_component'] },
    { key: 'logical_function', label: 'logical function', shape: 'rect', kinds: [] },
    { key: 'physical_node', label: 'physical node', shape: 'rect', kinds: ['physical_node', 'hardware_component'] },
    { key: 'behaviour_component', label: 'behavior component', shape: 'rect', kinds: ['behavior_component', 'deployed_component'] },
    { key: 'physical_function', label: 'physical function', shape: 'rect', kinds: [] },
    { key: 'capability', label: 'capability', shape: 'pill', kinds: ['capability', 'capability_realization'] },
    { key: 'mission', label: 'mission', shape: 'pill', kinds: ['mission'] },
    { key: 'functional_chain', label: 'functional chain', shape: 'rect', kinds: ['functional_chain'] },
    { key: 'data_type', label: 'class or data type', shape: 'rect', kinds: ['class', 'enumeration', 'data_type', 'value_property', 'sysml_definition'] },
    { key: 'exchange_item', label: 'exchange item', shape: 'rect', kinds: ['exchange_item'] },
    { key: 'state_mode', label: 'state or mode', shape: 'pill', kinds: ['state', 'mode'] },
    { key: 'requirement', label: 'requirement', shape: 'rect', kinds: ['requirement'] },
    { key: 'test_case', label: 'test case', shape: 'rect', kinds: ['test_case'] },
    { key: 'constraint', label: 'constraint', shape: 'pill', kinds: ['constraint'] },
    { key: 'configuration_item', label: 'configuration item', shape: 'rect', kinds: ['configuration_item'] },
  ];
  const FUNCTION_OF_VIEW = { oab: 'operational_activity', sab: 'system_function', lab: 'logical_function', pab: 'physical_function' };

  const EXCHANGES = [
    { key: 'functional_exchange', label: 'functional exchange', kinds: ['functional_exchange', 'flow'] },
    { key: 'component_exchange', label: 'component exchange', kinds: ['component_exchange', 'connection'] },
    { key: 'physical_exchange', label: 'physical exchange', kinds: ['physical_exchange'] },
    { key: 'interaction', label: 'interaction', kinds: ['interaction'] },
    { key: 'communication_mean', label: 'communication mean', kinds: ['communication_mean'] },
    { key: 'physical_link', label: 'physical link', kinds: ['physical_link'] },
    { key: 'physical_path', label: 'physical path', kinds: ['physical_path'] },
    { key: 'transition', label: 'transition', kinds: ['transition'] },
    { key: 'message', label: 'message', kinds: ['message'] },
    { key: 'trace', label: 'trace', kinds: ['realization', 'trace'] },
    { key: 'relation', label: 'relation', kinds: ['exploitation', 'involvement', 'capability_association', 'generalization', 'association', 'item_element', 'breakdown', 'containment', 'binding', 'typing', 'composition', 'succession'] },
  ];

  // One concept on several layers: one color.
  const SAME_CONCEPT = [
    ['system_function', 'logical_function', 'physical_function'],
    ['system_component', 'logical_component', 'behaviour_component'],
  ];

  // What each view draws together (diagram::ViewKind).
  const VIEWS = {
    oab: { name: 'Operational', concepts: ['operational_entity', 'operational_activity'], exchanges: ['interaction', 'communication_mean'] },
    sab: { name: 'System', concepts: ['system_actor', 'system_component', 'system_function'], exchanges: ['functional_exchange', 'component_exchange', 'trace'] },
    lab: { name: 'Logical', concepts: ['system_actor', 'logical_component', 'logical_function'], exchanges: ['functional_exchange', 'component_exchange', 'trace'] },
    pab: { name: 'Physical', concepts: ['system_actor', 'physical_node', 'behaviour_component', 'physical_function'], exchanges: ['functional_exchange', 'component_exchange', 'physical_exchange', 'physical_link', 'physical_path', 'trace'] },
    msm: { name: 'Modes and states', concepts: ['state_mode'], exchanges: ['transition'] },
    cap: { name: 'Capabilities', concepts: ['mission', 'capability', 'operational_capability', 'system_actor', 'operational_entity', 'system_function', 'functional_chain'], exchanges: ['relation', 'trace'] },
    cdb: { name: 'Data', concepts: ['data_type', 'exchange_item'], exchanges: ['relation'] },
    pbs: { name: 'Product breakdown', concepts: ['configuration_item', 'requirement'], exchanges: ['relation'] },
    req: { name: 'Requirements', concepts: ['requirement', 'test_case'], exchanges: ['trace', 'relation'] },
    par: { name: 'Parametric', concepts: ['constraint', 'data_type'], exchanges: ['relation'] },
    gv: { name: 'General View', concepts: ['data_type', 'system_component'], exchanges: ['relation'] },
    iv: { name: 'Interconnection View', concepts: ['system_component'], exchanges: ['component_exchange'] },
    afv: { name: 'Action Flow View', concepts: ['system_function'], exchanges: ['functional_exchange'] },
  };

  // Colors the tool gives a meaning of its own (VS Code's light defaults).
  // An element kind colored with one would read as an error, a warning,
  // the selection or an AI proposal. Lines keep only the last two: a red
  // physical link is Capella's own code, and nothing marks a line as an error.
  const RESERVED = [
    { name: 'error red', color: '#E51400', lines: false },
    { name: 'warning amber', color: '#BF8803', lines: false },
    { name: 'selection accent', color: '#0B5CAD', lines: true },
    { name: 'AI-proposal purple', color: '#8A5CF5', lines: true },
  ];

  const LIMITS = {
    distinct: 10, // CIEDE2000 between two concepts drawn together
    reserved: 10, // CIEDE2000 to a reserved color
    cvdError: 5,
    cvdWarning: 8,
    grayError: 5, // L* points in a grayscale print
    grayWarning: 8,
    text: 4.5, // WCAG AA, normal text
    graphics: 3, // WCAG 1.4.11, lines and outlines on the sheet
    fills: 16,
    fillsWarning: 13,
    lines: 10,
    sameColour: 2, // CIEDE2000 under which two colors are the same
  };
  const STYLES = { solid: null, dashed: '6 3', dotted: '2 2.5' };

  const conceptByKey = new Map(CONCEPTS.map((c) => [c.key, c]));
  const exchangeByKey = new Map(EXCHANGES.map((e) => [e.key, e]));
  const conceptOfKind = new Map(CONCEPTS.flatMap((c) => c.kinds.map((kind) => [kind, c.key])));
  const exchangeOfKind = new Map(EXCHANGES.flatMap((e) => e.kinds.map((kind) => [kind, e.key])));
  const presetById = new Map(PRESETS.map((preset) => [preset.id, preset]));

  /** The concept family that colors a box of `kind` on `view`; null for notation marks. */
  function conceptFor(kind, view) {
    if (kind === 'function') return FUNCTION_OF_VIEW[view] || 'system_function';
    return conceptOfKind.get(kind) || null;
  }

  function exchangeFor(kind) {
    return exchangeOfKind.get(kind) || null;
  }

  const clone = (value) => JSON.parse(JSON.stringify(value));
  const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

  /** `base` with `over` on top, one level into concepts, exchanges and markings. */
  function merge(base, over) {
    const out = { ...clone(base), ...clone(over) };
    for (const part of ['concepts', 'exchanges', 'markings']) {
      const merged = { ...((base && base[part]) || {}) };
      for (const [key, value] of Object.entries((over && over[part]) || {})) merged[key] = { ...(merged[key] || {}), ...value };
      out[part] = clone(merged);
    }
    return out;
  }

  /** The theme as drawn: its `extends` preset under it. */
  function complete(theme) {
    const base = theme.extends ? presetById.get(theme.extends) : null;
    const full = base ? merge(base, theme) : clone(theme);
    // A dark variant of its own completes the preset's, entry by entry.
    if (base && isObject(base.dark) && isObject(theme.dark)) full.dark = merge(base.dark, theme.dark);
    delete full.extends;
    return full;
  }

  /** The dark variant of a complete theme, complete itself; null without one. */
  function darkOf(full) {
    if (!isObject(full.dark)) return null;
    const { dark, ...light } = full;
    const out = merge(light, dark);
    delete out.dark;
    return out;
  }

  // --- validation ---------------------------------------------------------
  const round = (value) => Math.round(value * 10) / 10;
  const pretty = (key) => (conceptByKey.get(key) || exchangeByKey.get(key) || { label: key }).label;

  function checkStructure(theme, report) {
    if (!isObject(theme)) return report.error('structure', 'a theme is a JSON object');
    if (theme.version !== VERSION) report.error('structure', `version must be ${VERSION} (found ${JSON.stringify(theme.version)})`);
    if (typeof theme.name !== 'string' || !theme.name.trim() || theme.name.length > 80) report.error('structure', 'name must be a text of 1 to 80 characters');
    if (theme.id !== undefined && (typeof theme.id !== 'string' || !/^[a-z0-9][a-z0-9-]{0,63}$/.test(theme.id))) report.error('structure', 'id must be lower-case letters, digits and dashes');
    if (theme.extends !== undefined && !presetById.has(theme.extends)) report.error('structure', `extends names no preset: "${theme.extends}" (presets: ${[...presetById.keys()].join(', ')})`);
    const allowed = new Set(['version', 'id', 'name', 'description', 'source', 'extends', 'background', 'text', 'muted', 'concepts', 'exchanges', 'markings', 'dark']);
    for (const key of Object.keys(theme)) if (!allowed.has(key)) report.error('structure', `unknown field "${key}"`);
    checkPalette(theme, '', report);
    if (theme.dark !== undefined) {
      if (!isObject(theme.dark)) report.error('structure', 'dark must be an object: the dark variant');
      else checkPalette(theme.dark, 'dark.', report);
    }
  }

  function checkColour(value, where, report) {
    if (value === undefined) return;
    if (!Color.parse(value)) report.error('structure', `${where} is not a color: ${JSON.stringify(value)} (write #RRGGBB)`);
  }

  function checkPalette(part, prefix, report) {
    for (const key of ['background', 'text', 'muted']) checkColour(part[key], `${prefix}${key}`, report);
    for (const [section, known, fields] of [['concepts', conceptByKey, ['fill', 'stroke', 'text', 'border']], ['exchanges', exchangeByKey, ['line', 'width', 'style', 'text']]]) {
      const entries = part[section];
      if (entries === undefined) continue;
      if (!isObject(entries)) { report.error('structure', `${prefix}${section} must be an object`); continue; }
      for (const [key, value] of Object.entries(entries)) {
        const where = `${prefix}${section}.${key}`;
        if (!known.has(key)) { report.error('structure', `unknown ${section === 'concepts' ? 'concept' : 'exchange'} "${key}" in ${prefix}${section}`); continue; }
        if (!isObject(value)) { report.error('structure', `${where} must be an object`); continue; }
        for (const field of Object.keys(value)) if (!fields.includes(field)) report.error('structure', `unknown field "${field}" in ${where}`);
        for (const field of ['fill', 'stroke', 'text', 'line']) checkColour(value[field], `${where}.${field}`, report);
        for (const field of ['style', 'border']) if (value[field] !== undefined && !(value[field] in STYLES)) report.error('structure', `${where}.${field} must be solid, dashed or dotted`);
        if (value.width !== undefined && !(typeof value.width === 'number' && value.width >= 0.5 && value.width <= 6)) report.error('structure', `${where}.width must be a number from 0.5 to 6`);
      }
    }
    const markings = part.markings;
    if (markings !== undefined) {
      if (!isObject(markings)) report.error('structure', `${prefix}markings must be an object`);
      else {
        for (const key of Object.keys(markings)) if (key !== 'variant' && key !== 'tag') report.error('structure', `unknown marking "${key}" (variant, tag)`);
        if (markings.variant) {
          checkColour(markings.variant.stroke, `${prefix}markings.variant.stroke`, report);
          if (markings.variant.style !== undefined && !(markings.variant.style in STYLES)) report.error('structure', `${prefix}markings.variant.style must be solid, dashed or dotted`);
        }
        if (markings.tag) for (const field of ['fill', 'stroke', 'text']) checkColour(markings.tag[field], `${prefix}markings.tag.${field}`, report);
      }
    }
  }

  function checkComplete(full, prefix, report) {
    for (const key of ['background', 'text', 'muted']) if (!full[key]) report.error('complete', `${prefix}${key} is missing`);
    for (const concept of CONCEPTS) {
      const entry = (full.concepts || {})[concept.key];
      if (!entry || !entry.fill || !entry.stroke) report.error('complete', `${prefix}${concept.label} has no fill and stroke (concepts.${concept.key})`);
    }
    for (const exchange of EXCHANGES) {
      const entry = (full.exchanges || {})[exchange.key];
      if (!entry || !entry.line) report.error('complete', `${prefix}${exchange.label} has no line color (exchanges.${exchange.key})`);
    }
  }

  const rgb = (value) => Color.parse(value);
  const textOf = (full, key) => full.concepts[key].text || full.text;
  const lineText = (full, key) => full.exchanges[key].text || full.exchanges[key].line;
  const borderOf = (full, key) => full.concepts[key].border || 'solid';
  const styleOf = (full, key) => full.exchanges[key].style || 'solid';
  const widthOf = (full, key) => full.exchanges[key].width || 1.4;
  const sameGroup = (a, b) => SAME_CONCEPT.some((group) => group.includes(a) && group.includes(b));

  function checkRules(full, prefix, report) {
    const bg = rgb(full.background);
    // One concept, one color on every layer.
    for (const group of SAME_CONCEPT) {
      const [first, ...rest] = group;
      for (const other of rest) {
        for (const field of ['fill', 'stroke']) {
          const d = Color.deltaE(rgb(full.concepts[first][field]), rgb(full.concepts[other][field]));
          if (d >= LIMITS.sameColour) {
            report.error('same-concept', `${prefix}${pretty(other)} and ${pretty(first)} are one concept on two layers: give them one ${field} (${full.concepts[other][field]} vs ${full.concepts[first][field]})`);
          }
        }
      }
    }
    // Reserved colors.
    for (const concept of CONCEPTS) {
      for (const field of ['fill', 'stroke']) {
        const value = full.concepts[concept.key][field];
        for (const reserved of RESERVED) {
          const d = Color.deltaE(rgb(value), rgb(reserved.color));
          if (d < LIMITS.reserved) report.error('reserved', `${prefix}${concept.label} ${field} ${value} is the ${reserved.name} the tool keeps for its own meaning (ΔE ${round(d)}); choose another color`);
        }
      }
    }
    for (const exchange of EXCHANGES) {
      const value = full.exchanges[exchange.key].line;
      for (const reserved of RESERVED.filter((r) => r.lines)) {
        const d = Color.deltaE(rgb(value), rgb(reserved.color));
        if (d < LIMITS.reserved) report.error('reserved', `${prefix}${exchange.label} line ${value} is the ${reserved.name} the tool keeps for its own meaning (ΔE ${round(d)}); choose another color`);
      }
    }
    // Contrast.
    const textBg = Color.contrast(rgb(full.text), bg);
    if (textBg < LIMITS.text) report.error('contrast', `${prefix}text on the background reaches ${round(textBg)}:1, under ${LIMITS.text}:1 (WCAG AA)`);
    for (const concept of CONCEPTS) {
      const entry = full.concepts[concept.key];
      const fill = rgb(entry.fill);
      const text = Color.contrast(rgb(textOf(full, concept.key)), fill);
      if (text < LIMITS.text) report.error('contrast', `${prefix}${concept.label}: text on its fill reaches ${round(text)}:1, under ${LIMITS.text}:1 (WCAG AA); darken the text or lighten the fill`);
      const muted = Color.contrast(rgb(full.muted), fill);
      if (muted < LIMITS.text) report.error('contrast', `${prefix}${concept.label}: the secondary text on its fill reaches ${round(muted)}:1, under ${LIMITS.text}:1 (WCAG AA)`);
      const outline = Math.max(Color.contrast(rgb(entry.stroke), bg), Color.contrast(fill, bg));
      if (outline < LIMITS.graphics) report.error('contrast', `${prefix}${concept.label}: neither its outline nor its fill stands out from the background (${round(outline)}:1, under ${LIMITS.graphics}:1)`);
    }
    for (const exchange of EXCHANGES) {
      const line = Color.contrast(rgb(full.exchanges[exchange.key].line), bg);
      if (line < LIMITS.graphics) report.error('contrast', `${prefix}${exchange.label}: the line on the background reaches ${round(line)}:1, under ${LIMITS.graphics}:1`);
      const label = Color.contrast(rgb(lineText(full, exchange.key)), bg);
      if (label < LIMITS.text) report.error('contrast', `${prefix}${exchange.label}: its label on the background reaches ${round(label)}:1, under ${LIMITS.text}:1 (WCAG AA); give it a darker text`);
    }
    // What a view draws together.
    for (const [view, spec] of Object.entries(VIEWS)) {
      pairs(spec.concepts).forEach(([a, b]) => {
        if (sameGroup(a, b)) return;
        if (conceptByKey.get(a).shape !== conceptByKey.get(b).shape || borderOf(full, a) !== borderOf(full, b)) return;
        distinguish(prefix, spec.name, pretty(a), pretty(b), full.concepts[a].fill, full.concepts[b].fill, report);
      });
      pairs(spec.exchanges).forEach(([a, b]) => {
        if (styleOf(full, a) !== styleOf(full, b) || Math.abs(widthOf(full, a) - widthOf(full, b)) >= 1) return;
        distinguish(prefix, spec.name, pretty(a), pretty(b), full.exchanges[a].line, full.exchanges[b].line, report);
      });
    }
    // One color, one concept: a color reused for another concept of the same shape on another view.
    const reported = new Set();
    for (const [a, b] of pairs(CONCEPTS.map((c) => c.key))) {
      if (sameGroup(a, b) || conceptByKey.get(a).shape !== conceptByKey.get(b).shape) continue;
      const together = Object.values(VIEWS).some((spec) => spec.concepts.includes(a) && spec.concepts.includes(b));
      if (together) continue;
      const d = Color.deltaE(rgb(full.concepts[a].fill), rgb(full.concepts[b].fill));
      if (d < LIMITS.sameColour * 2 && !reported.has(`${a}|${b}`)) {
        reported.add(`${a}|${b}`);
        report.warning('one-color', `${prefix}${pretty(a)} and ${pretty(b)} share a color (ΔE ${round(d)}): one color should mean one concept`);
      }
    }
    // A bounded palette.
    const fills = distinctColours(CONCEPTS.map((c) => full.concepts[c.key].fill));
    if (fills > LIMITS.fills) report.error('palette', `${prefix}${fills} different fills: keep to ${LIMITS.fills} or fewer, a reader learns no more`);
    else if (fills > LIMITS.fillsWarning) report.warning('palette', `${prefix}${fills} different fills: ${LIMITS.fillsWarning} or fewer read more easily`);
    const lines = distinctColours(EXCHANGES.map((e) => full.exchanges[e.key].line));
    if (lines > LIMITS.lines) report.error('palette', `${prefix}${lines} different line colors: keep to ${LIMITS.lines} or fewer, or tell lines apart by style`);
  }

  function distinguish(prefix, view, a, b, colourA, colourB, report) {
    const [x, y] = [rgb(colourA), rgb(colourB)];
    const d = Color.deltaE(x, y);
    if (d < LIMITS.distinct) {
      report.error('distinct', `${prefix}${a} and ${b} are too close on the ${view} view (ΔE ${round(d)}, under ${LIMITS.distinct}); change one of them, or their shape or line style`);
      return;
    }
    for (const kind of Color.CVD_KINDS) {
      const dc = Color.deltaE(Color.simulate(x, kind), Color.simulate(y, kind));
      if (dc < LIMITS.cvdError) report.error('cvd', `${prefix}${a} and ${b} look alike with ${kind} (ΔE ${round(dc)}) on the ${view} view; vary lightness or blue-yellow`);
      else if (dc < LIMITS.cvdWarning) report.warning('cvd', `${prefix}${a} and ${b} are close with ${kind} (ΔE ${round(dc)}) on the ${view} view`);
    }
    const dl = Math.abs(Color.lightness(x) - Color.lightness(y));
    if (dl < LIMITS.grayError) report.error('grayscale', `${prefix}${a} and ${b} are too close in grayscale (ΔL ${round(dl)}) on the ${view} view; vary lightness`);
    else if (dl < LIMITS.grayWarning) report.warning('grayscale', `${prefix}${a} and ${b} are close in grayscale (ΔL ${round(dl)}) on the ${view} view`);
  }

  function pairs(items) {
    const out = [];
    for (let i = 0; i < items.length; i += 1) for (let j = i + 1; j < items.length; j += 1) out.push([items[i], items[j]]);
    return out;
  }

  function distinctColours(values) {
    const kept = [];
    for (const value of values) if (!kept.some((other) => Color.deltaE(rgb(value), rgb(other)) < LIMITS.sameColour)) kept.push(value);
    return kept.length;
  }

  /**
   * Check a theme. Returns { ok, errors, warnings, theme } where `theme` is
   * the complete theme (its preset under it) when the structure allowed
   * completing it. Each error and warning is { rule, message }.
   */
  function validate(theme) {
    const errors = [];
    const warnings = [];
    const report = {
      error: (rule, message) => { if (!errors.some((e) => e.message === message)) errors.push({ rule, message }); },
      warning: (rule, message) => { if (!warnings.some((w) => w.message === message)) warnings.push({ rule, message }); },
    };
    checkStructure(theme, report);
    if (errors.length) return { ok: false, errors, warnings, theme: null };
    const full = complete(theme);
    checkComplete(full, '', report);
    const dark = darkOf(full);
    if (dark) checkComplete(dark, 'dark variant: ', report);
    if (errors.length) return { ok: false, errors, warnings, theme: null };
    checkRules(full, '', report);
    if (dark) checkRules(dark, 'dark variant: ', report);
    return { ok: errors.length === 0, errors, warnings, theme: full };
  }

  /** Parse and check a theme file's text. */
  function parse(text) {
    let value;
    try {
      value = JSON.parse(text);
    } catch (error) {
      return { ok: false, errors: [{ rule: 'structure', message: `not JSON: ${error.message}` }], warnings: [], theme: null };
    }
    return validate(value);
  }

  // --- drawing --------------------------------------------------------------
  /**
   * What the renderer draws with: a complete theme turned into functions of
   * the kinds it meets. `dark` picks the dark variant when the theme has one.
   * A theme that fails validation is refused here too (throws), unless
   * `draft` asks for the preview of a theme being edited: a complete one is
   * then drawn whatever its colors break, to show what is wrong.
   */
  function palette(theme, options) {
    const checked = validate(theme);
    const draft = Boolean(options && options.draft && checked.theme);
    if (!checked.ok && !draft) throw new Error(`theme refused: ${checked.errors.map((e) => e.message).join('; ')}`);
    const full = (options && options.dark && darkOf(checked.theme)) || checked.theme;
    const variant = (full.markings && full.markings.variant) || {};
    const tag = (full.markings && full.markings.tag) || {};
    return {
      id: full.id || null,
      name: full.name,
      dark: Boolean(options && options.dark && full !== checked.theme),
      background: full.background,
      text: full.text,
      muted: full.muted,
      node(kind, view) {
        const key = conceptFor(kind, view);
        const entry = key && full.concepts[key];
        if (!entry) return null;
        return { fill: entry.fill, stroke: entry.stroke, text: entry.text || full.text, dash: STYLES[entry.border || 'solid'] };
      },
      edge(kind) {
        const key = exchangeFor(kind);
        const entry = key && full.exchanges[key];
        if (!entry) return null;
        return { stroke: entry.line, text: entry.text || entry.line, width: entry.width, dash: entry.style ? STYLES[entry.style] : undefined };
      },
      variant: { dash: STYLES[variant.style || 'dashed'] || '5 3', stroke: variant.stroke || null },
      tag: { fill: tag.fill || full.background, stroke: tag.stroke || null, text: tag.text || null },
    };
  }

  /** A preset by id, as a fresh copy; null when there is none. */
  function preset(id) {
    const found = presetById.get(id);
    return found ? clone(found) : null;
  }

  /** A preset's copy under a new name, to edit: a custom theme of its own. */
  function duplicate(theme, name) {
    const copy = clone(complete(theme));
    copy.name = name;
    copy.id = slug(name);
    delete copy.source;
    return copy;
  }

  function slug(text) {
    return String(text).toLowerCase().normalize('NFKD').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 64) || 'theme';
  }

  return {
    VERSION,
    CONCEPTS,
    EXCHANGES,
    VIEWS,
    SAME_CONCEPT,
    RESERVED,
    LIMITS,
    presets: () => PRESETS.map((p) => ({ id: p.id, name: p.name, description: p.description || '' })),
    preset,
    validate,
    parse,
    palette,
    complete,
    darkOf,
    duplicate,
    slug,
    conceptFor,
    exchangeFor,
  };
});
