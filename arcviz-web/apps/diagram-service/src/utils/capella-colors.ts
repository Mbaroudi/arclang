/**
 * Official Capella/Arcadia Color Specifications
 * Reference: Capella specifications.pdf - Table 6
 * 
 * These colors MUST be used for Capella-compliant diagrams.
 * Deviation from these colors will cause compliance failures for:
 * - ISO 26262 (Automotive ASIL)
 * - DO-178C (Aerospace DAL)
 * - IEC 61508 (Industrial SIL)
 */

export const CapellaColors = {
  // Operational Analysis (OA) Layer
  OPERATIONAL_ENTITY: '#FFFF99',      // Yellow - Operational entities
  OPERATIONAL_ACTOR: '#FFFF99',        // Yellow - External actors (OA)
  OPERATIONAL_ACTIVITY: '#FFB266',     // Orange - Activities/processes
  OPERATIONAL_INTERACTION: '#808080',  // Gray - Interactions/exchanges
  
  // System Analysis (SA) Layer
  SYSTEM_ACTOR: '#E6D7B8',            // Beige - External actors (SA)
  SYSTEM_FUNCTION: '#ADD8E6',         // Light Blue - System functions
  SYSTEM: '#E8F4F8',                  // Very Light Blue - System boundary
  FUNCTIONAL_EXCHANGE: '#4169E1',     // Royal Blue - Functional exchanges
  
  // Logical Architecture (LA) Layer
  LOGICAL_COMPONENT: '#6495ED',       // Cornflower Blue - Logical components
  LOGICAL_FUNCTION: '#4682B4',        // Steel Blue - Logical functions
  LOGICAL_ACTOR: '#E6D7B8',           // Beige - External actors (LA)
  
  // Physical Architecture (PA) Layer
  PHYSICAL_NODE: '#FFD700',           // Gold - Hardware nodes
  PHYSICAL_BEHAVIORAL: '#4169E1',     // Royal Blue - Software components
  PHYSICAL_LINK: '#808080',           // Gray - Physical connections
  
  // EPBS Layer
  EPBS_CONFIGURATION_ITEM: '#C0C0C0', // Silver - Configuration items
  
  // Common Elements
  INTERACTION_EXCHANGE: '#808080',     // Gray - Generic interactions
  PORT: '#FFFFFF',                     // White - Port fill
  PORT_BORDER: '#000000',              // Black - Port border
  
  // Background
  DIAGRAM_BACKGROUND: '#FFFFFF',       // White
  CONTAINER_BACKGROUND: '#F5F5F5',     // Light gray
};

/**
 * Safety Integrity Level (SIL) Border Colors
 * Used as border overlays on components based on safety criticality
 */
export const SafetyColors = {
  // Automotive (ISO 26262) - ASIL
  ASIL_D: '#8B0000',  // Dark Red - Most critical
  ASIL_C: '#DC143C',  // Crimson
  ASIL_B: '#FF8C00',  // Dark Orange
  ASIL_A: '#FFD700',  // Gold
  QM: '#808080',      // Gray - Quality Managed (non-safety)
  
  // Aerospace (DO-178C) - DAL
  DAL_A: '#8B0000',   // Dark Red - Most critical
  DAL_B: '#DC143C',   // Crimson
  DAL_C: '#FF8C00',   // Dark Orange
  DAL_D: '#FFD700',   // Gold
  DAL_E: '#808080',   // Gray - No safety impact
  
  // Industrial (IEC 61508) - SIL
  SIL_4: '#8B0000',   // Dark Red - Highest
  SIL_3: '#DC143C',   // Crimson
  SIL_2: '#FF8C00',   // Dark Orange
  SIL_1: '#FFD700',   // Gold
};

/**
 * Component Type Colors by Architectural Layer
 */
export const ComponentColors = {
  // Sensors (usually green in operational/physical layers)
  SENSOR: '#70AD47',           // Green
  
  // Actuators (usually orange)
  ACTUATOR: '#ED7D31',         // Orange
  
  // Processing/Logic (layer-dependent)
  PROCESSING_OA: '#FFB266',    // Orange (operational activity)
  PROCESSING_SA: '#ADD8E6',    // Light Blue (system function)
  PROCESSING_LA: '#6495ED',    // Cornflower Blue (logical component)
  PROCESSING_PA_SW: '#4169E1', // Royal Blue (behavioral/software)
  PROCESSING_PA_HW: '#FFD700', // Gold (node/hardware)
};

/**
 * Stereotype Colors (for <<stereotype>> annotations)
 */
export const StereotypeColors = {
  SENSOR: '#70AD47',
  ACTUATOR: '#ED7D31',
  CONTROLLER: '#5B9BD5',
  INTERFACE: '#9B59B6',
  DATABASE: '#E74C3C',
  COMMUNICATION: '#3498DB',
};

/**
 * Get color for operational activity
 */
export function getOperationalActivityColor(): string {
  return CapellaColors.OPERATIONAL_ACTIVITY;
}

/**
 * Get color for operational entity/actor
 */
export function getOperationalEntityColor(): string {
  return CapellaColors.OPERATIONAL_ENTITY;
}

/**
 * Get color for system function
 */
export function getSystemFunctionColor(): string {
  return CapellaColors.SYSTEM_FUNCTION;
}

/**
 * Get color for logical component
 */
export function getLogicalComponentColor(): string {
  return CapellaColors.LOGICAL_COMPONENT;
}

/**
 * Get color for physical node (hardware)
 */
export function getPhysicalNodeColor(): string {
  return CapellaColors.PHYSICAL_NODE;
}

/**
 * Get color for physical behavioral (software)
 */
export function getPhysicalBehavioralColor(): string {
  return CapellaColors.PHYSICAL_BEHAVIORAL;
}

/**
 * Get safety border color based on safety level
 */
export function getSafetyBorderColor(safetyLevel: string): string | null {
  const level = safetyLevel.toUpperCase().replace(/[_\s-]/g, '_');
  
  // Automotive (ASIL)
  if (level in SafetyColors && level.startsWith('ASIL')) {
    return SafetyColors[level as keyof typeof SafetyColors];
  }
  
  // Aerospace (DAL)
  if (level in SafetyColors && level.startsWith('DAL')) {
    return SafetyColors[level as keyof typeof SafetyColors];
  }
  
  // Industrial (SIL)
  if (level in SafetyColors && level.startsWith('SIL')) {
    return SafetyColors[level as keyof typeof SafetyColors];
  }
  
  // QM (Quality Managed)
  if (level === 'QM') {
    return SafetyColors.QM;
  }
  
  return null;
}

/**
 * Get component color based on layer and type
 */
export function getComponentColor(
  layer: 'OA' | 'SA' | 'LA' | 'PA',
  componentType?: string,
  stereotype?: string
): string {
  // Check stereotype first
  if (stereotype) {
    const stereoUpper = stereotype.toUpperCase();
    if (stereoUpper.includes('SENSOR')) return ComponentColors.SENSOR;
    if (stereoUpper.includes('ACTUATOR')) return ComponentColors.ACTUATOR;
  }
  
  // Layer-specific colors
  switch (layer) {
    case 'OA':
      return CapellaColors.OPERATIONAL_ACTIVITY;
    case 'SA':
      return CapellaColors.SYSTEM_FUNCTION;
    case 'LA':
      return CapellaColors.LOGICAL_COMPONENT;
    case 'PA':
      // Physical layer distinguishes HW vs SW
      if (componentType?.toLowerCase().includes('node') || 
          componentType?.toLowerCase().includes('hardware')) {
        return CapellaColors.PHYSICAL_NODE;
      }
      return CapellaColors.PHYSICAL_BEHAVIORAL;
    default:
      return CapellaColors.LOGICAL_COMPONENT;
  }
}

/**
 * Get exchange/interaction color
 */
export function getExchangeColor(layer?: 'OA' | 'SA' | 'LA' | 'PA'): string {
  if (layer === 'SA') {
    return CapellaColors.FUNCTIONAL_EXCHANGE;
  }
  return CapellaColors.INTERACTION_EXCHANGE;
}
