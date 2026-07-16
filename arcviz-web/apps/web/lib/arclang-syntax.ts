// ArcLang Monaco Editor Language Definition

export const arclangLanguage = {
  defaultToken: '',
  tokenPostfix: '.arc',

  keywords: [
    'operational_analysis',
    'system_analysis',
    'logical_architecture',
    'physical_architecture',
    'epbs',
    'safety_analysis',
    'actor',
    'operational_capability',
    'operational_activity',
    'requirement',
    'system_function',
    'system_component',
    'component',
    'function',
    'interface',
    'node',
    'physical_link',
    'deploys',
    'trace',
    'satisfies',
    'implements',
    'realizes',
    'item',
    'subsystem',
    'system',
    'hazard',
    'fmea',
    'fault_tree',
  ],

  attributes: [
    'id',
    'description',
    'type',
    'component_type',
    'interface_type',
    'priority',
    'category',
    'safety_level',
    'from',
    'to',
    'protocol',
    'rate',
    'bandwidth',
    'latency',
    'processor',
    'memory',
    'supplier',
    'part_number',
    'version',
    'traces',
    'rationale',
    'verification_method',
    'standard',
    'asil',
    'dal',
    'severity',
    'likelihood',
    'classification',
  ],

  safetyLevels: [
    'ASIL_A',
    'ASIL_B',
    'ASIL_C',
    'ASIL_D',
    'DAL_A',
    'DAL_B',
    'DAL_C',
    'DAL_D',
    'QM',
  ],

  operators: [':', ',', '{', '}', '[', ']'],

  symbols: /[=><!~?:&|+\-*\/\^%]+/,

  escapes: /\\(?:[abfnrtv\\"']|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,

  tokenizer: {
    root: [
      // Keywords
      [
        /[a-z_$][\w$]*/,
        {
          cases: {
            '@keywords': 'keyword',
            '@attributes': 'attribute',
            '@safetyLevels': 'type',
            '@default': 'identifier',
          },
        },
      ],

      // Whitespace
      { include: '@whitespace' },

      // Delimiters
      [/[{}()\[\]]/, '@brackets'],
      [/[<>](?!@symbols)/, '@brackets'],
      [/@symbols/, 'delimiter'],

      // Numbers
      [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
      [/0[xX][0-9a-fA-F]+/, 'number.hex'],
      [/\d+/, 'number'],

      // Strings
      [/"([^"\\]|\\.)*$/, 'string.invalid'],
      [/"/, 'string', '@string_double'],
      [/'([^'\\]|\\.)*$/, 'string.invalid'],
      [/'/, 'string', '@string_single'],
    ],

    whitespace: [
      [/[ \t\r\n]+/, ''],
      [/\/\*/, 'comment', '@comment'],
      [/\/\/.*$/, 'comment'],
      [/#.*$/, 'comment'],
    ],

    comment: [
      [/[^\/*]+/, 'comment'],
      [/\*\//, 'comment', '@pop'],
      [/[\/*]/, 'comment'],
    ],

    string_double: [
      [/[^\\"]+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/"/, 'string', '@pop'],
    ],

    string_single: [
      [/[^\\']+/, 'string'],
      [/@escapes/, 'string.escape'],
      [/\\./, 'string.escape.invalid'],
      [/'/, 'string', '@pop'],
    ],
  },
}

export const arclangTheme = {
  base: 'vs-dark' as const,
  inherit: true,
  rules: [
    { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
    { token: 'attribute', foreground: '9CDCFE' },
    { token: 'type', foreground: '4EC9B0' },
    { token: 'identifier', foreground: 'D4D4D4' },
    { token: 'string', foreground: 'CE9178' },
    { token: 'number', foreground: 'B5CEA8' },
    { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
    { token: 'delimiter', foreground: 'D4D4D4' },
  ],
  colors: {
    'editor.background': '#1e1e1e',
    'editor.foreground': '#d4d4d4',
    'editorLineNumber.foreground': '#858585',
    'editorCursor.foreground': '#aeafad',
  },
}

export const arclangConfiguration = {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/'],
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  folding: {
    markers: {
      start: new RegExp('^\\s*//\\s*#?region\\b'),
      end: new RegExp('^\\s*//\\s*#?endregion\\b'),
    },
  },
}

// Sample ArcLang code for new files
export const defaultArcLangCode = `// Autonomous Emergency Braking System - Full Capella Architecture

operational_analysis "Vehicle Operation Context" {
  actor "Driver" {
    id: "ACT-001"
    description: "Vehicle operator"
  }
  
  actor "Pedestrian" {
    id: "ACT-002"
    description: "Vulnerable road user"
  }
  
  actor "Vehicle" {
    id: "ACT-003"
    description: "Other vehicles on the road"
  }
}

system_analysis "System Requirements" {
  requirement "SYS-001" {
    id: "SYS-001"
    description: "The system shall detect obstacles within 100m range"
    priority: "Critical"
    safety_level: "ASIL_D"
    verification_method: "Test"
  }
  
  requirement "SYS-002" {
    id: "SYS-002"
    description: "The system shall calculate collision risk in real-time"
    priority: "Critical"
    safety_level: "ASIL_D"
    verification_method: "Analysis"
  }
  
  requirement "SYS-003" {
    id: "SYS-003"
    description: "The system shall trigger emergency braking when collision is imminent"
    priority: "Critical"
    safety_level: "ASIL_D"
    verification_method: "Test"
  }
  
  requirement "SYS-004" {
    id: "SYS-004"
    description: "The system shall alert the driver before automatic braking"
    priority: "High"
    safety_level: "ASIL_B"
    verification_method: "Test"
  }
  
  system_function "Detect Environment" {
    id: "SF-001"
    description: "Perceive surrounding environment"
  }
  
  system_function "Assess Collision Risk" {
    id: "SF-002"
    description: "Analyze potential collision scenarios"
  }
  
  system_function "Execute Emergency Braking" {
    id: "SF-003"
    description: "Apply maximum braking force"
  }
}

logical_architecture "System Architecture" {
  component "Radar Sensor" {
    id: "LC-001"
    type: "Logical"
    description: "Long-range radar for obstacle detection"
    safety_level: "ASIL_D"
    
    function "Scan Environment" {
      id: "LF-001"
      description: "Continuously scan for obstacles"
    }
    
    function "Measure Distance" {
      id: "LF-002"
      description: "Calculate distance to detected objects"
    }
  }
  
  component "Camera System" {
    id: "LC-002"
    type: "Logical"
    description: "Vision-based object recognition"
    safety_level: "ASIL_D"
    
    function "Capture Image" {
      id: "LF-003"
      description: "Acquire visual data"
    }
    
    function "Classify Objects" {
      id: "LF-004"
      description: "Identify pedestrians, vehicles, obstacles"
    }
  }
  
  component "Sensor Fusion Controller" {
    id: "LC-003"
    type: "Logical"
    description: "Combine data from multiple sensors"
    safety_level: "ASIL_D"
    
    function "Fuse Sensor Data" {
      id: "LF-005"
      description: "Merge radar and camera inputs"
    }
    
    function "Validate Detection" {
      id: "LF-006"
      description: "Confirm obstacle presence"
    }
  }
  
  component "Collision Risk Analyzer" {
    id: "LC-004"
    type: "Logical"
    description: "Assess collision probability and timing"
    safety_level: "ASIL_D"
    
    function "Calculate TTC" {
      id: "LF-007"
      description: "Time To Collision calculation"
    }
    
    function "Determine Risk Level" {
      id: "LF-008"
      description: "Classify threat severity"
    }
  }
  
  component "Brake Actuator Controller" {
    id: "LC-005"
    type: "Logical"
    description: "Execute braking commands"
    safety_level: "ASIL_D"
    
    function "Apply Brake Force" {
      id: "LF-009"
      description: "Control brake hydraulics"
    }
    
    function "Monitor Brake Status" {
      id: "LF-010"
      description: "Verify braking effectiveness"
    }
  }
  
  component "Driver Interface" {
    id: "LC-006"
    type: "Logical"
    description: "Human-machine interface"
    safety_level: "ASIL_B"
    
    function "Display Warning" {
      id: "LF-011"
      description: "Alert driver of danger"
    }
    
    function "Log Events" {
      id: "LF-012"
      description: "Record system activations"
    }
  }
}

physical_architecture "Hardware Deployment" {
  node "Sensor ECU" {
    id: "PN-001"
    description: "Electronic Control Unit for sensors"
  }
  
  node "Central Processing Unit" {
    id: "PN-002"
    description: "Main computation unit"
  }
  
  node "Brake Control Unit" {
    id: "PN-003"
    description: "Brake system controller"
  }
}

trace "LC-001" satisfies "SYS-001" {
  rationale: "Radar sensor implements obstacle detection requirement"
}

trace "LC-002" satisfies "SYS-001" {
  rationale: "Camera system provides complementary detection"
}

trace "LC-004" satisfies "SYS-002" {
  rationale: "Risk analyzer implements collision risk assessment"
}

trace "LC-005" satisfies "SYS-003" {
  rationale: "Brake controller implements emergency braking"
}

trace "LC-006" satisfies "SYS-004" {
  rationale: "Driver interface implements warning alerts"
}

trace "LC-003" implements "LC-001" {
  rationale: "Sensor fusion processes radar data"
}

trace "LC-003" implements "LC-002" {
  rationale: "Sensor fusion processes camera data"
}

trace "LC-004" implements "LC-003" {
  rationale: "Risk analysis depends on fused sensor data"
}

trace "LC-005" implements "LC-004" {
  rationale: "Braking decision based on risk assessment"
}
`
