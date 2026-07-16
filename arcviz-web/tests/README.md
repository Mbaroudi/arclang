# ArcViz Comprehensive Test Suite

Complete test coverage for ArcViz platform including API, frontend, and end-to-end workflows with dynamic ArcLang code parsing.

## Overview

This test suite validates the complete ArcViz MBSE platform with focus on:
- **Dynamic Code Parsing**: Tests new ArcLangParser service that transforms user code into diagrams
- **API Endpoints**: Tests diagram generation with real ArcLang code
- **Frontend UI**: Tests Monaco editor and visualizer interfaces
- **E2E Workflows**: Tests complete user journeys from editing to visualization
- **MCP Integration**: Tests AI-powered features and MCP server integration

## Prerequisites

### Required Software
- Node.js >= 18.0.0
- Chrome/Chromium browser (for Selenium tests)
- Running API server (port 3001)
- Running Web server (port 3002)

### Install Dependencies
```bash
cd tests
npm install
```

Dependencies installed:
- `selenium-webdriver`: Frontend E2E testing
- `chromedriver`: Chrome automation driver

## Test Suites

### 1. API Diagram Generation Tests (`test-api-diagrams.js`)
Tests the core API functionality with dynamic code parsing.

**What it tests:**
- `/api/diagrams/generate` endpoint with ArcLang code parameter
- `/api/diagrams/generate-all` endpoint for bulk generation
- `/api/diagrams/types` endpoint for available diagram types
- Dynamic parsing of ArcLang code via ArcLangParser
- Sample data fallback when no code provided
- Error handling for invalid requests

**Run:**
```bash
node test-api-diagrams.js
```

**Example test code:**
```javascript
// Tests operational, functional, component, sequence, state-machine,
// physical, class, tree, capability, and functional-chain diagrams
const code = `
operational_analysis "Context" {
  actor "User" { id: "ACT-001" }
}
system_analysis "Requirements" {
  requirement "REQ-001" { priority: "High" }
}
logical_architecture "Architecture" {
  component "Frontend" { id: "LC-001" }
}
`;
```

### 2. Editor Interface Tests (`test-editor-selenium.js`)
Tests the Monaco editor and code editing functionality.

**What it tests:**
- Editor page loading and rendering
- Monaco editor initialization
- Code modification and typing
- Auto-save to localStorage
- Toolbar buttons and navigation
- Keyboard shortcuts (Ctrl+A, etc.)
- Code persistence across page loads

**Run:**
```bash
node test-editor-selenium.js
```

**Key scenarios:**
- Load editor → Verify Monaco → Edit code → Verify auto-save
- Test keyboard shortcuts and selection
- Navigate to visualizer with code

### 3. Visualizer Interface Tests (`test-visualizer-selenium.js`)
Tests the diagram visualization and generation interface.

**What it tests:**
- Visualizer page loading
- Diagram grid layout
- Generate All Diagrams button
- SVG diagram rendering
- Diagram visibility and display
- All 10 Capella diagram types
- Responsive layout (desktop, laptop, tablet)
- LocalStorage code retrieval

**Run:**
```bash
node test-visualizer-selenium.js
```

**Key scenarios:**
- Load visualizer → Click Generate → Verify SVG elements
- Check all diagram types present
- Test responsive layouts

### 4. End-to-End Workflow Tests (`test-e2e-workflow.js`)
Tests complete user workflows from editing to visualization.

**What it tests:**
- Complete user journey: Editor → Edit → Save → Visualize → Generate
- Code persistence through workflow
- API communication during generation
- Error handling with invalid code
- Round-trip data integrity
- LocalStorage integration throughout

**Run:**
```bash
node test-e2e-workflow.js
```

**Workflow tested:**
1. Navigate to Editor
2. Clear existing code
3. Insert test ArcLang code
4. Wait for auto-save
5. Navigate to Visualizer
6. Verify code loaded
7. Generate all diagrams
8. Verify SVG generation
9. Return to Editor
10. Verify code intact

### 5. MCP Integration Tests (`test-mcp-integration.js`)
Tests MCP server integration and AI-powered features.

**What it tests:**
- MCP server connectivity
- AI code generation endpoints
- Code review capabilities
- Syntax validation
- Environment configuration
- AI-powered diagram generation

**Run:**
```bash
node test-mcp-integration.js
```

**Endpoints tested:**
- `/api/ai/generate/component-code` - AI code generation
- `/api/ai/review` - Code review
- `/api/ai/validate-syntax` - Syntax validation
- `/api/ai/generate/:diagramType` - AI diagram generation
- `/api/ai/generate-all` - Bulk AI generation

**Note:** MCP tests work without MCP server but report configuration status.

## Master Test Runner

Run all test suites sequentially with `run-all-tests.js`:

```bash
# Run all tests
node run-all-tests.js

# Run only API and Integration tests (skip Selenium)
node run-all-tests.js --quick

# Force run even if servers unavailable
node run-all-tests.js --force

# Show help
node run-all-tests.js --help
```

**Features:**
- Pre-flight server availability check
- Sequential test execution with progress
- Comprehensive summary report
- Exit code 0 on success, 1 on failure
- Category-based filtering (--quick mode)

## Environment Variables

Configure test environment:

```bash
# API server URL (default: http://localhost:3001)
export API_URL=http://localhost:3001

# Web server URL (default: http://localhost:3002)
export WEB_URL=http://localhost:3002

# MCP server URL (optional)
export MCP_SERVER_URL=http://localhost:3003

# Run tests
node run-all-tests.js
```

## Quick Start

### 1. Start Servers
```bash
# Terminal 1: Start API server
cd apps/api
npm run dev

# Terminal 2: Start Web server
cd apps/web
npm run dev
```

### 2. Run Tests
```bash
# Terminal 3: Run all tests
cd tests
node run-all-tests.js

# Or run individual tests
node test-api-diagrams.js
node test-editor-selenium.js
node test-visualizer-selenium.js
node test-e2e-workflow.js
node test-mcp-integration.js
```

## Test Coverage

### API Layer
- ✓ Dynamic ArcLang code parsing
- ✓ ArcLangParser service integration
- ✓ All 10 Capella diagram types
- ✓ Sample data fallback
- ✓ Error handling
- ✓ Request validation

### Frontend Layer
- ✓ Monaco editor initialization
- ✓ Code editing and modification
- ✓ LocalStorage integration
- ✓ Auto-save functionality
- ✓ Diagram visualization
- ✓ SVG rendering
- ✓ Responsive layouts
- ✓ Navigation between pages

### Integration Layer
- ✓ Editor → Visualizer workflow
- ✓ Code persistence
- ✓ API communication
- ✓ Real-time diagram generation
- ✓ Error recovery
- ✓ MCP server integration

### Diagram Types Tested
All 10 Capella/Arcadia diagram types:
1. Operational Activity Diagram
2. Functional Dataflow Diagram
3. Component Architecture Diagram
4. Sequence Diagram
5. State Machine Diagram
6. Physical Architecture Diagram
7. Class Diagram
8. Tree Diagram
9. Capability Diagram
10. Functional Chain Diagram

## Test Data

Tests use comprehensive ArcLang code examples including:
- Operational analysis (actors, use cases)
- System analysis (requirements, system functions)
- Logical architecture (components, functions)
- Physical architecture (nodes, deployments)
- Traceability links

**Example test model:**
```arclang
operational_analysis "Smart Home Context" {
  actor "Homeowner" {
    id: "ACT-001"
    description: "Smart home user"
  }
}

system_analysis "Requirements" {
  requirement "REQ-001" {
    id: "REQ-001"
    description: "System shall control lighting"
    priority: "High"
  }
}

logical_architecture "Architecture" {
  component "Sensor Hub" {
    id: "LC-001"
    type: "Logical"
    
    function "Read Sensors" {
      id: "LF-001"
    }
  }
}

trace "LC-001" satisfies "REQ-001" {
  rationale: "Sensor hub enables lighting control"
}
```

## Troubleshooting

### Chrome Driver Issues
```bash
# Reinstall chromedriver
cd tests
npm uninstall chromedriver
npm install chromedriver
```

### Server Not Available
```bash
# Check if servers are running
curl http://localhost:3001/api/diagrams/types
curl http://localhost:3002

# Start servers if needed
cd apps/api && npm run dev
cd apps/web && npm run dev
```

### Test Failures
```bash
# Check API logs
tail -f /tmp/api-server.log

# Check Web logs  
tail -f /tmp/web-server.log

# Run tests individually to isolate issues
node test-api-diagrams.js
```

### MCP Integration
```bash
# Set MCP server URL if available
export MCP_SERVER_URL=http://localhost:3003

# Set AI API keys for full functionality
export ANTHROPIC_API_KEY=sk-...
# or
export OPENAI_API_KEY=sk-...

# Run MCP tests
node test-mcp-integration.js
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: ArcViz Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install dependencies
        run: |
          npm install
          cd tests && npm install
      
      - name: Start servers
        run: |
          cd apps/api && npm run dev &
          cd apps/web && npm run dev &
          sleep 10
      
      - name: Run tests
        run: cd tests && node run-all-tests.js
```

## Test Results Format

Each test suite provides:
- ✓ Passed test count
- ✗ Failed test count
- Detailed logs for each test
- Overall success rate
- Specific error messages

**Example output:**
```
╔════════════════════════════════════════════════╗
║           OVERALL TEST SUMMARY                 ║
╚════════════════════════════════════════════════╝

Total Tests: 25
✓ Passed: 24
✗ Failed: 1
Success Rate: 96.0%
```

## Contributing

When adding new tests:
1. Follow existing test patterns
2. Use descriptive test names
3. Include error handling
4. Document test scenarios
5. Update this README

## Support

For issues or questions:
- Check `/tmp/api-server.log` for API errors
- Check `/tmp/web-server.log` for frontend errors
- Review individual test output for details
- Ensure servers are running before tests

## License

Part of the ArcViz project.
