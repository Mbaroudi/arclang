// ═══════════════════════════════════════════════════════════════════════════
// KUBERNETES & HELM GENERATOR
// ═══════════════════════════════════════════════════════════════════════════
// Purpose: Generate Kubernetes manifests and Helm charts from ArcLang model
// Coverage: Deployments, Services, ConfigMaps, Secrets, HPA, Ingress
// Target: Kubernetes 1.28+, Helm 3.x
// ═══════════════════════════════════════════════════════════════════════════

use super::semantic::{SemanticModel, ComponentInfo};
use super::CompilerError;
use std::collections::HashMap;

pub fn generate_kubernetes_manifests(
    model: &SemanticModel,
    config: &KubernetesConfig,
) -> Result<String, CompilerError> {
    let mut output = String::new();
    
    output.push_str(&generate_k8s_header(config));
    output.push_str("\n---\n");
    output.push_str(&generate_namespace(config));
    output.push_str("\n---\n");
    output.push_str(&generate_deployments(model, config));
    output.push_str("\n---\n");
    output.push_str(&generate_services(model, config));
    output.push_str("\n---\n");
    output.push_str(&generate_configmaps(model, config));
    output.push_str("\n---\n");
    output.push_str(&generate_secrets(model, config));
    output.push_str("\n---\n");
    output.push_str(&generate_hpa(model, config));
    output.push_str("\n---\n");
    output.push_str(&generate_ingress(model, config));
    
    Ok(output)
}

pub fn generate_helm_chart(
    model: &SemanticModel,
    config: &KubernetesConfig,
) -> Result<HelmChart, CompilerError> {
    Ok(HelmChart {
        chart_yaml: generate_chart_yaml(config),
        values_yaml: generate_values_yaml(model, config),
        templates: vec![
            ("deployment.yaml".to_string(), generate_helm_deployment_template(model, config)),
            ("service.yaml".to_string(), generate_helm_service_template(model, config)),
            ("configmap.yaml".to_string(), generate_helm_configmap_template(model, config)),
            ("hpa.yaml".to_string(), generate_helm_hpa_template(model, config)),
            ("ingress.yaml".to_string(), generate_helm_ingress_template(model, config)),
        ],
    })
}

#[derive(Debug, Clone)]
pub struct KubernetesConfig {
    pub namespace: String,
    pub environment: String,
    pub registry: String,
    pub image_pull_policy: String,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        KubernetesConfig {
            namespace: "data-platform".to_string(),
            environment: "prod".to_string(),
            registry: "gcr.io/my-project".to_string(),
            image_pull_policy: "IfNotPresent".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct HelmChart {
    pub chart_yaml: String,
    pub values_yaml: String,
    pub templates: Vec<(String, String)>,
}

fn generate_k8s_header(config: &KubernetesConfig) -> String {
    format!(
r#"# ═══════════════════════════════════════════════════════════════════════════
# KUBERNETES MANIFESTS - GENERATED FROM ARCLANG MODEL
# ═══════════════════════════════════════════════════════════════════════════
# Generated: {}
# Namespace: {}
# Environment: {}
# ═══════════════════════════════════════════════════════════════════════════"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.namespace,
        config.environment
    )
}

fn generate_namespace(config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: v1
kind: Namespace
metadata:
  name: {}
  labels:
    environment: {}
    managed-by: arclang"#,
        config.namespace,
        config.environment
    )
}

fn generate_deployments(model: &SemanticModel, config: &KubernetesConfig) -> String {
    let mut output = String::new();
    
    output.push_str(&format!(
r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: etl-orchestrator
  namespace: {}
  labels:
    app: etl-orchestrator
    component: la-mig-001
    environment: {}
spec:
  replicas: 2
  selector:
    matchLabels:
      app: etl-orchestrator
  template:
    metadata:
      labels:
        app: etl-orchestrator
        component: la-mig-001
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: etl-orchestrator
      containers:
      - name: orchestrator
        image: {}/etl-orchestrator:latest
        imagePullPolicy: {}
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        env:
        - name: ENVIRONMENT
          value: "{}"
        - name: LOG_LEVEL
          value: "info"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-credentials
              key: url
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /etc/config
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: etl-orchestrator-config"#,
        config.namespace,
        config.environment,
        config.registry,
        config.image_pull_policy,
        config.environment
    ));
    
    output.push_str("\n---\n");
    
    output.push_str(&format!(
r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: data-processor
  namespace: {}
  labels:
    app: data-processor
    component: la-proc-001
    environment: {}
spec:
  replicas: 3
  selector:
    matchLabels:
      app: data-processor
  template:
    metadata:
      labels:
        app: data-processor
        component: la-proc-001
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
    spec:
      serviceAccountName: data-processor
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - data-processor
              topologyKey: kubernetes.io/hostname
      containers:
      - name: processor
        image: {}/data-processor:latest
        imagePullPolicy: {}
        ports:
        - containerPort: 9090
          name: metrics
        env:
        - name: ENVIRONMENT
          value: "{}"
        - name: WORKER_THREADS
          value: "8"
        - name: BATCH_SIZE
          value: "1000"
        envFrom:
        - configMapRef:
            name: data-processor-config
        resources:
          requests:
            cpu: "2000m"
            memory: "8Gi"
          limits:
            cpu: "8000m"
            memory: "32Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 9090
          initialDelaySeconds: 60
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 30
          periodSeconds: 10
      tolerations:
      - key: workload
        operator: Equal
        value: data-processing
        effect: NoSchedule"#,
        config.namespace,
        config.environment,
        config.registry,
        config.image_pull_policy,
        config.environment
    ));
    
    output.push_str("\n---\n");
    
    output.push_str(&format!(
r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: {}
  labels:
    app: api-gateway
    component: la-int-001
    environment: {}
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
        component: la-int-001
        version: v1
    spec:
      containers:
      - name: gateway
        image: {}/api-gateway:latest
        imagePullPolicy: {}
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 8443
          name: https
        env:
        - name: ENVIRONMENT
          value: "{}"
        - name: RATE_LIMIT
          value: "1000"
        resources:
          requests:
            cpu: "500m"
            memory: "512Mi"
          limits:
            cpu: "1000m"
            memory: "2Gi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5"#,
        config.namespace,
        config.environment,
        config.registry,
        config.image_pull_policy,
        config.environment
    ));
    
    output
}

fn generate_services(model: &SemanticModel, config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: v1
kind: Service
metadata:
  name: etl-orchestrator
  namespace: {}
  labels:
    app: etl-orchestrator
spec:
  type: ClusterIP
  selector:
    app: etl-orchestrator
  ports:
  - port: 8080
    targetPort: 8080
    protocol: TCP
    name: http
---
apiVersion: v1
kind: Service
metadata:
  name: data-processor
  namespace: {}
  labels:
    app: data-processor
spec:
  type: ClusterIP
  selector:
    app: data-processor
  ports:
  - port: 9090
    targetPort: 9090
    protocol: TCP
    name: metrics
---
apiVersion: v1
kind: Service
metadata:
  name: api-gateway
  namespace: {}
  labels:
    app: api-gateway
spec:
  type: LoadBalancer
  selector:
    app: api-gateway
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
    name: http
  - port: 443
    targetPort: 8443
    protocol: TCP
    name: https"#,
        config.namespace,
        config.namespace,
        config.namespace
    )
}

fn generate_configmaps(model: &SemanticModel, config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: etl-orchestrator-config
  namespace: {}
data:
  config.yaml: |
    environment: {}
    orchestration:
      max_concurrent_jobs: 10
      job_timeout: 3600
      retry_attempts: 3
    storage:
      bronze_path: gs://data-platform-bronze
      silver_path: gs://data-platform-silver
      gold_path: gs://data-platform-gold
    monitoring:
      metrics_port: 8080
      log_level: info
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: data-processor-config
  namespace: {}
data:
  WORKER_THREADS: "8"
  BATCH_SIZE: "1000"
  MAX_MEMORY: "30Gi"
  CHECKPOINT_INTERVAL: "300""#,
        config.namespace,
        config.environment,
        config.namespace
    )
}

fn generate_secrets(model: &SemanticModel, config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: v1
kind: Secret
metadata:
  name: database-credentials
  namespace: {}
type: Opaque
stringData:
  url: "postgresql://user:password@postgres.{}.svc.cluster.local:5432/dataplatform"
  username: "dataplatform_user"
  password: "REPLACE_WITH_ACTUAL_PASSWORD"
---
apiVersion: v1
kind: Secret
metadata:
  name: cloud-credentials
  namespace: {}
type: Opaque
stringData:
  gcp-service-account.json: |
    {{
      "type": "service_account",
      "project_id": "my-project",
      "private_key_id": "REPLACE_ME",
      "private_key": "REPLACE_ME"
    }}"#,
        config.namespace,
        config.namespace,
        config.namespace
    )
}

fn generate_hpa(model: &SemanticModel, config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: etl-orchestrator-hpa
  namespace: {}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: etl-orchestrator
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
      - type: Pods
        value: 2
        periodSeconds: 30
      selectPolicy: Max
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: data-processor-hpa
  namespace: {}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: data-processor
  minReplicas: 3
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 75
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 85"#,
        config.namespace,
        config.namespace
    )
}

fn generate_ingress(model: &SemanticModel, config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: data-platform-ingress
  namespace: {}
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  tls:
  - hosts:
    - api.dataplatform.example.com
    secretName: api-tls-secret
  rules:
  - host: api.dataplatform.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 80"#,
        config.namespace
    )
}

fn generate_chart_yaml(config: &KubernetesConfig) -> String {
    format!(
r#"apiVersion: v2
name: data-platform
description: Data Platform Migration System
type: application
version: 1.0.0
appVersion: "1.0.0"
keywords:
  - data-platform
  - etl
  - migration
home: https://github.com/your-org/data-platform
sources:
  - https://github.com/your-org/data-platform
maintainers:
  - name: Data Engineering Team
    email: data-engineering@company.com
dependencies: []"#
    )
}

fn generate_values_yaml(model: &SemanticModel, config: &KubernetesConfig) -> String {
    format!(
r#"# Default values for data-platform Helm chart
# Generated from ArcLang model

global:
  environment: {}
  namespace: {}
  registry: {}
  imagePullPolicy: {}

etlOrchestrator:
  enabled: true
  replicaCount: 2
  image:
    repository: {}/etl-orchestrator
    tag: latest
  resources:
    requests:
      cpu: 500m
      memory: 1Gi
    limits:
      cpu: 2000m
      memory: 4Gi
  autoscaling:
    enabled: true
    minReplicas: 2
    maxReplicas: 10
    targetCPUUtilizationPercentage: 70
  service:
    type: ClusterIP
    port: 8080
  labels:
    component: la-mig-001
    requirement: sys-mig-005

dataProcessor:
  enabled: true
  replicaCount: 3
  image:
    repository: {}/data-processor
    tag: latest
  resources:
    requests:
      cpu: 2000m
      memory: 8Gi
    limits:
      cpu: 8000m
      memory: 32Gi
  autoscaling:
    enabled: true
    minReplicas: 3
    maxReplicas: 50
    targetCPUUtilizationPercentage: 75
  service:
    type: ClusterIP
    port: 9090
  labels:
    component: la-proc-001
    requirement: sys-scale-001
  tolerations:
  - key: workload
    operator: Equal
    value: data-processing
    effect: NoSchedule

apiGateway:
  enabled: true
  replicaCount: 3
  image:
    repository: {}/api-gateway
    tag: latest
  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: 1000m
      memory: 2Gi
  service:
    type: LoadBalancer
    httpPort: 80
    httpsPort: 443
  labels:
    component: la-int-001
    physical_node: pa-int-001

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
  hosts:
  - host: api.dataplatform.example.com
    paths:
    - path: /
      pathType: Prefix
  tls:
  - secretName: api-tls-secret
    hosts:
    - api.dataplatform.example.com

config:
  orchestration:
    maxConcurrentJobs: 10
    jobTimeout: 3600
    retryAttempts: 3
  storage:
    bronzePath: gs://data-platform-bronze
    silverPath: gs://data-platform-silver
    goldPath: gs://data-platform-gold
  monitoring:
    metricsPort: 8080
    logLevel: info

secrets:
  database:
    url: postgresql://user:password@postgres:5432/dataplatform
    username: dataplatform_user
    password: REPLACE_WITH_ACTUAL_PASSWORD

# Model metadata
modelMetadata:
  requirements: {}
  components: {}
  traces: {}
  generatedFrom: ArcLang"#,
        config.environment,
        config.namespace,
        config.registry,
        config.image_pull_policy,
        config.registry,
        config.registry,
        config.registry,
        model.requirements.len(),
        model.components.len(),
        model.traces.len()
    )
}

fn generate_helm_deployment_template(_model: &SemanticModel, _config: &KubernetesConfig) -> String {
r#"{{- range $app := list "etlOrchestrator" "dataProcessor" "apiGateway" }}
{{- $appConfig := index $.Values $app }}
{{- if $appConfig.enabled }}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ $app | kebabcase }}
  namespace: {{ $.Values.global.namespace }}
  labels:
    app: {{ $app | kebabcase }}
    {{- range $key, $val := $appConfig.labels }}
    {{ $key }}: {{ $val }}
    {{- end }}
    environment: {{ $.Values.global.environment }}
spec:
  replicas: {{ $appConfig.replicaCount }}
  selector:
    matchLabels:
      app: {{ $app | kebabcase }}
  template:
    metadata:
      labels:
        app: {{ $app | kebabcase }}
        version: v1
        {{- range $key, $val := $appConfig.labels }}
        {{ $key }}: {{ $val }}
        {{- end }}
    spec:
      containers:
      - name: {{ $app | kebabcase }}
        image: "{{ $appConfig.image.repository }}:{{ $appConfig.image.tag }}"
        imagePullPolicy: {{ $.Values.global.imagePullPolicy }}
        ports:
        - containerPort: {{ $appConfig.service.port }}
          name: http
        env:
        - name: ENVIRONMENT
          value: {{ $.Values.global.environment }}
        resources:
          {{- toYaml $appConfig.resources | nindent 10 }}
        livenessProbe:
          httpGet:
            path: /health
            port: {{ $appConfig.service.port }}
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: {{ $appConfig.service.port }}
          initialDelaySeconds: 10
          periodSeconds: 5
      {{- with $appConfig.tolerations }}
      tolerations:
        {{- toYaml . | nindent 6 }}
      {{- end }}
{{- end }}
{{- end }}"#.to_string()
}

fn generate_helm_service_template(_model: &SemanticModel, _config: &KubernetesConfig) -> String {
r#"{{- range $app := list "etlOrchestrator" "dataProcessor" "apiGateway" }}
{{- $appConfig := index $.Values $app }}
{{- if $appConfig.enabled }}
---
apiVersion: v1
kind: Service
metadata:
  name: {{ $app | kebabcase }}
  namespace: {{ $.Values.global.namespace }}
  labels:
    app: {{ $app | kebabcase }}
spec:
  type: {{ $appConfig.service.type }}
  selector:
    app: {{ $app | kebabcase }}
  ports:
  - port: {{ $appConfig.service.port }}
    targetPort: {{ $appConfig.service.port }}
    protocol: TCP
    name: http
{{- end }}
{{- end }}"#.to_string()
}

fn generate_helm_configmap_template(_model: &SemanticModel, _config: &KubernetesConfig) -> String {
r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ .Release.Name }}-config
  namespace: {{ .Values.global.namespace }}
data:
  config.yaml: |
    environment: {{ .Values.global.environment }}
    orchestration:
      max_concurrent_jobs: {{ .Values.config.orchestration.maxConcurrentJobs }}
      job_timeout: {{ .Values.config.orchestration.jobTimeout }}
      retry_attempts: {{ .Values.config.orchestration.retryAttempts }}
    storage:
      bronze_path: {{ .Values.config.storage.bronzePath }}
      silver_path: {{ .Values.config.storage.silverPath }}
      gold_path: {{ .Values.config.storage.goldPath }}
    monitoring:
      metrics_port: {{ .Values.config.monitoring.metricsPort }}
      log_level: {{ .Values.config.monitoring.logLevel }}"#.to_string()
}

fn generate_helm_hpa_template(_model: &SemanticModel, _config: &KubernetesConfig) -> String {
r#"{{- range $app := list "etlOrchestrator" "dataProcessor" }}
{{- $appConfig := index $.Values $app }}
{{- if and $appConfig.enabled $appConfig.autoscaling.enabled }}
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ $app | kebabcase }}-hpa
  namespace: {{ $.Values.global.namespace }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ $app | kebabcase }}
  minReplicas: {{ $appConfig.autoscaling.minReplicas }}
  maxReplicas: {{ $appConfig.autoscaling.maxReplicas }}
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: {{ $appConfig.autoscaling.targetCPUUtilizationPercentage }}
{{- end }}
{{- end }}"#.to_string()
}

fn generate_helm_ingress_template(_model: &SemanticModel, _config: &KubernetesConfig) -> String {
r#"{{- if .Values.ingress.enabled }}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ .Release.Name }}-ingress
  namespace: {{ .Values.global.namespace }}
  annotations:
    {{- range $key, $val := .Values.ingress.annotations }}
    {{ $key }}: {{ $val | quote }}
    {{- end }}
spec:
  ingressClassName: {{ .Values.ingress.className }}
  {{- if .Values.ingress.tls }}
  tls:
  {{- range .Values.ingress.tls }}
  - hosts:
    {{- range .hosts }}
    - {{ . }}
    {{- end }}
    secretName: {{ .secretName }}
  {{- end }}
  {{- end }}
  rules:
  {{- range .Values.ingress.hosts }}
  - host: {{ .host }}
    http:
      paths:
      {{- range .paths }}
      - path: {{ .path }}
        pathType: {{ .pathType }}
        backend:
          service:
            name: api-gateway
            port:
              number: 80
      {{- end }}
  {{- end }}
{{- end }}"#.to_string()
}
