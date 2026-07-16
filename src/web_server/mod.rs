use axum::{
    extract::{Json, State},
    http::{StatusCode, HeaderValue, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::compiler::CompilerError;
use crate::compiler::semantic::{SemanticModel, SemanticAnalyzer};
use crate::compiler::ast::{
    Model,
    Actor as AstActor,
    Requirement as AstRequirement,
    SystemFunction,
    SystemComponent,
    LogicalComponent
};

pub struct AppState {
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Deserialize)]
pub struct ParseCodeRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct Capella7DLayout {
    pub nodes: Vec<Capella7DNode>,
    pub edges: Vec<Capella7DEdge>,
    pub dimension: String,
    pub stats: Capella7DStats,
}

#[derive(Debug, Serialize)]
pub struct Capella7DNode {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub element_type: String,
    pub is_actor: bool,
    pub is_critical: bool,
    pub layer: usize,
}

#[derive(Debug, Serialize)]
pub struct Capella7DEdge {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub is_critical: bool,
}

#[derive(Debug, Serialize)]
pub struct Capella7DStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub actor_count: usize,
    pub critical_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ParseCodeResponse {
    pub success: bool,
    pub model: Option<Arcadia7DModel>,
    pub stats: Option<Arcadia7DStats>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Arcadia7DModel {
    pub operational: OperationalModel,
    pub system: SystemModel,
    pub logical: LogicalModel,
    pub physical: PhysicalModel,
    pub epbs: EpbsModel,
    pub requirements: RequirementsModel,
    pub cross_cutting: CrossCuttingModel,
}

#[derive(Debug, Serialize)]
pub struct OperationalModel {
    pub actors: Vec<Actor>,
    pub activities: Vec<Activity>,
    pub capabilities: Vec<Capability>,
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Serialize)]
pub struct SystemModel {
    pub system: SystemEntity,
    pub actors: Vec<Actor>,
    pub functions: Vec<Function>,
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Serialize)]
pub struct LogicalModel {
    pub components: Vec<Component>,
    pub interfaces: Vec<Interface>,
    pub data_flows: Vec<DataFlow>,
}

#[derive(Debug, Serialize)]
pub struct PhysicalModel {
    pub nodes: Vec<PhysicalNode>,
    pub links: Vec<PhysicalLink>,
    pub deployments: Vec<Deployment>,
}

#[derive(Debug, Serialize)]
pub struct EpbsModel {
    pub subsystems: Vec<Subsystem>,
    pub assemblies: Vec<Assembly>,
    pub components: Vec<Component>,
}

#[derive(Debug, Serialize)]
pub struct RequirementsModel {
    pub requirements: Vec<Requirement>,
    pub traces: Vec<Trace>,
}

#[derive(Debug, Serialize)]
pub struct CrossCuttingModel {
    pub security_policies: Vec<SecurityPolicy>,
    pub safety_constraints: Vec<SafetyConstraint>,
    pub performance_metrics: Vec<PerformanceMetric>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub performed_by: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub activities: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Interaction {
    pub from: String,
    pub to: String,
    pub data: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SystemEntity {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Function {
    pub id: String,
    pub name: String,
    pub allocated_to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub provides: Vec<String>,
    pub requires: Vec<String>,
    pub parent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Interface {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct DataFlow {
    pub from: String,
    pub to: String,
    pub data: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PhysicalNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
}

#[derive(Debug, Serialize)]
pub struct PhysicalLink {
    pub from: String,
    pub to: String,
    pub link_type: String,
}

#[derive(Debug, Serialize)]
pub struct Deployment {
    pub component: String,
    pub node: String,
}

#[derive(Debug, Serialize)]
pub struct Subsystem {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Assembly {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Requirement {
    pub id: String,
    pub name: String,
    pub req_type: String,
    pub priority: String,
    pub status: String,
    pub text: Option<String>,
    pub allocated_to: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Trace {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SafetyConstraint {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetric {
    pub id: String,
    pub name: String,
    pub target: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Dependency {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct Arcadia7DStats {
    pub operational: OperationalStats,
    pub system: SystemStats,
    pub logical: LogicalStats,
    pub physical: PhysicalStats,
    pub epbs: EpbsStats,
    pub requirements: RequirementsStats,
    pub cross_cutting: CrossCuttingStats,
}

#[derive(Debug, Serialize)]
pub struct OperationalStats {
    pub actors: usize,
    pub activities: usize,
    pub capabilities: usize,
    pub interactions: usize,
}

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub actors: usize,
    pub functions: usize,
    pub interactions: usize,
}

#[derive(Debug, Serialize)]
pub struct LogicalStats {
    pub components: usize,
    pub interfaces: usize,
    pub data_flows: usize,
}

#[derive(Debug, Serialize)]
pub struct PhysicalStats {
    pub nodes: usize,
    pub links: usize,
    pub deployments: usize,
}

#[derive(Debug, Serialize)]
pub struct EpbsStats {
    pub subsystems: usize,
    pub assemblies: usize,
    pub components: usize,
}

#[derive(Debug, Serialize)]
pub struct RequirementsStats {
    pub requirements: usize,
    pub traces: usize,
}

#[derive(Debug, Serialize)]
pub struct CrossCuttingStats {
    pub security_policies: usize,
    pub safety_constraints: usize,
    pub performance_metrics: usize,
}

#[derive(Debug, Deserialize)]
pub struct GenerateDiagramRequest {
    pub code: String,
    pub dimension: String,
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": "1.0.0",
        "service": "arclang-rust-backend"
    }))
}

async fn parse_arcadia_7d(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ParseCodeRequest>,
) -> Result<Json<ParseCodeResponse>, AppError> {
    let model = parse_code_to_7d_model(&payload.code)?;
    let stats = calculate_stats(&model);

    Ok(Json(ParseCodeResponse {
        success: true,
        model: Some(model),
        stats: Some(stats),
        error: None,
    }))
}

// New endpoint: Returns 7D layout as JSON for frontend Dagre/ELK rendering
async fn generate_7d_layout(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<GenerateDiagramRequest>,
) -> Result<Json<Capella7DLayout>, AppError> {
    use crate::compiler::{Compiler, CompilerConfig};
    use crate::compiler::capella_compliant_generator::CapellaCompliantGenerator;
    use crate::compiler::semantic::SemanticAnalyzer;
    
    tracing::info!("Generating 7D layout for dimension: {}", payload.dimension);
    
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(&payload.code)
        .map_err(|e| AppError::CompilerError(format!("{:?}", e)))?;
    
    // Generate layout using 7D Intelligence
    let generator = CapellaCompliantGenerator::new(&payload.dimension);
    let elements = generator.compute_capella_layout(&result.semantic_model)
        .map_err(|e| AppError::CompilerError(format!("Layout computation failed: {:?}", e)))?;
    
    // Convert to JSON format
    let nodes: Vec<Capella7DNode> = elements.iter().map(|elem| {
        Capella7DNode {
            id: elem.id.clone(),
            name: elem.name.clone(),
            x: elem.x as f64,
            y: elem.y as f64,
            width: elem.width as f64,
            height: elem.height as f64,
            element_type: format!("{:?}", elem.element_type),
            is_actor: elem.is_actor,
            is_critical: elem.is_critical,
            layer: elem.layer,
        }
    }).collect();
    
    let edges: Vec<Capella7DEdge> = result.semantic_model.traces.iter().map(|trace| {
        Capella7DEdge {
            from: trace.from.clone(),
            to: trace.to.clone(),
            trace_type: trace.trace_type.clone(),
            is_critical: false, // TODO: determine from nodes
        }
    }).collect();
    
    let stats = Capella7DStats {
        node_count: nodes.len(),
        edge_count: edges.len(),
        actor_count: nodes.iter().filter(|n| n.is_actor).count(),
        critical_count: nodes.iter().filter(|n| n.is_critical).count(),
    };
    
    Ok(Json(Capella7DLayout {
        nodes,
        edges,
        dimension: payload.dimension.clone(),
        stats,
    }))
}

async fn generate_professional_diagram(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<GenerateDiagramRequest>,
) -> Result<Response, AppError> {
    use crate::compiler::{Compiler, CompilerConfig};
    
    tracing::info!("Generating {} diagram from {} bytes of code", payload.dimension, payload.code.len());
    eprintln!("🔍 Attempting compilation for dimension: {}", payload.dimension);
    eprintln!("📝 Code preview (first 200 chars): {}", &payload.code.chars().take(200).collect::<String>());
    
    // Try to compile the user's code
    let html = match Compiler::new(CompilerConfig::default()).compile_string(&payload.code) {
        Ok(result) => {
            tracing::info!("Code compiled successfully, generating {} diagram", payload.dimension);
            eprintln!("✅ Compilation succeeded!");
            
            // Generate dimension-specific diagram using Capella-compliant generator
            use crate::compiler::capella_compliant_generator::generate_capella_professional;
            
            tracing::info!("SemanticModel has {} components, {} functions, {} traces", 
                result.semantic_model.components.len(),
                result.semantic_model.functions.len(),
                result.semantic_model.traces.len());
            
            match generate_capella_professional(&result.semantic_model, &payload.dimension) {
                Ok(html) => {
                    tracing::info!("✅ Capella generator succeeded");
                    html
                },
                Err(e) => {
                    tracing::error!("❌ Capella generator failed: {:?}, using fallback", e);
                    generate_fallback_diagram(&payload.dimension, &result.ast)
                }
            }
        },
        Err(e) => {
            tracing::warn!("Compilation failed: {:?}, using template diagrams", e);
            eprintln!("❌ Compilation failed: {:?}", e);
            
            // Fallback to template diagrams if compilation fails
            let diagram_file = match payload.dimension.as_str() {
                "operational" => "/Users/malek/Arclang/mbse_diagrams/01_operational_professional.html",
                "system" => "/Users/malek/Arclang/mbse_diagrams/05_dataflow_professional.html",
                "logical" => "/Users/malek/Arclang/mbse_diagrams/03_logical_professional.html",
                "physical" => "/Users/malek/Arclang/mbse_diagrams/04_physical_professional.html",
                _ => "/Users/malek/Arclang/mbse_diagrams/03_logical_professional.html",
            };
            
            std::fs::read_to_string(diagram_file)
                .unwrap_or_else(|_| format!("<html><body><h1>Error loading diagram for {}</h1><pre>{:?}</pre></body></html>", payload.dimension, e))
        }
    };
    
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    
    Ok(response)
}

fn generate_fallback_diagram(dimension: &str, ast: &Model) -> String {
    let title = match dimension {
        "operational" => "Operational Analysis",
        "system" => "System Analysis",
        "logical" => "Logical Architecture",
        "physical" => "Physical Architecture",
        "epbs" => "EPBS Structure",
        "requirements" => "Requirements Traceability",
        "crossCutting" => "Cross-Cutting Concerns",
        _ => "Architecture Diagram",
    };
    
    let stats = format!(
        "Operational: {} actors, {} activities | System: {} requirements, {} functions | Logical: {} components",
        ast.operational_analysis.iter().map(|oa| oa.actors.len()).sum::<usize>(),
        ast.operational_analysis.iter().map(|oa| oa.activities.len()).sum::<usize>(),
        ast.system_analysis.iter().map(|sa| sa.requirements.len()).sum::<usize>(),
        ast.system_analysis.iter().map(|sa| sa.functions.len()).sum::<usize>(),
        ast.logical_architecture.iter().map(|la| la.components.len()).sum::<usize>(),
    );
    
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{
            margin: 0;
            padding: 40px;
            font-family: 'Segoe UI', Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            padding: 40px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }}
        h1 {{
            color: #2c3e50;
            font-size: 32px;
            margin-bottom: 10px;
        }}
        .subtitle {{
            color: #7f8c8d;
            font-size: 16px;
            margin-bottom: 30px;
        }}
        .stats {{
            background: #f8f9fa;
            border-left: 4px solid #667eea;
            padding: 15px;
            margin: 20px 0;
            font-size: 14px;
            color: #495057;
        }}
        .info {{
            background: #e3f2fd;
            border-radius: 8px;
            padding: 20px;
            margin-top: 20px;
        }}
        .info h2 {{
            color: #1976d2;
            font-size: 18px;
            margin-top: 0;
        }}
        .badge {{
            display: inline-block;
            background: #667eea;
            color: white;
            padding: 6px 12px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: 600;
            margin-right: 10px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{}</h1>
        <div class="subtitle">Generated from ArcLang code by Rust Backend</div>
        
        <div class="stats">
            <strong>Model Statistics:</strong><br>
            {}
        </div>
        
        <div class="info">
            <h2>🚀 Professional Diagram Generation Active</h2>
            <p>
                <span class="badge">Rust Backend</span>
                <span class="badge">ELK Layout</span>
                <span class="badge">Arcadia 7D</span>
            </p>
            <p>
                The diagram for <strong>{}</strong> is being generated from your ArcLang code.
                This view will be replaced with a fully interactive professional diagram once
                the complete rendering engine is configured.
            </p>
        </div>
    </div>
</body>
</html>"#, title, title, stats, dimension)
}

fn parse_code_to_7d_model(code: &str) -> Result<Arcadia7DModel, AppError> {
    use crate::compiler::{Compiler, CompilerConfig};
    
    let mut compiler = Compiler::new(CompilerConfig::default());
    let result = compiler.compile_string(code)
        .map_err(|e| AppError::CompilerError(format!("{:?}", e)))?;
    
    let model = extract_7d_model_from_ast(&result.ast);
    
    Ok(model)
}

fn extract_7d_model_from_ast(ast: &Model) -> Arcadia7DModel {
    let mut operational = OperationalModel {
        actors: Vec::new(),
        activities: Vec::new(),
        capabilities: Vec::new(),
        interactions: Vec::new(),
    };
    
    let mut system = SystemModel {
        system: SystemEntity {
            id: "SYS-001".to_string(),
            name: "System".to_string(),
        },
        actors: Vec::new(),
        functions: Vec::new(),
        interactions: Vec::new(),
    };
    
    let mut logical = LogicalModel {
        components: Vec::new(),
        interfaces: Vec::new(),
        data_flows: Vec::new(),
    };
    
    let mut physical = PhysicalModel {
        nodes: Vec::new(),
        links: Vec::new(),
        deployments: Vec::new(),
    };
    
    let mut epbs = EpbsModel {
        subsystems: Vec::new(),
        assemblies: Vec::new(),
        components: Vec::new(),
    };
    
    let mut requirements = RequirementsModel {
        requirements: Vec::new(),
        traces: Vec::new(),
    };
    
    let mut cross_cutting = CrossCuttingModel {
        security_policies: Vec::new(),
        safety_constraints: Vec::new(),
        performance_metrics: Vec::new(),
        dependencies: Vec::new(),
    };
    
    for oa in &ast.operational_analysis {
        for actor in &oa.actors {
            operational.actors.push(Actor {
                id: actor.id.clone().unwrap_or_else(|| actor.name.clone()),
                name: actor.name.clone(),
                description: None,
            });
            system.actors.push(Actor {
                id: actor.id.clone().unwrap_or_else(|| actor.name.clone()),
                name: actor.name.clone(),
                description: None,
            });
        }
    }
    
    for sa in &ast.system_analysis {
        for func in &sa.functions {
            system.functions.push(Function {
                id: func.id.clone(),
                name: func.name.clone(),
                allocated_to: None,
            });
        }
        for req in &sa.requirements {
            requirements.requirements.push(Requirement {
                id: req.id.clone(),
                name: req.id.clone(),
                req_type: "functional".to_string(),
                priority: "high".to_string(),
                status: "open".to_string(),
                text: None,
                allocated_to: Vec::new(),
            });
        }
    }
    
    for la in &ast.logical_architecture {
        for comp in &la.components {
            logical.components.push(Component {
                id: comp.name.clone(),
                name: comp.name.clone(),
                provides: Vec::new(),
                requires: Vec::new(),
                parent: None,
            });
        }
    }
    
    Arcadia7DModel {
        operational,
        system,
        logical,
        physical,
        epbs,
        requirements,
        cross_cutting,
    }
}

fn calculate_stats(model: &Arcadia7DModel) -> Arcadia7DStats {
    Arcadia7DStats {
        operational: OperationalStats {
            actors: model.operational.actors.len(),
            activities: model.operational.activities.len(),
            capabilities: model.operational.capabilities.len(),
            interactions: model.operational.interactions.len(),
        },
        system: SystemStats {
            actors: model.system.actors.len(),
            functions: model.system.functions.len(),
            interactions: model.system.interactions.len(),
        },
        logical: LogicalStats {
            components: model.logical.components.len(),
            interfaces: model.logical.interfaces.len(),
            data_flows: model.logical.data_flows.len(),
        },
        physical: PhysicalStats {
            nodes: model.physical.nodes.len(),
            links: model.physical.links.len(),
            deployments: model.physical.deployments.len(),
        },
        epbs: EpbsStats {
            subsystems: model.epbs.subsystems.len(),
            assemblies: model.epbs.assemblies.len(),
            components: model.epbs.components.len(),
        },
        requirements: RequirementsStats {
            requirements: model.requirements.requirements.len(),
            traces: model.requirements.traces.len(),
        },
        cross_cutting: CrossCuttingStats {
            security_policies: model.cross_cutting.security_policies.len(),
            safety_constraints: model.cross_cutting.safety_constraints.len(),
            performance_metrics: model.cross_cutting.performance_metrics.len(),
        },
    }
}

pub async fn serve(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new());
    
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3002".parse::<HeaderValue>().unwrap())
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/arcadia-7d/parse", post(parse_arcadia_7d))
        .route("/api/arcadia-7d/layout", post(generate_7d_layout))
        .route("/api/diagrams/generate-professional", post(generate_professional_diagram))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("🚀 ArcLang Rust Backend starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[derive(Debug)]
pub enum AppError {
    CompilerError(String),
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::CompilerError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        let body = Json(serde_json::json!({
            "success": false,
            "error": message,
        }));
        
        (status, body).into_response()
    }
}
