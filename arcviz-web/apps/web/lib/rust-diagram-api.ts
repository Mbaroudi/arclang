const RUST_API_URL = process.env.NEXT_PUBLIC_RUST_API_URL || 'http://localhost:5001';

export interface ProfessionalDiagramRequest {
  code: string;
  dimension: 'operational' | 'system' | 'logical' | 'physical' | 'epbs' | 'requirements' | 'crossCutting';
}

export async function generateProfessionalDiagram(
  request: ProfessionalDiagramRequest
): Promise<{ success: boolean; html?: string; error?: string }> {
  try {
    console.log(`[Rust] Generating professional ${request.dimension} diagram...`);
    
    const response = await fetch(`${RUST_API_URL}/api/diagrams/generate-professional`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = await response.json();
      return {
        success: false,
        error: error.error || `HTTP ${response.status}`,
      };
    }

    const html = await response.text();
    console.log(`[Rust] ✅ Professional diagram generated (${html.length} bytes)`);
    
    return {
      success: true,
      html,
    };
  } catch (error: any) {
    console.error('[Rust] Failed to generate professional diagram:', error);
    return {
      success: false,
      error: error.message || 'Network error connecting to Rust backend',
    };
  }
}

export async function checkRustBackendHealth(): Promise<boolean> {
  try {
    const response = await fetch(`${RUST_API_URL}/health`, { method: 'GET' });
    return response.ok;
  } catch {
    return false;
  }
}
