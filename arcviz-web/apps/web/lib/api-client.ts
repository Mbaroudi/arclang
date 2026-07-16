const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001';
const RUST_API_URL = process.env.NEXT_PUBLIC_RUST_API_URL || 'http://localhost:5001';

export interface Arcadia7DModel {
  operational: {
    actors: Array<{ id: string; name: string; description?: string }>;
    activities: Array<{ id: string; name: string; performed_by?: string }>;
    capabilities: Array<{ id: string; name: string; activities?: string[] }>;
    interactions: Array<{ from: string; to: string; data?: string }>;
  };
  system: {
    system: { id: string; name: string };
    actors: Array<{ id: string; name: string }>;
    functions: Array<{ id: string; name: string; allocated_to?: string }>;
    interactions: Array<{ from: string; to: string }>;
  };
  logical: {
    components: Array<{ id: string; name: string; provides?: string[]; requires?: string[] }>;
    interfaces: Array<{ id: string; name: string }>;
    dataFlows: Array<{ from: string; to: string; data?: string }>;
  };
  physical: {
    nodes: Array<{ id: string; name: string; type: 'hardware' | 'software' | 'behavior' }>;
    links: Array<{ from: string; to: string; type: string }>;
    deployments: Array<{ component: string; node: string }>;
  };
  epbs: {
    subsystems: Array<{ id: string; name: string }>;
    assemblies: Array<{ id: string; name: string; parent?: string }>;
    components: Array<{ id: string; name: string; parent?: string }>;
  };
  requirements: {
    requirements: Array<{
      id: string;
      name: string;
      type: 'functional' | 'non-functional';
      priority: string;
      status: string;
      text?: string;
      allocated_to?: string[];
    }>;
    traces: Array<{ from: string; to: string }>;
  };
  crossCutting: {
    securityPolicies: Array<{ id: string; name: string; description?: string }>;
    safetyConstraints: Array<{ id: string; name: string }>;
    performanceMetrics: Array<{ id: string; name: string; target?: string }>;
    dependencies: Array<{ from: string; to: string }>;
  };
}

export interface Arcadia7DStats {
  operational: { actors: number; activities: number; capabilities: number; interactions: number };
  system: { actors: number; functions: number; interactions: number };
  logical: { components: number; interfaces: number; dataFlows: number };
  physical: { nodes: number; links: number; deployments: number };
  epbs: { subsystems: number; assemblies: number; components: number };
  requirements: { requirements: number; traces: number };
  crossCutting: { securityPolicies: number; safetyConstraints: number; performanceMetrics: number };
}

export interface Arcadia7DParseResponse {
  success: boolean;
  model?: Arcadia7DModel;
  stats?: Arcadia7DStats;
  error?: string;
}

export interface Arcadia7DValidateResponse {
  success: boolean;
  valid: boolean;
  issues: Array<{
    type: string;
    message: string;
    severity: 'error' | 'warning' | 'info';
  }>;
  summary: {
    errors: number;
    warnings: number;
    info: number;
  };
}

export async function parseArcadia7D(code: string): Promise<Arcadia7DParseResponse> {
  try {
    const response = await fetch(`${RUST_API_URL}/api/arcadia-7d/parse`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ code }),
    });

    if (!response.ok) {
      const error = await response.json();
      return {
        success: false,
        error: error.error || 'Failed to parse code',
      };
    }

    return await response.json();
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Network error',
    };
  }
}

export async function validateArcadia7D(code: string): Promise<Arcadia7DValidateResponse> {
  try {
    const response = await fetch(`${API_URL}/api/arcadia-7d/validate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ code }),
    });

    if (!response.ok) {
      throw new Error('Validation failed');
    }

    return await response.json();
  } catch (error: any) {
    return {
      success: false,
      valid: false,
      issues: [{
        type: 'system',
        message: error.message || 'Validation error',
        severity: 'error',
      }],
      summary: { errors: 1, warnings: 0, info: 0 },
    };
  }
}
