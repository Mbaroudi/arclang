/*
 * ArcViz color science — what the theme validator measures colors with.
 *
 * Pure functions on `#RRGGBB` strings: CIE L*a*b* (sRGB, D65), the CIEDE2000
 * color difference, the WCAG 2.x contrast ratio, color-vision-deficiency
 * simulation (Machado, Oliveira and Fernandes 2009, severity 1) and the
 * lightness a grayscale print keeps (CIE L*). Kept in step with
 * `src/compiler/diagram/theme.rs`, which mirrors it for the CLI.
 */
(function (root, factory) {
  if (typeof module === 'object' && module.exports) module.exports = factory();
  else root.ArcVizColor = factory();
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  const HEX = /^#([0-9a-f]{6})$/i;

  /** `#RRGGBB` (or `#RGB`) to [r, g, b] in 0..255; null when not a color. */
  function parse(text) {
    if (typeof text !== 'string') return null;
    const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/i.exec(text.trim());
    const value = short ? `#${short[1]}${short[1]}${short[2]}${short[2]}${short[3]}${short[3]}` : text.trim();
    const match = HEX.exec(value);
    if (!match) return null;
    const n = parseInt(match[1], 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
  }

  function hex(rgb) {
    return `#${rgb.map((c) => Math.max(0, Math.min(255, Math.round(c))).toString(16).padStart(2, '0')).join('').toUpperCase()}`;
  }

  const toLinear = (c) => {
    const v = c / 255;
    return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  const fromLinear = (v) => 255 * (v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055);

  /** WCAG relative luminance, 0..1. */
  function luminance(rgb) {
    const [r, g, b] = rgb.map(toLinear);
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  }

  /** WCAG contrast ratio of two colors, 1..21. */
  function contrast(a, b) {
    const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
    return (hi + 0.05) / (lo + 0.05);
  }

  /** CIE L*a*b* (D65) of an sRGB color. */
  function lab(rgb) {
    const [r, g, b] = rgb.map(toLinear);
    const x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047;
    const y = 0.2126729 * r + 0.7151522 * g + 0.072175 * b;
    const z = (0.0193339 * r + 0.119192 * g + 0.9503041 * b) / 1.08883;
    const f = (t) => (t > 216 / 24389 ? Math.cbrt(t) : (24389 / 27 * t + 16) / 116);
    const [fx, fy, fz] = [f(x), f(y), f(z)];
    return [116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz)];
  }

  /** CIE L* alone: what a grayscale print keeps of a color. */
  function lightness(rgb) {
    return lab(rgb)[0];
  }

  /** CIEDE2000 color difference of two sRGB colors (Sharma, Wu, Dalal 2005). */
  function deltaE(a, b) {
    return deltaELab(lab(a), lab(b));
  }

  /** CIEDE2000 of two L*a*b* triples. */
  function deltaELab([L1, a1, b1], [L2, a2, b2]) {
    const rad = Math.PI / 180;
    const C1 = Math.hypot(a1, b1);
    const C2 = Math.hypot(a2, b2);
    const Cm = (C1 + C2) / 2;
    const G = 0.5 * (1 - Math.sqrt(Cm ** 7 / (Cm ** 7 + 25 ** 7)));
    const a1p = (1 + G) * a1;
    const a2p = (1 + G) * a2;
    const C1p = Math.hypot(a1p, b1);
    const C2p = Math.hypot(a2p, b2);
    const hue = (x, y) => {
      if (x === 0 && y === 0) return 0;
      const h = Math.atan2(y, x) / rad;
      return h < 0 ? h + 360 : h;
    };
    const h1p = hue(a1p, b1);
    const h2p = hue(a2p, b2);
    const dLp = L2 - L1;
    const dCp = C2p - C1p;
    let dhp = 0;
    if (C1p * C2p !== 0) {
      dhp = h2p - h1p;
      if (dhp > 180) dhp -= 360;
      else if (dhp < -180) dhp += 360;
    }
    const dHp = 2 * Math.sqrt(C1p * C2p) * Math.sin((dhp / 2) * rad);
    const Lpm = (L1 + L2) / 2;
    const Cpm = (C1p + C2p) / 2;
    let hpm = h1p + h2p;
    if (C1p * C2p !== 0) {
      if (Math.abs(h1p - h2p) <= 180) hpm = (h1p + h2p) / 2;
      else hpm = h1p + h2p < 360 ? (h1p + h2p + 360) / 2 : (h1p + h2p - 360) / 2;
    }
    const T = 1 - 0.17 * Math.cos((hpm - 30) * rad) + 0.24 * Math.cos(2 * hpm * rad)
      + 0.32 * Math.cos((3 * hpm + 6) * rad) - 0.2 * Math.cos((4 * hpm - 63) * rad);
    const dTheta = 30 * Math.exp(-(((hpm - 275) / 25) ** 2));
    const Rc = 2 * Math.sqrt(Cpm ** 7 / (Cpm ** 7 + 25 ** 7));
    const Sl = 1 + (0.015 * (Lpm - 50) ** 2) / Math.sqrt(20 + (Lpm - 50) ** 2);
    const Sc = 1 + 0.045 * Cpm;
    const Sh = 1 + 0.015 * Cpm * T;
    const Rt = -Math.sin(2 * dTheta * rad) * Rc;
    return Math.sqrt((dLp / Sl) ** 2 + (dCp / Sc) ** 2 + (dHp / Sh) ** 2 + Rt * (dCp / Sc) * (dHp / Sh));
  }

  // Machado, Oliveira, Fernandes (2009), severity 1.0, on linear RGB.
  const CVD = {
    protanopia: [0.152286, 1.052583, -0.204868, 0.114503, 0.786281, 0.099216, -0.003882, -0.048116, 1.051998],
    deuteranopia: [0.367322, 0.860646, -0.227968, 0.280085, 0.672501, 0.047413, -0.01182, 0.04294, 0.968881],
  };

  /** How a color looks to a reader with protanopia or deuteranopia. */
  function simulate(rgb, kind) {
    const m = CVD[kind];
    const [r, g, b] = rgb.map(toLinear);
    const clamp = (v) => Math.max(0, Math.min(1, v));
    return [
      fromLinear(clamp(m[0] * r + m[1] * g + m[2] * b)),
      fromLinear(clamp(m[3] * r + m[4] * g + m[5] * b)),
      fromLinear(clamp(m[6] * r + m[7] * g + m[8] * b)),
    ];
  }

  return { parse, hex, luminance, contrast, lab, lightness, deltaE, deltaELab, simulate, CVD_KINDS: Object.keys(CVD) };
});
