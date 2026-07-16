# ArcLang Diagram Generation - Testing Guide

## 🎯 System Status

All systems are **READY** for testing!

### Running Services

✅ **API Server**: http://localhost:4001  
✅ **Web Application**: http://localhost:3002  
✅ **MCP Server**: Ready (Python 3.11)

---

## 🧪 Testing Checklist

### 1. Web Application Testing

#### Access the Editor
1. Open browser: **http://localhost:3002/editor**
2. You should see the ArcViz Editor interface

#### Test Diagram Generation
1. **Save a Model** (required first):
   - Type some ArcLang code in the editor
   - Click "Save" button (or Ctrl+S)
   - This sets the `modelPath` for diagram generation

2. **Click the "Diagrams" Tab** (right panel):
   - You'll see 3 tabs: AI | Diagrams | Docs
   - Click "Diagrams"

3. **Generate a Single Diagram**:
   - Select diagram type from dropdown (e.g., "Operational Activity")
   - Click "Generate" button
   - Wait for SVG to appear
   - You should see the diagram rendered below

4. **Test Export Features**:
   - **SVG Export**: Click "SVG" button → downloads `.svg` file
   - **PNG Export**: Click "PNG" button → downloads high-res `.png` file

5. **Test Caching**:
   - Generate a diagram (e.g., "Functional")
   - Switch to another type (e.g., "Component")
   - Generate it
   - Switch back to "Functional"
   - Click "Generate" again
   - Should see "⚡ Loaded from cache" instantly
   - Check top-right for "Clear Cache (2)" button

6. **Test Bulk Generation**:
   - Click "Generate All 10" button
   - Wait ~30 seconds
   - Should see toast: "Generated X/10 diagrams successfully"

---

### 2. MCP Server Testing

#### Start MCP Server
```bash
cd /Users/malek/Arclang/mcp-server
/opt/homebrew/bin/python3.11 -m arclang_mcp.server
```

#### Test with Claude Desktop

1. **Configure Claude Desktop**  
   Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:
   ```json
   {
     "mcpServers": {
       "arclang": {
         "command": "/opt/homebrew/bin/python3.11",
         "args": ["-m", "arclang_mcp.server"],
         "env": {
           "ARCLANG_WORKSPACE": "/Users/malek/Arclang"
         }
       }
     }
   }
   ```

2. **Restart Claude Desktop**

3. **Test Diagram Generation**  
   Try these prompts:
   
   ```
   Generate an operational diagram for my ACC model at examples/acc.arc
   ```
   
   ```
   Generate all 10 Capella diagram types for my model
   ```
   
   ```
   Create a functional diagram showing the data flow
   ```

4. **Verify Output**:
   - Claude should respond with diagram metadata
   - Check for SVG file creation
   - Verify element counts are reported
   - Confirm features are listed

---

### 3. API Endpoint Testing

#### Test with curl

**1. Generate Single Diagram**
```bash
curl -X POST http://localhost:4001/api/diagrams/generate \
  -H "Content-Type: application/json" \
  -d '{
    "modelPath": "/Users/malek/Arclang/examples/acc.arc",
    "diagramType": "operational"
  }'
```

**2. Generate All Diagrams**
```bash
curl -X POST http://localhost:4001/api/diagrams/generate-all \
  -H "Content-Type: application/json" \
  -d '{
    "modelPath": "/Users/malek/Arclang/examples/acc.arc",
    "outputDir": "./test-diagrams"
  }'
```

**3. Get Diagram Types**
```bash
curl http://localhost:4001/api/diagrams/types
```

---

### 4. Integration Testing

#### End-to-End Flow

1. **Create Model in Editor**:
   ```arc
   system "TestSystem" {
     operational_analysis {
       entity Actor1 "Driver" {}
       activity A1 "Drive" { performer: Actor1 }
     }
   }
   ```

2. **Save Model** (Ctrl+S)

3. **Generate Diagram** via UI

4. **Export as PNG**

5. **Verify Output**:
   - SVG displays correctly
   - PNG downloads properly
   - Cache works on re-generation

#### AI → Editor Flow

1. **Use Claude to generate model**:
   ```
   Generate an ArcLang model for adaptive cruise control
   ```

2. **Copy generated code to Editor**

3. **Generate diagrams** from the AI-created model

4. **Verify** all 10 types work

---

## 📊 Expected Results

### Diagram Quality
- ✅ All 10 diagram types render
- ✅ No text overlaps (< 2%)
- ✅ Labels have backgrounds
- ✅ Professional Capella-style output
- ✅ SVG size: 5-30KB per diagram

### Performance
- ✅ Generation: < 5 seconds per diagram
- ✅ Bulk generation: ~30 seconds for all 10
- ✅ Cached load: < 100ms (instant)
- ✅ Export: < 2 seconds

### Features
- ✅ SVG export works
- ✅ PNG export (2x resolution, white background)
- ✅ Caching persists during session
- ✅ Clear cache button appears
- ✅ Cache indicator shows (⚡)

---

## 🐛 Known Issues

### API Server
- ⚠️ TypeScript compilation has auth-related type warnings
- ✅ Server runs fine despite warnings (runtime not affected)
- 🔧 Fix: Pre-existing issue, not related to diagram changes

### MCP Server
- ⚠️ Requires Python 3.10+ (3.11 recommended)
- ✅ Installed successfully with Homebrew Python 3.11

### Ports
- ✅ API: Changed from 4000 → 4001 (to avoid conflicts)
- ✅ Web: Running on 3002 (default)

---

## 🚀 Quick Start Commands

```bash
# Kill any existing processes
lsof -ti:4001 -ti:3002 | xargs kill -9

# Start API server
cd /Users/malek/Arclang/arcviz-web/apps/api
PORT=4001 npm run dev

# Start web server (in new terminal)
cd /Users/malek/Arclang/arcviz-web/apps/web
npm run dev

# Start MCP server (in new terminal)
cd /Users/malek/Arclang/mcp-server
/opt/homebrew/bin/python3.11 -m arclang_mcp.server

# Open in browser
open http://localhost:3002/editor
```

---

## 📝 Test Report Template

After testing, record results:

### ✅ Successful Tests
- [ ] Web UI loads
- [ ] Diagram generation works
- [ ] SVG export works
- [ ] PNG export works
- [ ] Caching works
- [ ] Bulk generation works
- [ ] MCP server responds
- [ ] Claude Desktop integration works

### ❌ Failed Tests
- Issue description
- Steps to reproduce
- Error messages
- Screenshots

### 📈 Performance Metrics
- Diagram generation time: ___
- Bulk generation time: ___
- Cache hit time: ___
- Export time: ___

---

## 🎉 Success Criteria

**Testing is complete when:**

1. ✅ All 10 diagram types generate successfully
2. ✅ Export to SVG and PNG works
3. ✅ Caching shows "⚡ Loaded from cache"
4. ✅ MCP server responds to Claude prompts
5. ✅ No critical errors in console
6. ✅ Performance meets targets (< 5s per diagram)

---

**Current Status**: 🟢 **Ready for Testing**

**Services Running**:
- ✅ API Server: http://localhost:4001
- ✅ Web App: http://localhost:3002
- ⏳ MCP Server: Awaiting manual start

**Next Steps**: Start testing with the Web UI first, then MCP integration.
