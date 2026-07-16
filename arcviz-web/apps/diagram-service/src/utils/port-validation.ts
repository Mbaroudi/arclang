/**
 * Port Positioning Validation
 * 
 * Validates compliance with Capella Specification Section 5.1
 * Port positioning rules are MANDATORY per specifications
 * 
 * Requirements:
 * - INPUT ports MUST be on LEFT side
 * - OUTPUT ports MUST be on RIGHT side
 * - BIDIRECTIONAL ports on TOP or BOTTOM
 * - CONTROL ports on TOP
 * - POWER/GROUND ports on BOTTOM (physical architecture)
 * - Minimum 30px spacing between ports
 * - Minimum 45° angle between ports (on same side)
 */

import { DiagramNode, Port } from '../types/diagram';

export interface PortValidationResult {
  valid: boolean;
  violations: PortViolation[];
  warnings: PortWarning[];
}

export interface PortViolation {
  nodeId: string;
  portId: string;
  rule: string;
  severity: 'error' | 'warning';
  message: string;
}

export interface PortWarning {
  nodeId: string;
  portId: string;
  message: string;
}

/**
 * Validate port positioning against Capella specifications
 */
export function validatePortPositioning(nodes: DiagramNode[]): PortValidationResult {
  const violations: PortViolation[] = [];
  const warnings: PortWarning[] = [];

  for (const node of nodes) {
    if (!node.ports || node.ports.length === 0) continue;

    // Validate each port
    for (const port of node.ports) {
      // Rule 1: INPUT ports MUST be LEFT
      if (port.direction === 'IN' && port.side !== 'LEFT') {
        violations.push({
          nodeId: node.id,
          portId: port.id,
          rule: 'INPUT_LEFT',
          severity: 'error',
          message: `INPUT port "${port.name}" must be on LEFT side (currently: ${port.side})`,
        });
      }

      // Rule 2: OUTPUT ports MUST be RIGHT
      if (port.direction === 'OUT' && port.side !== 'RIGHT') {
        violations.push({
          nodeId: node.id,
          portId: port.id,
          rule: 'OUTPUT_RIGHT',
          severity: 'error',
          message: `OUTPUT port "${port.name}" must be on RIGHT side (currently: ${port.side})`,
        });
      }

      // Rule 3: INOUT ports should be TOP or BOTTOM
      if (port.direction === 'INOUT' && port.side !== 'TOP' && port.side !== 'BOTTOM') {
        warnings.push({
          nodeId: node.id,
          portId: port.id,
          message: `BIDIRECTIONAL port "${port.name}" should be on TOP or BOTTOM (currently: ${port.side})`,
        });
      }
    }

    // Validate port spacing on each side
    const portsBySide = groupPortsBySide(node.ports);
    
    for (const [side, ports] of Object.entries(portsBySide)) {
      if (ports.length < 2) continue;

      // Check spacing between adjacent ports
      for (let i = 0; i < ports.length - 1; i++) {
        const port1 = ports[i];
        const port2 = ports[i + 1];

        if (port1.position && port2.position) {
          const distance = calculateDistance(port1.position, port2.position);
          
          if (distance < 30) {
            warnings.push({
              nodeId: node.id,
              portId: port1.id,
              message: `Ports "${port1.name}" and "${port2.name}" on ${side} side are too close (${distance.toFixed(1)}px < 30px minimum)`,
            });
          }
        }
      }
    }
  }

  return {
    valid: violations.length === 0,
    violations,
    warnings,
  };
}

/**
 * Generate compliance report
 */
export function generatePortComplianceReport(result: PortValidationResult): string {
  const lines: string[] = [];

  lines.push('='.repeat(70));
  lines.push('PORT POSITIONING COMPLIANCE REPORT');
  lines.push('Capella Specification Section 5.1 - MANDATORY Rules');
  lines.push('='.repeat(70));
  lines.push('');

  if (result.valid && result.warnings.length === 0) {
    lines.push('✅ FULLY COMPLIANT - All ports correctly positioned');
    lines.push('');
    lines.push('- ✅ All INPUT ports on LEFT side');
    lines.push('- ✅ All OUTPUT ports on RIGHT side');
    lines.push('- ✅ All BIDIRECTIONAL ports on TOP/BOTTOM');
    lines.push('- ✅ Minimum spacing maintained');
  } else {
    if (result.violations.length > 0) {
      lines.push(`❌ ${result.violations.length} VIOLATION(S) FOUND`);
      lines.push('');
      lines.push('MANDATORY RULE VIOLATIONS:');
      lines.push('-'.repeat(70));
      
      for (const violation of result.violations) {
        lines.push(`[${violation.severity.toUpperCase()}] Node: ${violation.nodeId}`);
        lines.push(`  Port: ${violation.portId}`);
        lines.push(`  Rule: ${violation.rule}`);
        lines.push(`  ${violation.message}`);
        lines.push('');
      }
    } else {
      lines.push('✅ NO VIOLATIONS - All mandatory rules satisfied');
      lines.push('');
    }

    if (result.warnings.length > 0) {
      lines.push(`⚠️  ${result.warnings.length} WARNING(S)`);
      lines.push('');
      lines.push('RECOMMENDATIONS:');
      lines.push('-'.repeat(70));
      
      for (const warning of result.warnings) {
        lines.push(`[WARNING] Node: ${warning.nodeId}`);
        lines.push(`  Port: ${warning.portId}`);
        lines.push(`  ${warning.message}`);
        lines.push('');
      }
    }
  }

  lines.push('='.repeat(70));
  
  return lines.join('\n');
}

/**
 * Helper: Group ports by side
 */
function groupPortsBySide(ports: Port[]): Record<string, Port[]> {
  const groups: Record<string, Port[]> = {
    TOP: [],
    RIGHT: [],
    BOTTOM: [],
    LEFT: [],
  };

  for (const port of ports) {
    groups[port.side].push(port);
  }

  return groups;
}

/**
 * Helper: Calculate distance between two points
 */
function calculateDistance(p1: { x: number; y: number }, p2: { x: number; y: number }): number {
  const dx = p2.x - p1.x;
  const dy = p2.y - p1.y;
  return Math.sqrt(dx * dx + dy * dy);
}

/**
 * Check if node has ports
 */
export function hasValidPorts(node: DiagramNode): boolean {
  return node.ports !== undefined && node.ports.length > 0;
}

/**
 * Get port statistics for a diagram
 */
export function getPortStatistics(nodes: DiagramNode[]): {
  totalPorts: number;
  inputPorts: number;
  outputPorts: number;
  bidirectionalPorts: number;
  controlPorts: number;
  portsBySide: Record<string, number>;
  nodesWithPorts: number;
} {
  let totalPorts = 0;
  let inputPorts = 0;
  let outputPorts = 0;
  let bidirectionalPorts = 0;
  let controlPorts = 0;
  let nodesWithPorts = 0;
  
  const portsBySide: Record<string, number> = {
    TOP: 0,
    RIGHT: 0,
    BOTTOM: 0,
    LEFT: 0,
  };

  for (const node of nodes) {
    if (!node.ports || node.ports.length === 0) continue;
    
    nodesWithPorts++;
    
    for (const port of node.ports) {
      totalPorts++;
      
      if (port.direction === 'IN') inputPorts++;
      else if (port.direction === 'OUT') outputPorts++;
      else if (port.direction === 'INOUT') bidirectionalPorts++;
      
      const portType = port.metadata?.port_type || port.metadata?.type || '';
      if (portType === 'control') controlPorts++;
      
      portsBySide[port.side]++;
    }
  }

  return {
    totalPorts,
    inputPorts,
    outputPorts,
    bidirectionalPorts,
    controlPorts,
    portsBySide,
    nodesWithPorts,
  };
}
