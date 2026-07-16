// ArcViz Capella Engine - Configuration Centrale
// Replicates the configuration from arcviz_explorer_template.html

export const ARCVIZ_CONFIG = {
  // Layout Engine Selection
  engine: 'elk' as const, // 'elk' (default) | 'dagre' (fallback)

  // ELK Layout Configuration (Primary) - Optimized for Capella
  elk: {
    algorithm: 'layered',
    'elk.direction': 'DOWN',
    'elk.hierarchyHandling': 'INCLUDE_CHILDREN',

    // Layer spacing (Capella swimlanes)
    'elk.layered.spacing.nodeNodeBetweenLayers': '250',
    'elk.spacing.nodeNode': '100',
    'elk.spacing.edgeNode': '50',
    'elk.spacing.edgeEdge': '30',

    // Port positioning (Capella interfaces)
    'elk.portConstraints': 'FIXED_SIDE',
    'elk.port.borderOffset': '0',
    'elk.spacing.portPort': '50',

    // Edge routing (Capella orthogonal style)
    'elk.edgeRouting': 'ORTHOGONAL',
    'elk.layered.edgeRouting.sloppiness': '0.3',
    'elk.layered.edgeRouting.minimizeNumBends': 'true',

    // Node placement (optimal layout)
    'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
    'elk.layered.nodePlacement.favorStraightEdges': 'true',
    'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
    'elk.layered.thoroughness': '150',

    // Layer separation
    'elk.separateConnectedComponents': 'false',
    'elk.layered.compaction.postCompaction.strategy': 'EDGE_LENGTH',

    // Label placement
    'elk.edgeLabels.placement': 'CENTER',
    'elk.edgeLabels.inline': 'true',
  },

  // Dagre Layout Configuration (Fallback)
  dagre: {
    rankdir: 'TB', // Direction: TB (top-bottom), LR (left-right)
    nodesep: 350, // Horizontal spacing between nodes (px)
    ranksep: 200, // Vertical spacing between layers (px)
    marginx: 150, // Left/right margins (px)
    marginy: 100, // Top/bottom margins (px)
    edgesep: 100, // Spacing between edges (px)
    ranker: 'network-simplex', // Layout algorithm
  },

  // Component Node Configuration (Capella-compliant)
  node: {
    defaultWidth: 320, // Default node width (px)
    defaultHeight: 200, // Default node height (px)
    minWidth: 300, // Minimum node width (px)
    minHeight: 160, // Minimum node height (px)
    maxWidth: 700, // Maximum node width (px)
    maxHeight: 600, // Maximum node height (px)
    borderRadius: 6, // Component border radius (px)
    headerHeight: 65, // Header section height (px)
    padding: 15, // Internal padding (px)
  },

  // Port Configuration (Capella interface style)
  port: {
    size: 14, // Port square size (px)
    spacing: 55, // Vertical spacing between ports (px)
    borderRadius: 3, // Port corner radius (px)
    edgeOffset: 7, // Distance from component edge (size/2)
    labelGap: 8, // Gap between port and label (px)
    nameYOffset: 3, // Y offset for port name (px)
    protocolYOffset: 16, // Y offset for protocol label (px)
    colors: {
      inFill: '#4caf50', // IN port fill (Capella green)
      inStroke: '#2e7d32', // IN port border
      outFill: '#ff9800', // OUT port fill (Capella orange)
      outStroke: '#e65100', // OUT port border
    },
  },

  // Label Configuration
  label: {
    portName: {
      fontSize: 9, // Port name font size (px)
      fontWeight: 600, // Port name font weight
      maxLength: 15, // Max characters before truncate
      color: '#263238', // Text color
    },
    protocol: {
      fontSize: 7, // Protocol font size (px)
      fontStyle: 'italic', // Font style
      maxLength: 12, // Max characters before truncate
      color: '#546e7a', // Text color
    },
    function: {
      fontSize: 10, // Function name font size (px)
      fontWeight: 500, // Function font weight
      lineHeight: 18, // Line spacing (px)
      color: '#37474f', // Text color
    },
    stereotype: {
      fontSize: 16, // Icon size (px)
      xOffset: 10, // X offset from left (px)
      yOffset: 36, // Y offset from top (px)
    },
  },

  // Function List Configuration
  functions: {
    lineHeight: 18, // Line spacing (px)
    fontSize: 10, // Font size (px)
    xOffset: 12, // Indent from left (px)
    yOffset: 16, // Offset from header (px)
    moreIndicatorSize: 9, // "more..." font size (px)
    portReserveMultiplier: 50, // Space per port
    minPortArea: 100, // Minimum space for ports (px)
  },

  // Layer Swimlane Configuration
  layer: {
    padding: {
      left: 30,
      right: 30,
      top: 50,
      bottom: 30,
    },
    labelFont: 16, // Layer label font size (px)
    labelColor: '#263238', // Layer label color
    borderDash: [8, 4], // Border dash pattern
    borderWidth: 2, // Border width (px)
    borderRadius: 12, // Corner radius (px)
    colors: {
      operational: 'rgba(255, 152, 0, 0.08)', // Orange
      system: 'rgba(76, 175, 80, 0.08)', // Green
      logical: 'rgba(33, 150, 243, 0.08)', // Blue
      physical: 'rgba(156, 39, 176, 0.08)', // Purple
      epbs: 'rgba(121, 85, 72, 0.08)', // Brown
    },
    borderColors: {
      operational: '#ff9800',
      system: '#4caf50',
      logical: '#2196f3',
      physical: '#9c27b0',
      epbs: '#795548',
    },
  },

  // Edge Configuration
  edge: {
    strokeWidth: 2.5, // Edge line width (px)
    color: '#607d8b', // Edge color
    arrowhead: 'vee', // Arrowhead style
    labelBox: {
      width: 80, // Exchange item label width (px)
      height: 24, // Exchange item label height (px)
      fontSize: 9, // Label font size (px)
      maxLength: 12, // Max characters
      borderRadius: 4, // Corner radius (px)
      fill: 'white', // Background color
      stroke: '#b0bec5', // Border color
    },
  },

  // Safety Badge Configuration
  safety: {
    radius: 12, // Badge circle radius (px)
    fontSize: 7, // Badge font size (px)
    fontWeight: 'bold', // Font weight
    xOffset: 20, // Offset from right edge (px)
    yOffset: 20, // Offset from top (px)
    colors: {
      ASIL_A: '#10b981',
      ASIL_B: '#ff9800', // Orange for ASIL B
      ASIL_C: '#f44336', // Red for ASIL C
      ASIL_D: '#d32f2f', // Dark red for ASIL D
      DAL_A: '#d32f2f',
      DAL_B: '#f44336',
      DAL_C: '#ff9800',
      DAL_D: '#10b981',
      QM: '#6366f1',
    },
  },

  // Auto-sizing Configuration
  autoSize: {
    enabled: true, // Enable automatic node sizing
    widthPerChar: 7, // Pixels per character for width
    heightPerFunction: 18, // Pixels per function for height
    heightPerPort: 50, // Pixels per port for height
    minPadding: 40, // Minimum padding (px)
  },
}

export type ArcVizConfig = typeof ARCVIZ_CONFIG
