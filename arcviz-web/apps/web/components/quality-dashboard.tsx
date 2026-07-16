'use client';

/**
 * Quality Score Dashboard Component
 * 
 * Interactive dashboard for visualizing diagram quality metrics
 * Based on LaTeX Specification Section 7 (Quality Metrics)
 * 
 * Features:
 * - Overall quality score with gauge
 * - Individual metric breakdown
 * - Regulatory compliance status
 * - Violation details
 * - Real-time validation
 * - Export reports
 */

import React, { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Alert } from '@/components/ui/alert';

interface QualityMetrics {
  actorPlacement: MetricResult;
  systemBoundary: MetricResult;
  containmentValidity: MetricResult;
  edgeCrossings: MetricResult;
  portSideCorrectness: MetricResult;
  colorCompliance: MetricResult;
  gridAlignment: MetricResult;
  labelOverlap: MetricResult;
  flowDirection: MetricResult;
  whitespaceBalance: MetricResult;
  componentNesting: MetricResult;
  interfaceNotation: MetricResult;
  traceabilityLinks: MetricResult;
  safetyAnnotations: MetricResult;
  overallScore: number;
  qualityLevel: 'Excellent' | 'Good' | 'Acceptable' | 'Poor' | 'Unacceptable';
  regulatoryCompliance: {
    iso26262_asil_d: boolean;
    do178c_dal_a: boolean;
    iec61508_sil4: boolean;
    minScore: number;
  };
}

interface MetricResult {
  score: number;
  weight: number;
  pass: boolean;
  measurement: number;
  threshold: number;
  violations: string[];
  description: string;
}

interface QualityDashboardProps {
  metrics?: QualityMetrics;
  diagramType?: string;
  onRefresh?: () => void;
  onExportReport?: () => void;
}

export function QualityDashboard({
  metrics,
  diagramType,
  onRefresh,
  onExportReport,
}: QualityDashboardProps) {
  const [expandedMetric, setExpandedMetric] = useState<string | null>(null);

  if (!metrics) {
    return (
      <Card className="p-6">
        <div className="text-center text-gray-500">
          <p className="text-lg font-semibold mb-2">No Quality Metrics Available</p>
          <p className="text-sm">Generate a diagram to see quality metrics</p>
        </div>
      </Card>
    );
  }

  const qualityColor = getQualityColor(metrics.qualityLevel);
  const qualityIcon = getQualityIcon(metrics.qualityLevel);

  return (
    <div className="space-y-6">
      {/* Overall Score Card */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold">Diagram Quality Assessment</h2>
          <div className="flex gap-2">
            {onRefresh && (
              <Button variant="outline" size="sm" onClick={onRefresh}>
                ↻ Refresh
              </Button>
            )}
            {onExportReport && (
              <Button variant="outline" size="sm" onClick={onExportReport}>
                📄 Export Report
              </Button>
            )}
          </div>
        </div>

        {diagramType && (
          <p className="text-sm text-gray-600 mb-4">
            Diagram Type: <span className="font-semibold">{diagramType}</span>
          </p>
        )}

        {/* Score Gauge */}
        <div className="flex items-center gap-6 mb-6">
          <div className="relative w-40 h-40">
            <CircularGauge score={metrics.overallScore} color={qualityColor} />
          </div>
          <div>
            <div className="flex items-center gap-3 mb-2">
              <span className="text-4xl">{qualityIcon}</span>
              <Badge
                className={`text-lg px-4 py-1`}
                style={{ backgroundColor: qualityColor, color: 'white' }}
              >
                {metrics.qualityLevel}
              </Badge>
            </div>
            <p className="text-3xl font-bold" style={{ color: qualityColor }}>
              {metrics.overallScore.toFixed(1)} / 100
            </p>
            <p className="text-sm text-gray-600 mt-2">
              {getQualityDescription(metrics.qualityLevel)}
            </p>
          </div>
        </div>

        {/* Regulatory Compliance */}
        <div className="border-t pt-4">
          <h3 className="font-semibold mb-3">Regulatory Compliance</h3>
          <div className="grid grid-cols-3 gap-4">
            <ComplianceCard
              standard="ISO 26262 ASIL-D"
              required={85}
              actual={metrics.overallScore}
              pass={metrics.regulatoryCompliance.iso26262_asil_d}
            />
            <ComplianceCard
              standard="DO-178C DAL-A"
              required={90}
              actual={metrics.overallScore}
              pass={metrics.regulatoryCompliance.do178c_dal_a}
            />
            <ComplianceCard
              standard="IEC 61508 SIL-4"
              required={85}
              actual={metrics.overallScore}
              pass={metrics.regulatoryCompliance.iec61508_sil4}
            />
          </div>
        </div>
      </Card>

      {/* Metrics Breakdown */}
      <Card className="p-6">
        <h3 className="text-xl font-bold mb-4">Detailed Metrics</h3>
        <div className="space-y-3">
          <MetricRow
            name="Actor Placement"
            metric={metrics.actorPlacement}
            critical
            expanded={expandedMetric === 'actorPlacement'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'actorPlacement' ? null : 'actorPlacement'
              )
            }
          />
          <MetricRow
            name="System Boundary"
            metric={metrics.systemBoundary}
            critical
            expanded={expandedMetric === 'systemBoundary'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'systemBoundary' ? null : 'systemBoundary'
              )
            }
          />
          <MetricRow
            name="Containment Validity"
            metric={metrics.containmentValidity}
            critical
            expanded={expandedMetric === 'containmentValidity'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'containmentValidity' ? null : 'containmentValidity'
              )
            }
          />
          <MetricRow
            name="Edge Crossings"
            metric={metrics.edgeCrossings}
            expanded={expandedMetric === 'edgeCrossings'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'edgeCrossings' ? null : 'edgeCrossings'
              )
            }
          />
          <MetricRow
            name="Port Side Correctness"
            metric={metrics.portSideCorrectness}
            expanded={expandedMetric === 'portSideCorrectness'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'portSideCorrectness' ? null : 'portSideCorrectness'
              )
            }
          />
          <MetricRow
            name="Color Compliance"
            metric={metrics.colorCompliance}
            expanded={expandedMetric === 'colorCompliance'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'colorCompliance' ? null : 'colorCompliance'
              )
            }
          />
          <MetricRow
            name="Grid Alignment"
            metric={metrics.gridAlignment}
            expanded={expandedMetric === 'gridAlignment'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'gridAlignment' ? null : 'gridAlignment'
              )
            }
          />
          <MetricRow
            name="Label Overlap"
            metric={metrics.labelOverlap}
            critical
            expanded={expandedMetric === 'labelOverlap'}
            onToggle={() =>
              setExpandedMetric(
                expandedMetric === 'labelOverlap' ? null : 'labelOverlap'
              )
            }
          />
        </div>
      </Card>
    </div>
  );
}

function CircularGauge({ score, color }: { score: number; color: string }) {
  const radius = 70;
  const strokeWidth = 12;
  const normalizedRadius = radius - strokeWidth / 2;
  const circumference = normalizedRadius * 2 * Math.PI;
  const strokeDashoffset = circumference - (score / 100) * circumference;

  return (
    <svg height={radius * 2} width={radius * 2} className="transform -rotate-90">
      <circle
        stroke="#e5e7eb"
        fill="transparent"
        strokeWidth={strokeWidth}
        r={normalizedRadius}
        cx={radius}
        cy={radius}
      />
      <circle
        stroke={color}
        fill="transparent"
        strokeWidth={strokeWidth}
        strokeDasharray={circumference + ' ' + circumference}
        style={{ strokeDashoffset, transition: 'stroke-dashoffset 0.5s ease' }}
        strokeLinecap="round"
        r={normalizedRadius}
        cx={radius}
        cy={radius}
      />
      <text
        x="50%"
        y="50%"
        className="transform rotate-90"
        textAnchor="middle"
        dy=".3em"
        fontSize="24"
        fontWeight="bold"
        fill={color}
      >
        {score.toFixed(0)}
      </text>
    </svg>
  );
}

function ComplianceCard({
  standard,
  required,
  actual,
  pass,
}: {
  standard: string;
  required: number;
  actual: number;
  pass: boolean;
}) {
  return (
    <div className="border rounded-lg p-3">
      <div className="flex items-center justify-between mb-2">
        <span className="font-semibold text-sm">{standard}</span>
        <Badge variant={pass ? 'default' : 'destructive'}>
          {pass ? '✓ PASS' : '✗ FAIL'}
        </Badge>
      </div>
      <p className="text-xs text-gray-600">
        Required: {required} | Actual: {actual.toFixed(1)}
      </p>
    </div>
  );
}

function MetricRow({
  name,
  metric,
  critical = false,
  expanded,
  onToggle,
}: {
  name: string;
  metric: MetricResult;
  critical?: boolean;
  expanded: boolean;
  onToggle: () => void;
}) {
  const scoreColor = metric.pass ? '#10b981' : '#ef4444';

  return (
    <div className="border rounded-lg overflow-hidden">
      <button
        className="w-full p-4 flex items-center justify-between hover:bg-gray-50 transition-colors"
        onClick={onToggle}
      >
        <div className="flex items-center gap-3 flex-1">
          <div className="flex items-center gap-2">
            {metric.pass ? (
              <span className="text-green-500 text-xl">✓</span>
            ) : (
              <span className="text-red-500 text-xl">✗</span>
            )}
            <span className="font-semibold">{name}</span>
            {critical && (
              <Badge variant="destructive" className="text-xs">
                CRITICAL
              </Badge>
            )}
          </div>
          <div className="flex-1 mx-4">
            <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
              <div
                className="h-full transition-all duration-300"
                style={{
                  width: `${metric.score}%`,
                  backgroundColor: scoreColor,
                }}
              />
            </div>
          </div>
          <span className="font-bold text-lg" style={{ color: scoreColor }}>
            {metric.score.toFixed(0)}
          </span>
        </div>
        <span className="text-gray-400 ml-2">{expanded ? '▼' : '▶'}</span>
      </button>

      {expanded && (
        <div className="border-t bg-gray-50 p-4 space-y-3">
          <p className="text-sm text-gray-700">{metric.description}</p>

          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-gray-600">Measurement:</span>
              <span className="ml-2 font-semibold">{metric.measurement}</span>
            </div>
            <div>
              <span className="text-gray-600">Threshold:</span>
              <span className="ml-2 font-semibold">{metric.threshold}</span>
            </div>
          </div>

          {metric.violations.length > 0 && (
            <div>
              <p className="text-sm font-semibold text-red-600 mb-2">
                Violations ({metric.violations.length}):
              </p>
              <ul className="text-xs space-y-1 text-gray-700">
                {metric.violations.slice(0, 5).map((violation, i) => (
                  <li key={i} className="pl-2 border-l-2 border-red-300">
                    {violation}
                  </li>
                ))}
                {metric.violations.length > 5 && (
                  <li className="pl-2 text-gray-500 italic">
                    ... and {metric.violations.length - 5} more
                  </li>
                )}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function getQualityColor(level: string): string {
  switch (level) {
    case 'Excellent':
      return '#10b981';
    case 'Good':
      return '#3b82f6';
    case 'Acceptable':
      return '#f59e0b';
    case 'Poor':
      return '#ef4444';
    case 'Unacceptable':
      return '#991b1b';
    default:
      return '#6b7280';
  }
}

function getQualityIcon(level: string): string {
  switch (level) {
    case 'Excellent':
      return '🌟';
    case 'Good':
      return '✅';
    case 'Acceptable':
      return '⚠️';
    case 'Poor':
      return '❌';
    case 'Unacceptable':
      return '🚫';
    default:
      return '❓';
  }
}

function getQualityDescription(level: string): string {
  switch (level) {
    case 'Excellent':
      return 'Production-ready, passes all audits';
    case 'Good':
      return 'Minor improvements needed';
    case 'Acceptable':
      return 'Significant improvements required';
    case 'Poor':
      return 'Major rework necessary';
    case 'Unacceptable':
      return 'Does not meet standards';
    default:
      return '';
  }
}
