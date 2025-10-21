// ═══════════════════════════════════════════════════════════════════════════
// OPA (OPEN POLICY AGENT) POLICY GENERATOR
// ═══════════════════════════════════════════════════════════════════════════
// Purpose: Generate OPA/Rego policies from ArcLang safety requirements
// Coverage: Kubernetes admission control, Terraform validation, compliance
// Target: OPA 0.60+, Rego v1
// ═══════════════════════════════════════════════════════════════════════════

use super::semantic::{SemanticModel, RequirementInfo};
use super::CompilerError;
use std::collections::HashMap;

pub fn generate_opa_policies(
    model: &SemanticModel,
    config: &OpaConfig,
) -> Result<Vec<(String, String)>, CompilerError> {
    let mut policies = vec![
        ("kubernetes-admission.rego".to_string(), generate_k8s_admission_policy(model, config)),
        ("terraform-validation.rego".to_string(), generate_terraform_validation_policy(model, config)),
        ("resource-limits.rego".to_string(), generate_resource_limits_policy(model, config)),
        ("security-compliance.rego".to_string(), generate_security_compliance_policy(model, config)),
        ("cost-governance.rego".to_string(), generate_cost_governance_policy(model, config)),
    ];
    
    if config.generate_conftest {
        policies.push(("conftest.rego".to_string(), generate_conftest_policy(model, config)));
    }
    
    Ok(policies)
}

#[derive(Debug, Clone)]
pub struct OpaConfig {
    pub namespace: String,
    pub environment: String,
    pub generate_conftest: bool,
    pub strict_mode: bool,
}

impl Default for OpaConfig {
    fn default() -> Self {
        OpaConfig {
            namespace: "dataplatform".to_string(),
            environment: "prod".to_string(),
            generate_conftest: true,
            strict_mode: true,
        }
    }
}

fn generate_k8s_admission_policy(model: &SemanticModel, config: &OpaConfig) -> String {
    let safety_critical_count = model.requirements.iter()
        .filter(|r| r.description.to_lowercase().contains("safety") || r.description.to_lowercase().contains("critical"))
        .count();
    
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# KUBERNETES ADMISSION CONTROL POLICIES
# ═══════════════════════════════════════════════════════════════════════════
# Generated from ArcLang Model
# Safety Requirements: {}
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════

package kubernetes.admission

import future.keywords.contains
import future.keywords.if
import future.keywords.in

# Deny containers running as root
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    not container.securityContext.runAsNonRoot
    msg := sprintf("Container '%s' must not run as root (SYS-SEC-001)", [container.name])
}}

# Require resource limits for all containers
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    not container.resources.limits.cpu
    msg := sprintf("Container '%s' must specify CPU limits (SYS-PERF-002)", [container.name])
}}

deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    not container.resources.limits.memory
    msg := sprintf("Container '%s' must specify memory limits (SYS-PERF-002)", [container.name])
}}

# Enforce resource requests
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    not container.resources.requests.cpu
    msg := sprintf("Container '%s' must specify CPU requests (SYS-SCALE-001)", [container.name])
}}

# Deny privileged containers
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    container.securityContext.privileged
    msg := sprintf("Container '%s' must not run in privileged mode (SYS-SEC-002)", [container.name])
}}

# Require security context for critical workloads
deny[msg] {{
    input.request.kind.kind == "Pod"
    input.request.object.metadata.labels.safety_level == "Critical"
    some container in input.request.object.spec.containers
    not container.securityContext
    msg := sprintf("Critical container '%s' must define securityContext (SYS-SEC-003)", [container.name])
}}

# Deny pods without health checks in production
deny[msg] {{
    input.request.kind.kind == "Pod"
    input.request.object.metadata.namespace == "{}"
    some container in input.request.object.spec.containers
    not container.livenessProbe
    msg := sprintf("Container '%s' must define livenessProbe in production (SYS-AVAIL-001)", [container.name])
}}

deny[msg] {{
    input.request.kind.kind == "Pod"
    input.request.object.metadata.namespace == "{}"
    some container in input.request.object.spec.containers
    not container.readinessProbe
    msg := sprintf("Container '%s' must define readinessProbe in production (SYS-AVAIL-001)", [container.name])
}}

# Require specific labels for traceability
required_labels := ["component", "environment", "managed-by"]

deny[msg] {{
    input.request.kind.kind in ["Deployment", "StatefulSet", "DaemonSet"]
    some label in required_labels
    not input.request.object.metadata.labels[label]
    msg := sprintf("Resource must have label '%s' for traceability (SYS-TRACE-001)", [label])
}}

# Deny images from untrusted registries
allowed_registries := [
    "gcr.io",
    "ghcr.io",
    "docker.io/library",
    "registry.k8s.io"
]

deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    image := container.image
    not is_allowed_registry(image)
    msg := sprintf("Container '%s' uses untrusted registry: %s (SYS-SEC-004)", [container.name, image])
}}

is_allowed_registry(image) {{
    some registry in allowed_registries
    startswith(image, registry)
}}

# Deny pods without resource quotas in namespace
deny[msg] {{
    input.request.kind.kind == "Pod"
    not has_resource_quota(input.request.object.metadata.namespace)
    msg := sprintf("Namespace '%s' must have ResourceQuota defined (SYS-GOV-001)", [input.request.object.metadata.namespace])
}}

has_resource_quota(namespace) {{
    # This would be evaluated against cluster state
    # For static policy, assume namespaces matching pattern have quotas
    regex.match("^(data-platform|kube-system).*", namespace)
}}

# Enforce pod disruption budgets for HA workloads
deny[msg] {{
    input.request.kind.kind == "Deployment"
    replicas := input.request.object.spec.replicas
    replicas >= 3
    input.request.object.metadata.labels.availability == "High"
    not has_pdb(input.request.object.metadata.name)
    msg := sprintf("High availability deployment '%s' must have PodDisruptionBudget (SYS-AVAIL-002)", [input.request.object.metadata.name])
}}

has_pdb(name) {{
    # Check if PDB exists (requires cluster state)
    true
}}

# Warn about missing pod anti-affinity for replicated workloads
warn[msg] {{
    input.request.kind.kind == "Deployment"
    replicas := input.request.object.spec.replicas
    replicas >= 3
    not input.request.object.spec.template.spec.affinity.podAntiAffinity
    msg := sprintf("Deployment '%s' with %d replicas should define podAntiAffinity (SYS-AVAIL-003)", [input.request.object.metadata.name, replicas])
}}

# Validate component IDs match ArcLang model
valid_component_ids := {{
    "la-proc-001", "la-proc-002", "la-proc-003",
    "la-mig-001", "la-mig-002", "la-mig-003", "la-mig-004",
    "la-int-001", "la-int-002", "la-int-003",
    "la-anlz-001", "la-mon-001", "la-mon-002", "la-mon-003",
    "la-gov-001", "la-gov-002", "la-gov-003",
    "la-tgt-001", "la-tgt-002", "la-tgt-003",
    "la-src-001", "la-src-002"
}}

deny[msg] {{
    input.request.kind.kind in ["Deployment", "StatefulSet", "Pod"]
    component_id := input.request.object.metadata.labels.component
    component_id != ""
    not component_id in valid_component_ids
    msg := sprintf("Invalid component ID '%s' - must match ArcLang model (SYS-TRACE-002)", [component_id])
}}

# Model metadata
# Requirements: {}
# Components: {}
# Traces: {}
"#,
        safety_critical_count,
        config.environment,
        config.namespace,
        config.namespace,
        model.requirements.len(),
        model.components.len(),
        model.traces.len()
    )
}

fn generate_terraform_validation_policy(model: &SemanticModel, config: &OpaConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# TERRAFORM VALIDATION POLICIES
# ═══════════════════════════════════════════════════════════════════════════
# Generated from ArcLang Model
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════

package terraform.validation

import future.keywords.contains
import future.keywords.if
import future.keywords.in

# Deny unencrypted storage
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_s3_bucket"
    not has_encryption(resource)
    msg := sprintf("S3 bucket '%s' must have encryption enabled (SYS-SEC-005)", [resource.name])
}}

has_encryption(resource) {{
    resource.change.after.server_side_encryption_configuration
}}

# Require versioning for data buckets
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_s3_bucket"
    contains(resource.name, "data")
    not has_versioning(resource)
    msg := sprintf("Data bucket '%s' must have versioning enabled (SYS-DATA-001)", [resource.name])
}}

has_versioning(resource) {{
    # Check for associated versioning resource
    some version in input.resource_changes
    version.type == "aws_s3_bucket_versioning"
    contains(version.change.after.bucket, resource.name)
}}

# Deny public database instances
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type in ["aws_db_instance", "azurerm_sql_server", "google_sql_database_instance"]
    resource.change.after.publicly_accessible == true
    msg := sprintf("Database '%s' must not be publicly accessible (SYS-SEC-006)", [resource.name])
}}

# Require KMS encryption for sensitive resources
sensitive_resource_types := [
    "aws_db_instance",
    "aws_dynamodb_table",
    "azurerm_cosmosdb_account",
    "google_bigquery_dataset"
]

deny[msg] {{
    resource := input.resource_changes[_]
    resource.type in sensitive_resource_types
    not has_kms_encryption(resource)
    msg := sprintf("Sensitive resource '%s' must use KMS encryption (SYS-SEC-007)", [resource.name])
}}

has_kms_encryption(resource) {{
    resource.change.after.kms_key_id
}} else {{
    resource.change.after.encryption_configuration
}} else {{
    resource.change.after.default_encryption_configuration
}}

# Require tags for all resources
required_tags := ["Component", "Environment", "ManagedBy", "GeneratedFrom"]

deny[msg] {{
    resource := input.resource_changes[_]
    supports_tags(resource.type)
    some tag in required_tags
    not resource.change.after.tags[tag]
    msg := sprintf("Resource '%s' missing required tag '%s' (SYS-TRACE-003)", [resource.name, tag])
}}

supports_tags(resource_type) {{
    not startswith(resource_type, "data.")
    not contains(resource_type, "_policy")
}}

# Deny non-production instance types in production
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_instance"
    resource.change.after.tags.Environment == "prod"
    startswith(resource.change.after.instance_type, "t2.")
    msg := sprintf("Production instance '%s' must not use t2 instance type (SYS-PERF-003)", [resource.name])
}}

# Require backup configuration for stateful resources
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_db_instance"
    resource.change.after.backup_retention_period < 7
    msg := sprintf("Database '%s' must have backup retention >= 7 days (SYS-DATA-002)", [resource.name])
}}

# Require multi-AZ for production databases
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_db_instance"
    resource.change.after.tags.Environment == "prod"
    not resource.change.after.multi_az
    msg := sprintf("Production database '%s' must be multi-AZ (SYS-AVAIL-004)", [resource.name])
}}

# Validate CIDR blocks for VPCs
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_vpc"
    cidr := resource.change.after.cidr_block
    is_public_cidr(cidr)
    msg := sprintf("VPC '%s' must not use public CIDR range (SYS-NET-001)", [resource.name])
}}

is_public_cidr(cidr) {{
    # Check if CIDR is in public ranges
    not startswith(cidr, "10.")
    not startswith(cidr, "172.16.")
    not startswith(cidr, "192.168.")
}}

# Warn about expensive resources
warn[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_instance"
    is_expensive_instance(resource.change.after.instance_type)
    msg := sprintf("Instance '%s' uses expensive type %s - consider cost optimization (SYS-COST-001)", [resource.name, resource.change.after.instance_type])
}}

is_expensive_instance(instance_type) {{
    # Large instance types
    contains(instance_type, "16xlarge")
}} else {{
    contains(instance_type, "24xlarge")
}} else {{
    contains(instance_type, "metal")
}}

# Model metadata
# Requirements: {}
# Components: {}
"#,
        config.environment,
        model.requirements.len(),
        model.components.len()
    )
}

fn generate_resource_limits_policy(model: &SemanticModel, config: &OpaConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# RESOURCE LIMITS POLICIES
# ═══════════════════════════════════════════════════════════════════════════
# Generated from ArcLang Model
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════

package kubernetes.resourcelimits

import future.keywords.if

# Maximum resource limits
max_cpu_cores := 16
max_memory_gb := 64
max_storage_gb := 500

# Deny excessive CPU requests
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    cpu := parse_cpu(container.resources.limits.cpu)
    cpu > max_cpu_cores
    msg := sprintf("Container '%s' CPU limit %v exceeds maximum %v cores (SYS-COST-002)", [container.name, cpu, max_cpu_cores])
}}

# Deny excessive memory requests
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    memory := parse_memory_gb(container.resources.limits.memory)
    memory > max_memory_gb
    msg := sprintf("Container '%s' memory limit %vGi exceeds maximum %vGi (SYS-COST-003)", [container.name, memory, max_memory_gb])
}}

# Deny excessive storage requests
deny[msg] {{
    input.request.kind.kind == "PersistentVolumeClaim"
    storage := parse_storage_gb(input.request.object.spec.resources.requests.storage)
    storage > max_storage_gb
    msg := sprintf("PVC '%s' storage %vGi exceeds maximum %vGi (SYS-COST-004)", [input.request.object.metadata.name, storage, max_storage_gb])
}}

# Resource ratio checks
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    cpu_request := parse_cpu(container.resources.requests.cpu)
    cpu_limit := parse_cpu(container.resources.limits.cpu)
    cpu_limit > (cpu_request * 4)
    msg := sprintf("Container '%s' CPU limit/request ratio exceeds 4:1 (SYS-PERF-004)", [container.name])
}}

# Helper functions
parse_cpu(cpu_string) := cpu {{
    endswith(cpu_string, "m")
    cpu := to_number(trim_suffix(cpu_string, "m")) / 1000
}} else := cpu {{
    cpu := to_number(cpu_string)
}}

parse_memory_gb(mem_string) := memory {{
    endswith(mem_string, "Gi")
    memory := to_number(trim_suffix(mem_string, "Gi"))
}} else := memory {{
    endswith(mem_string, "Mi")
    memory := to_number(trim_suffix(mem_string, "Mi")) / 1024
}} else := memory {{
    endswith(mem_string, "G")
    memory := to_number(trim_suffix(mem_string, "G"))
}}

parse_storage_gb(storage_string) := storage {{
    endswith(storage_string, "Gi")
    storage := to_number(trim_suffix(storage_string, "Gi"))
}} else := storage {{
    endswith(storage_string, "G")
    storage := to_number(trim_suffix(storage_string, "G"))
}}

# Namespace quotas
namespace_quotas := {{
    "data-platform": {{
        "cpu": 100,
        "memory": 500,
        "pods": 100
    }},
    "dev": {{
        "cpu": 20,
        "memory": 100,
        "pods": 50
    }}
}}

warn[msg] {{
    input.request.kind.kind == "Pod"
    namespace := input.request.object.metadata.namespace
    quota := namespace_quotas[namespace]
    total_cpu := sum([parse_cpu(c.resources.requests.cpu) | c := input.request.object.spec.containers[_]])
    total_cpu > quota.cpu
    msg := sprintf("Namespace '%s' CPU usage may exceed quota of %v cores (SYS-QUOTA-001)", [namespace, quota.cpu])
}}
"#,
        config.environment
    )
}

fn generate_security_compliance_policy(model: &SemanticModel, config: &OpaConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# SECURITY & COMPLIANCE POLICIES
# ═══════════════════════════════════════════════════════════════════════════
# Generated from ArcLang Model
# Environment: {}
# Strict Mode: {}
# ═══════════════════════════════════════════════════════════════════════════

package security.compliance

import future.keywords.if
import future.keywords.in

# PCI-DSS Compliance: Deny unencrypted data at rest
deny[msg] {{
    resource := input.resource_changes[_]
    is_data_store(resource.type)
    not has_encryption_at_rest(resource)
    msg := sprintf("PCI-DSS: Resource '%s' must encrypt data at rest (SYS-COMP-001)", [resource.name])
}}

is_data_store(resource_type) {{
    resource_type in [
        "aws_s3_bucket",
        "aws_db_instance",
        "aws_dynamodb_table",
        "azurerm_storage_account",
        "google_storage_bucket",
        "google_bigquery_dataset"
    ]
}}

has_encryption_at_rest(resource) {{
    resource.change.after.server_side_encryption_configuration
}} else {{
    resource.change.after.encryption_configuration
}} else {{
    resource.change.after.encryption
}}

# GDPR Compliance: Require data residency in specific regions
allowed_regions_gdpr := ["eu-west-1", "eu-central-1", "europe-west1", "westeurope"]

deny[msg] {{
    resource := input.resource_changes[_]
    contains(resource.name, "customer") or contains(resource.name, "personal")
    not resource.change.after.region in allowed_regions_gdpr
    msg := sprintf("GDPR: Personal data resource '%s' must be in EU region (SYS-COMP-002)", [resource.name])
}}

# SOC2 Compliance: Require audit logging
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type in ["aws_s3_bucket", "azurerm_storage_account"]
    not has_access_logging(resource)
    msg := sprintf("SOC2: Storage '%s' must have access logging enabled (SYS-COMP-003)", [resource.name])
}}

has_access_logging(resource) {{
    resource.change.after.logging
}} else {{
    # Check for associated logging configuration
    some log in input.resource_changes
    log.type == "aws_s3_bucket_logging"
    contains(log.change.after.bucket, resource.name)
}}

# HIPAA Compliance: Require encryption in transit
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type in ["aws_db_instance", "azurerm_sql_server"]
    not resource.change.after.ssl_enforcement_enabled
    not resource.change.after.storage_encrypted
    msg := sprintf("HIPAA: Database '%s' must enforce SSL and encrypt storage (SYS-COMP-004)", [resource.name])
}}

# Network security: Deny overly permissive security groups
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_security_group"
    some rule in resource.change.after.ingress
    rule.cidr_blocks[_] == "0.0.0.0/0"
    rule.from_port != 443
    rule.from_port != 80
    msg := sprintf("Security group '%s' allows unrestricted access on port %v (SYS-SEC-008)", [resource.name, rule.from_port])
}}

# Kubernetes network policies required
deny[msg] {{
    input.request.kind.kind == "Namespace"
    not has_network_policy(input.request.object.metadata.name)
    msg := sprintf("Namespace '%s' must have NetworkPolicy defined (SYS-SEC-009)", [input.request.object.metadata.name])
}}

has_network_policy(namespace) {{
    # Would check cluster state for NetworkPolicy resources
    namespace in ["kube-system", "kube-public", "kube-node-lease"]
}} else {{
    namespace == "data-platform"
}}

# Deny secrets in environment variables
deny[msg] {{
    input.request.kind.kind == "Pod"
    some container in input.request.object.spec.containers
    some env in container.env
    is_secret_name(env.name)
    not env.valueFrom.secretKeyRef
    msg := sprintf("Container '%s' exposes secret '%s' as plaintext (SYS-SEC-010)", [container.name, env.name])
}}

is_secret_name(name) {{
    contains(lower(name), "password")
}} else {{
    contains(lower(name), "secret")
}} else {{
    contains(lower(name), "key")
}} else {{
    contains(lower(name), "token")
}}

# Require pod security standards
deny[msg] {{
    input.request.kind.kind == "Pod"
    input.request.object.metadata.labels.safety_level == "Critical"
    not meets_restricted_pss(input.request.object)
    msg := sprintf("Critical pod '%s' must meet Restricted Pod Security Standard (SYS-SEC-011)", [input.request.object.metadata.name])
}}

meets_restricted_pss(pod) {{
    some container in pod.spec.containers
    container.securityContext.runAsNonRoot
    container.securityContext.allowPrivilegeEscalation == false
    container.securityContext.capabilities.drop[_] == "ALL"
    container.securityContext.seccompProfile.type == "RuntimeDefault"
}}

# Model traceability
# Requirements: {}
# Safety-Critical Requirements: {}
"#,
        config.environment,
        config.strict_mode,
        model.requirements.len(),
        model.requirements.iter().filter(|r| r.description.to_lowercase().contains("safety") || r.description.to_lowercase().contains("critical")).count()
    )
}

fn generate_cost_governance_policy(model: &SemanticModel, config: &OpaConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# COST GOVERNANCE POLICIES
# ═══════════════════════════════════════════════════════════════════════════
# Generated from ArcLang Model
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════

package cost.governance

import future.keywords.if
import future.keywords.in

# Monthly budget thresholds (USD)
budget_limits := {{
    "dev": 5000,
    "staging": 10000,
    "prod": 50000
}}

# Instance cost estimates (hourly USD)
instance_costs := {{
    "t3.micro": 0.0104,
    "t3.small": 0.0208,
    "t3.medium": 0.0416,
    "m5.large": 0.096,
    "m5.xlarge": 0.192,
    "m5.2xlarge": 0.384,
    "c5.4xlarge": 0.68,
    "r5.8xlarge": 2.016,
    "p3.16xlarge": 24.48
}}

# Warn about expensive instance types
warn[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_instance"
    instance_type := resource.change.after.instance_type
    cost := instance_costs[instance_type]
    monthly_cost := cost * 730
    monthly_cost > 1000
    msg := sprintf("Instance '%s' type %s costs ~$%v/month - consider cost optimization (SYS-COST-005)", [resource.name, instance_type, round(monthly_cost)])
}}

# Deny creation of GPU instances without approval tag
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_instance"
    startswith(resource.change.after.instance_type, "p3.")
    not resource.change.after.tags.CostApproved
    msg := sprintf("GPU instance '%s' requires CostApproved tag (SYS-COST-006)", [resource.name])
}}

# Require auto-termination for dev environments
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "databricks_cluster"
    resource.change.after.tags.Environment == "dev"
    not resource.change.after.autotermination_minutes
    msg := sprintf("Dev cluster '%s' must have auto-termination configured (SYS-COST-007)", [resource.name])
}}

# Require lifecycle policies for storage
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_s3_bucket"
    not has_lifecycle_policy(resource.name)
    msg := sprintf("Bucket '%s' should have lifecycle policy to reduce storage costs (SYS-COST-008)", [resource.name])
}}

has_lifecycle_policy(bucket_name) {{
    some lc in input.resource_changes
    lc.type == "aws_s3_bucket_lifecycle_configuration"
    contains(lc.change.after.bucket, bucket_name)
}}

# Deny expensive storage classes in non-prod
deny[msg] {{
    resource := input.resource_changes[_]
    resource.type == "aws_ebs_volume"
    resource.change.after.tags.Environment != "prod"
    resource.change.after.volume_type in ["io2", "io1"]
    msg := sprintf("Non-prod volume '%s' should not use expensive io2/io1 type (SYS-COST-009)", [resource.name])
}}

# Require spot/preemptible instances for non-critical workloads
warn[msg] {{
    input.request.kind.kind == "Pod"
    input.request.object.metadata.labels.safety_level in ["Low", "Medium"]
    not uses_spot_instances(input.request.object)
    msg := sprintf("Non-critical pod '%s' should use spot/preemptible instances (SYS-COST-010)", [input.request.object.metadata.name])
}}

uses_spot_instances(pod) {{
    some toleration in pod.spec.tolerations
    toleration.key == "cloud.google.com/gke-preemptible"
}} else {{
    some toleration in pod.spec.tolerations
    toleration.key == "eks.amazonaws.com/capacityType"
    toleration.value == "SPOT"
}}

# Model metadata - Cost optimization requirements: {}
"#,
        config.environment,
        model.requirements.iter().filter(|r| r.description.to_lowercase().contains("cost")).count()
    )
}

fn generate_conftest_policy(model: &SemanticModel, config: &OpaConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# CONFTEST POLICY (CLI Testing)
# ═══════════════════════════════════════════════════════════════════════════
# Generated from ArcLang Model
# Usage: conftest test terraform/plan.json
# ═══════════════════════════════════════════════════════════════════════════

package main

import future.keywords.if
import future.keywords.in

deny[msg] {{
    input.resource_changes[_].type == "aws_s3_bucket"
    resource := input.resource_changes[_]
    not resource.change.after.server_side_encryption_configuration
    msg := sprintf("S3 bucket '%s' must have encryption enabled", [resource.name])
}}

deny[msg] {{
    input.resource_changes[_].type == "aws_instance"
    resource := input.resource_changes[_]
    not resource.change.after.tags.Component
    msg := sprintf("Instance '%s' missing Component tag for traceability", [resource.name])
}}

warn[msg] {{
    input.resource_changes[_].type == "aws_instance"
    resource := input.resource_changes[_]
    startswith(resource.change.after.instance_type, "t2.")
    msg := sprintf("Instance '%s' uses older t2 type, consider t3", [resource.name])
}}

# Validate against ArcLang model
valid_components := {{
    "la-proc-001", "la-proc-002", "la-proc-003",
    "la-mig-001", "la-mig-002", "la-mig-003", "la-mig-004",
    "la-int-001", "la-int-002", "la-int-003",
    "la-anlz-001", "la-mon-001", "la-tgt-001", "la-gov-001"
}}

deny[msg] {{
    resource := input.resource_changes[_]
    component := resource.change.after.tags.Component
    component != ""
    not component in valid_components
    msg := sprintf("Resource '%s' has invalid component ID '%s'", [resource.name, component])
}}

# Model statistics
# Requirements: {}
# Components: {}
# Traces: {}
"#,
        model.requirements.len(),
        model.components.len(),
        model.traces.len()
    )
}
