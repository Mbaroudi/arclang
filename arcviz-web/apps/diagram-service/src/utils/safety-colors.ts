/**
 * Safety-Critical Color System
 * 
 * Implements LaTeX Specification Section 3 (Color Code Specification)
 * Safety Integrity Level (SIL) Color Overlays - page 10
 * 
 * Standards supported:
 * - ISO 26262 (Automotive): ASIL A-D
 * - DO-178C (Aerospace): DAL A-D
 * - IEC 61508 (Industrial): SIL 1-4
 */

export type SafetyStandard = 'ISO26262' | 'DO178C' | 'IEC61508';

export type ASILLevel = 'QM' | 'ASIL_A' | 'ASIL_B' | 'ASIL_C' | 'ASIL_D';
export type DALLevel = 'DAL_E' | 'DAL_D' | 'DAL_C' | 'DAL_B' | 'DAL_A';
export type SILLevel = 'SIL_0' | 'SIL_1' | 'SIL_2' | 'SIL_3' | 'SIL_4';

export type SafetyLevel = ASILLevel | DALLevel | SILLevel;

export interface SafetyColorConfig {
  borderColor: string;
  borderWidth: number;
  badgeColor: string;
  badgeBackground: string;
  glowColor?: string;
  criticality: 'none' | 'low' | 'medium' | 'high' | 'critical';
}

/**
 * ISO 26262 (Automotive Safety Integrity Levels)
 * CORRECTED per Capella specifications Table 6
 */
const ASIL_COLORS: Record<ASILLevel, SafetyColorConfig> = {
  QM: {
    borderColor: '#808080',    // Gray - Quality Managed (non-safety)
    borderWidth: 2,
    badgeColor: '#FFFFFF',
    badgeBackground: '#808080',
    criticality: 'none',
  },
  ASIL_A: {
    borderColor: '#FFD700',    // Gold - Lowest automotive safety level
    borderWidth: 3,
    badgeColor: '#000000',
    badgeBackground: '#FFD700',
    glowColor: 'rgba(255, 215, 0, 0.3)',
    criticality: 'low',
  },
  ASIL_B: {
    borderColor: '#FF8C00',    // Dark Orange - Medium automotive safety
    borderWidth: 4,
    badgeColor: '#FFFFFF',
    badgeBackground: '#FF8C00',
    glowColor: 'rgba(255, 140, 0, 0.4)',
    criticality: 'medium',
  },
  ASIL_C: {
    borderColor: '#DC143C',    // Crimson - High automotive safety
    borderWidth: 5,
    badgeColor: '#FFFFFF',
    badgeBackground: '#DC143C',
    glowColor: 'rgba(220, 20, 60, 0.5)',
    criticality: 'high',
  },
  ASIL_D: {
    borderColor: '#8B0000',    // Dark Red - Highest automotive safety (CRITICAL)
    borderWidth: 6,
    badgeColor: '#FFFFFF',
    badgeBackground: '#8B0000',
    glowColor: 'rgba(139, 0, 0, 0.6)',
    criticality: 'critical',
  },
};

/**
 * DO-178C (Aerospace Design Assurance Levels)
 * CORRECTED per Capella specifications Table 6
 */
const DAL_COLORS: Record<DALLevel, SafetyColorConfig> = {
  DAL_E: {
    borderColor: '#808080',    // Gray - No safety impact
    borderWidth: 2,
    badgeColor: '#FFFFFF',
    badgeBackground: '#808080',
    criticality: 'none',
  },
  DAL_D: {
    borderColor: '#FFD700',    // Gold - Lowest aerospace assurance level
    borderWidth: 3,
    badgeColor: '#000000',
    badgeBackground: '#FFD700',
    glowColor: 'rgba(255, 215, 0, 0.3)',
    criticality: 'low',
  },
  DAL_C: {
    borderColor: '#FF8C00',    // Dark Orange - Medium aerospace assurance
    borderWidth: 4,
    badgeColor: '#FFFFFF',
    badgeBackground: '#FF8C00',
    glowColor: 'rgba(255, 140, 0, 0.4)',
    criticality: 'medium',
  },
  DAL_B: {
    borderColor: '#DC143C',    // Crimson - High aerospace assurance
    borderWidth: 5,
    badgeColor: '#FFFFFF',
    badgeBackground: '#DC143C',
    glowColor: 'rgba(220, 20, 60, 0.5)',
    criticality: 'high',
  },
  DAL_A: {
    borderColor: '#8B0000',    // Dark Red - Highest aerospace assurance (CRITICAL)
    borderWidth: 6,
    badgeColor: '#FFFFFF',
    badgeBackground: '#8B0000',
    glowColor: 'rgba(139, 0, 0, 0.6)',
    criticality: 'critical',
  },
};

/**
 * IEC 61508 (Industrial Safety Integrity Levels)
 * CORRECTED per Capella specifications Table 6
 */
const SIL_COLORS: Record<SILLevel, SafetyColorConfig> = {
  SIL_0: {
    borderColor: '#808080',    // Gray - No safety requirements
    borderWidth: 2,
    badgeColor: '#FFFFFF',
    badgeBackground: '#808080',
    criticality: 'none',
  },
  SIL_1: {
    borderColor: '#FFD700',    // Gold - Lowest industrial safety level
    borderWidth: 3,
    badgeColor: '#000000',
    badgeBackground: '#FFD700',
    glowColor: 'rgba(255, 215, 0, 0.3)',
    criticality: 'low',
  },
  SIL_2: {
    borderColor: '#FF8C00',    // Dark Orange - Medium industrial safety
    borderWidth: 4,
    badgeColor: '#FFFFFF',
    badgeBackground: '#FF8C00',
    glowColor: 'rgba(255, 140, 0, 0.4)',
    criticality: 'medium',
  },
  SIL_3: {
    borderColor: '#DC143C',    // Crimson - High industrial safety
    borderWidth: 5,
    badgeColor: '#FFFFFF',
    badgeBackground: '#DC143C',
    glowColor: 'rgba(220, 20, 60, 0.5)',
    criticality: 'high',
  },
  SIL_4: {
    borderColor: '#8B0000',    // Dark Red - Highest industrial safety (CRITICAL)
    borderWidth: 6,
    badgeColor: '#FFFFFF',
    badgeBackground: '#8B0000',
    glowColor: 'rgba(139, 0, 0, 0.6)',
    criticality: 'critical',
  },
};

/**
 * Get safety color configuration for a given safety level
 */
export function getSafetyColorConfig(
  safetyLevel: SafetyLevel,
  standard?: SafetyStandard
): SafetyColorConfig {
  // Auto-detect standard if not provided
  if (!standard) {
    if (safetyLevel.startsWith('ASIL') || safetyLevel === 'QM') {
      standard = 'ISO26262';
    } else if (safetyLevel.startsWith('DAL')) {
      standard = 'DO178C';
    } else if (safetyLevel.startsWith('SIL')) {
      standard = 'IEC61508';
    }
  }

  switch (standard) {
    case 'ISO26262':
      return ASIL_COLORS[safetyLevel as ASILLevel] || ASIL_COLORS.QM;
    case 'DO178C':
      return DAL_COLORS[safetyLevel as DALLevel] || DAL_COLORS.DAL_E;
    case 'IEC61508':
      return SIL_COLORS[safetyLevel as SILLevel] || SIL_COLORS.SIL_0;
    default:
      return ASIL_COLORS.QM;
  }
}

/**
 * Parse safety level from metadata
 */
export function parseSafetyLevel(metadata: any): {
  level: SafetyLevel | null;
  standard: SafetyStandard | null;
} {
  if (!metadata) {
    return { level: null, standard: null };
  }

  let safetyLevel = metadata.safety_level || metadata.safetyLevel || metadata.asil || metadata.dal || metadata.sil;
  
  // Handle ArcLang attribute wrapper: { String: "ASIL_D" }
  if (safetyLevel && typeof safetyLevel === 'object' && safetyLevel.String) {
    safetyLevel = safetyLevel.String;
  }
  
  if (!safetyLevel) {
    return { level: null, standard: null };
  }

  const levelStr = String(safetyLevel).toUpperCase();

  if (levelStr.includes('ASIL') || levelStr === 'QM') {
    return {
      level: levelStr.replace('-', '_').replace(' ', '_') as ASILLevel,
      standard: 'ISO26262',
    };
  } else if (levelStr.includes('DAL')) {
    return {
      level: levelStr.replace('-', '_').replace(' ', '_') as DALLevel,
      standard: 'DO178C',
    };
  } else if (levelStr.includes('SIL')) {
    return {
      level: levelStr.replace('-', '_').replace(' ', '_') as SILLevel,
      standard: 'IEC61508',
    };
  }

  return { level: null, standard: null };
}

/**
 * Generate SVG attributes for safety-critical border
 */
export function getSafetyBorderAttributes(
  safetyLevel: SafetyLevel,
  standard?: SafetyStandard
): Record<string, string> {
  const config = getSafetyColorConfig(safetyLevel, standard);
  
  const attrs: Record<string, string> = {
    'stroke': config.borderColor,
    'stroke-width': String(config.borderWidth),
  };

  if (config.glowColor) {
    attrs['filter'] = `drop-shadow(0 0 ${config.borderWidth * 2}px ${config.glowColor})`;
  }

  return attrs;
}

/**
 * Generate SVG badge element for safety level
 */
export function createSafetyBadge(
  x: number,
  y: number,
  safetyLevel: SafetyLevel,
  standard?: SafetyStandard,
  size: 'small' | 'medium' | 'large' = 'medium'
): string {
  const config = getSafetyColorConfig(safetyLevel, standard);
  
  const sizes = {
    small: { width: 50, height: 18, fontSize: 10 },
    medium: { width: 70, height: 24, fontSize: 12 },
    large: { width: 90, height: 30, fontSize: 14 },
  };

  const dims = sizes[size];
  const label = safetyLevel.replace('_', ' ');

  return `
    <g class="safety-badge">
      <rect 
        x="${x}" 
        y="${y}" 
        width="${dims.width}" 
        height="${dims.height}" 
        rx="4" 
        ry="4"
        fill="${config.badgeBackground}"
        stroke="${config.borderColor}"
        stroke-width="2"
        filter="drop-shadow(0 2px 4px rgba(0,0,0,0.2))"
      />
      <text 
        x="${x + dims.width / 2}" 
        y="${y + dims.height / 2}" 
        text-anchor="middle" 
        dominant-baseline="middle"
        font-family="Arial, sans-serif"
        font-size="${dims.fontSize}"
        font-weight="bold"
        fill="${config.badgeColor}"
      >${label}</text>
    </g>
  `;
}

/**
 * Get criticality icon for safety level
 */
export function getCriticalityIcon(criticality: SafetyColorConfig['criticality']): string {
  const icons: Record<typeof criticality, string> = {
    none: '○',
    low: '△',
    medium: '◇',
    high: '▲',
    critical: '⚠',
  };
  return icons[criticality];
}

/**
 * Generate safety legend for diagram
 */
export function createSafetyLegend(
  x: number,
  y: number,
  standard: SafetyStandard
): string {
  const levels: SafetyLevel[] =
    standard === 'ISO26262'
      ? ['QM', 'ASIL_A', 'ASIL_B', 'ASIL_C', 'ASIL_D']
      : standard === 'DO178C'
      ? ['DAL_E', 'DAL_D', 'DAL_C', 'DAL_B', 'DAL_A']
      : ['SIL_0', 'SIL_1', 'SIL_2', 'SIL_3', 'SIL_4'];

  const standardName =
    standard === 'ISO26262' ? 'ISO 26262' : standard === 'DO178C' ? 'DO-178C' : 'IEC 61508';

  let svg = `<g class="safety-legend">`;
  
  svg += `<rect x="${x}" y="${y}" width="200" height="${levels.length * 30 + 50}" rx="8" fill="white" stroke="#CCC" stroke-width="2" opacity="0.95"/>`;
  
  svg += `<text x="${x + 100}" y="${y + 25}" text-anchor="middle" font-family="Arial" font-size="14" font-weight="bold" fill="#333">${standardName} Levels</text>`;

  levels.forEach((level, index) => {
    const config = getSafetyColorConfig(level, standard);
    const levelY = y + 45 + index * 30;
    const label = level.replace('_', ' ');
    const icon = getCriticalityIcon(config.criticality);

    svg += `<rect x="${x + 10}" y="${levelY}" width="30" height="20" rx="3" fill="${config.badgeBackground}" stroke="${config.borderColor}" stroke-width="${config.borderWidth}"/>`;
    
    svg += `<text x="${x + 50}" y="${levelY + 10}" dominant-baseline="middle" font-family="Arial" font-size="11" fill="#333">${icon} ${label}</text>`;
  });

  svg += `</g>`;
  
  return svg;
}

/**
 * Apply safety styling to SVG element
 */
export function applySafetyStyling(
  svgElement: string,
  safetyLevel: SafetyLevel | null,
  standard?: SafetyStandard
): string {
  if (!safetyLevel) return svgElement;

  const config = getSafetyColorConfig(safetyLevel, standard);

  if (svgElement.includes('<rect') || svgElement.includes('<path')) {
    svgElement = svgElement.replace(
      /stroke="[^"]*"/,
      `stroke="${config.borderColor}"`
    );
    svgElement = svgElement.replace(
      /stroke-width="[^"]*"/,
      `stroke-width="${config.borderWidth}"`
    );

    if (config.glowColor && !svgElement.includes('filter=')) {
      svgElement = svgElement.replace(
        '/>',
        ` filter="drop-shadow(0 0 ${config.borderWidth * 2}px ${config.glowColor})"/>`
      );
    }
  }

  return svgElement;
}

/**
 * Check if component is safety-critical
 */
export function isSafetyCritical(metadata: any): boolean {
  const { level } = parseSafetyLevel(metadata);
  if (!level) return false;

  const criticalLevels: SafetyLevel[] = ['ASIL_C', 'ASIL_D', 'DAL_A', 'DAL_B', 'SIL_3', 'SIL_4'];
  return criticalLevels.includes(level);
}

/**
 * Get recommended verification level based on safety level
 */
export function getVerificationLevel(safetyLevel: SafetyLevel): {
  reviews: number;
  testing: string;
  documentation: string;
} {
  const config = getSafetyColorConfig(safetyLevel);

  const levels = {
    none: {
      reviews: 1,
      testing: 'Unit testing',
      documentation: 'Basic',
    },
    low: {
      reviews: 2,
      testing: 'Unit + Integration',
      documentation: 'Standard',
    },
    medium: {
      reviews: 3,
      testing: 'Unit + Integration + System',
      documentation: 'Detailed',
    },
    high: {
      reviews: 4,
      testing: 'Full V&V + Regression',
      documentation: 'Comprehensive',
    },
    critical: {
      reviews: 5,
      testing: 'Full V&V + Regression + Independent Testing',
      documentation: 'Complete + Traceability Matrix',
    },
  };

  return levels[config.criticality];
}

/**
 * Export all levels for reference
 */
export const SAFETY_LEVELS = {
  ISO26262: Object.keys(ASIL_COLORS) as ASILLevel[],
  DO178C: Object.keys(DAL_COLORS) as DALLevel[],
  IEC61508: Object.keys(SIL_COLORS) as SILLevel[],
};
