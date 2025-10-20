# ArcLang Issues Report - Part 2: Solutions and Roadmap

## 🟡 MAJOR ISSUES (continued)

### 5. Limited Keyword Support - Missing MBSE Constructs

**Problem**: The lexer only supports basic Arcadia keywords but lacks extended MBSE concepts.

**Missing Keywords** (line 190-212 in `lexer.rs`):
- `model`, `metadata`, `version`, `author`
- `requirements`, `stakeholder`, `system`, `safety`
- `architecture`, `logical`, `physical`
- `provides`, `requires`, `interface`, `signals`
- `connect`, `via`
- `scenarios`, `steps`, `precondition`, `postcondition`
- `properties`, `implements`, `parent`

**Current Keywords**:
Only supports: `operational_analysis`, `system_analysis`, `logical_architecture`, `physical_architecture`, `actor`, `requirement`, `component`, `function`, etc.

**Impact**: HIGH  
- Forces unnatural syntax
- Example file created (adaptive_cruise_control.arc with 500+ lines) is **completely invalid**
- Users must learn ArcLang-specific DSL instead of industry-standard Arcadia terminology

**Example of Broken User Experience**:
```arc
model AdaptiveCruiseControl {  # ❌ 'model' not recognized
    metadata {  # ❌ 'metadata' not recognized
        version: "1.0.0"  # ❌ Decimal point rejected
    }
}
```

Must write:
```arc
operational_analysis "ACC" {  # ✓ Weird but works
    actor "System" {
        id: "ACT-001"  # No metadata support
    }
}
```

---

### 6. String Literal Restrictions

**File**: `src/compiler/lexer.rs:132-162`

**Problem**: Strings support escape sequences but lexer rejects technical characters:

**Works**:
```arc
description: "Basic string"
description: "String with \n newline"
```

**Fails**:
- `"ISO 26262"` → Space causes issues
- `"0.5m accuracy"` → Decimal in string  
- `"30-180 km/h"` → Special characters
- `"-40°C to 85°C"` → Degree symbol
- `"1280x960"` → May work but inconsistent

**Root Cause**: Not the string parsing itself, but downstream parser expectations.

**Impact**: MEDIUM  
- MBSE documentation requires precise technical specifications
- Units, ranges, and standards are core MBSE data
- Forces unnatural workarounds

---

### 7. No Component Port/Interface Syntax

**Problem**: Capella and Arcadia use `Component.Port` or `Component.Interface` notation extensively.

**Current State**: 
```arc
connect SensingSubsystem.IObjectDetection -> ControllerSubsystem  # ❌ FAILS
```

**Must Use**:
```arc
# No direct connection syntax supported
# Must define in component blocks only
```

**Impact**: HIGH  
- Cannot express architectural connections naturally
- Capella models use this everywhere
- Major deviation from Arcadia methodology

---

## 🟢 MODERATE ISSUES

### 8. Export Format Implementation Status

**Status Check**:

| Format | Enum Defined | CLI Parsing | Implementation | Output | Status |
|--------|--------------|-------------|----------------|--------|--------|
| Capella | ✓ | ✓ | ? | ❌ | **Broken** |
| JSON | ✓ | ✓ | ? | ❌ | **Broken** |
| YAML | ✓ | ✓ | ❌ | ❌ | **Missing** |
| XML | ✓ | ✓ | ? | ❌ | **Broken** |
| Markdown | ✓ | ✓ | ❌ | ❌ | **Missing** |
| Mermaid | ✓ | ✓ | ❌ | ❌ | **Missing** |
| PlantUML | ✓ | ✓ | ❌ | ❌ | **Missing** |
| ArcViz* | ✓ | ✓ | ? | ❌ | **Unknown** |
| HTML | ✓ | ✓ | ? | ❌ | **Broken** |
| PDF | ✓ | ✓ | ❌ | ❌ | **Missing** |

Note: `build` command DOES work and creates Capella XML (mislabeled as .json)

---

### 9. Diagram Command vs Export Command Confusion

**Two Commands Exist**:
```rust
Commands::Export { format: ExportFormat }  // Line 100-109
Commands::Diagram { format: DiagramFormat }  // Line 171-186
```

**DiagramFormat Enum** (line 299-305):
```rust
pub enum DiagramFormat {
    Mermaid,
    PlantUML,
    Graphviz,
    SVG,
}
```

**Problem**: 
- Why two separate commands?
- `export --format mermaid` vs `diagram --format mermaid`?
- User confusion about which to use
- Potential code duplication

---

### 10. MCP Server Async Wrapper Issues

**File**: `mcp-server/src/arclang_mcp/compiler/wrapper.py:189-219`

**Problem**: The compiler wrapper assumes certain command structures:
```python
cmd = [self.binary_path, "build", str(model_path)]  # Line 32
cmd = [self.binary_path, "check", str(model_path), "--lint"]  # Line 50
cmd = [self.binary_path, "export", str(model_path), "-o", output_path, "-f", format_type]  # Line 95
```

**Issues**:
1. **No error parsing**: Rust compiler errors aren't parsed into structured format
2. **Timeout hardcoded**: 30 seconds may be too short for large models
3. **No progress feedback**: Long compilations appear frozen
4. **Return code only**: Doesn't parse compiler's structured output

**Impact**: MEDIUM  
- Poor user experience in Claude Desktop
- No actionable error messages
- Can't show compilation progress

---

### 11. Safety Standard Representation Inconsistency

**In Compiler** (`cli/mod.rs:293-297`):
```rust
pub enum SafetyStandard {
    ISO26262,    // No spaces, no hyphens
    DO178C,
    IEC61508,
}
```

**In User Examples**:
```arc
safety_standard: "ISO 26262"  # User expectation - FAILS
safety_standard: "ISO_26262"  # Current workaround - UGLY
```

**In Real World**:
- Standard is officially "ISO 26262" (with space)
- DO-178C (with hyphen)
- IEC 61508 (with space)

**Impact**: LOW but ANNOYING  
- Breaks copy-paste from standards documents
- Confusion in documentation
- Looks unprofessional

---

## 📊 SYNTAX ENHANCEMENT PROPOSALS

### Proposal 1: Enhanced Number Lexing

**Change**: `src/compiler/lexer.rs:164-180`

**Current**:
```rust
fn read_number(&mut self) -> Result<Token, String> {
    let mut number_str = String::new();
    
    if self.current_char() == '-' {
        number_str.push('-');
        self.advance();
    }
    
    while !self.is_at_end() && (self.current_char().is_ascii_digit() || self.current_char() == '.') {
        number_str.push(self.current_char());
        self.advance();
    }
    
    number_str.parse::<f64>()
        .map(Token::Number)
        .map_err(|_| format!("Invalid number: {}", number_str))
}
```

**Proposed**:
```rust
fn read_number(&mut self) -> Result<Token, String> {
    let mut number_str = String::new();
    let mut has_decimal = false;
    
    if self.current_char() == '-' {
        number_str.push('-');
        self.advance();
    }
    
    while !self.is_at_end() {
        let ch = self.current_char();
        
        if ch.is_ascii_digit() {
            number_str.push(ch);
            self.advance();
        } else if ch == '.' && !has_decimal && self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
            // Only consume '.' if followed by digit (true decimal point)
            has_decimal = true;
            number_str.push(ch);
            self.advance();
        } else if ch == '_' {
            // Allow underscores as digit separators (e.g., 1_000_000)
            // Don't include in number string, just skip
            self.advance();
        } else {
            break;
        }
    }
    
    number_str.parse::<f64>()
        .map(Token::Number)
        .map_err(|_| format!("Invalid number: {}", number_str))
}
```

**Benefits**:
- Supports `1_000_000` (digit separators)
- Only treats `.` as decimal when followed by digit
- Allows `version: "1.0.0"` in strings (not parsed as number)

---

### Proposal 2: Flexible String Content

**Current**: Strings work, but downstream parser rejects certain patterns

**Proposal**: Add validation pass that allows:
- Decimal points in quoted strings: `"1.0.0"`, `"ISO 26262"`
- Special characters: `"30-180 km/h"`, `"-40°C to 85°C"`
- Technical notation: `"0.5m accuracy"`, `"±2 km/h"`

**Implementation**: No lexer change needed - fix parser validation

---

### Proposal 3: Extend Keyword Set

**Add to `lexer.rs:190-212`**:

```rust
"model" => Token::Model,
"metadata" => Token::Metadata,
"version" => Token::Version,
"author" => Token::Author,
"description" => Token::Description,
"requirements" => Token::Requirements,
"stakeholder" => Token::Stakeholder,
"architecture" => Token::Architecture,
"logical" => Token::Logical,
"physical" => Token::Physical,
"provides" => Token::Provides,
"requires" => Token::Requires,
"signals" => Token::Signals,
"connect" => Token::Connect,
"via" => Token::Via,
"scenarios" => Token::Scenarios,
"scenario" => Token::Scenario,
"steps" => Token::Steps,
"precondition" => Token::Precondition,
"postcondition" => Token::Postcondition,
"properties" => Token::Properties,
"parent" => Token::Parent,
"safety_level" => Token::SafetyLevel,
"priority" => Token::Priority,
"traces" => Token::Traces,
"verification" => Token::Verification,
"rationale" => Token::Rationale,
```

**Benefit**: Support intuitive MBSE syntax

---

### Proposal 4: Dot Notation for Architectural Elements

**Add Tokens**:
```rust
Dot,  // '.' for Component.Interface syntax
Arrow,  // '->' for connections
```

**Update Lexer** (`lexer.rs:100`):
```rust
'.' => {
    self.advance();
    Ok(Token::Dot)
}
'-' => {
    if self.peek_char() == Some('>') {
        self.advance();
        self.advance();
        Ok(Token::Arrow)
    } else if self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
        self.read_number()
    } else {
        Ok(Token::Minus)  // Or Hyphen for identifiers
    }
}
```

**Enables**:
```arc
connect SensingSubsystem.IObjectDetection -> ControllerSubsystem
connect RadarECU -> ACCMainECU via "CAN Bus (500 kbps)"
```

---

## 🔧 IMPLEMENTATION PRIORITIES

### Phase 1: Critical Fixes (Week 1)

**Priority 1.1**: Fix Export Commands ⚠️ CRITICAL
- File: `src/cli/commands.rs`
- Implement complete `run_export()` function
- At minimum: HTML, Capella XML, JSON must work
- Test with real output file creation

**Priority 1.2**: Fix Build Output Extension
- File: `src/compiler/mod.rs` (build output logic)
- Change: `model.arc` → `model.capella.xml` (not `model.json`)
- Or make format detection automatic

**Priority 1.3**: Enhanced Number Lexing
- File: `src/compiler/lexer.rs:164-180`
- Implement Proposal 1
- Add tests for edge cases
- Maintain backward compatibility

**Estimated Effort**: 3-4 days

---

### Phase 2: Syntax Extensions (Week 2)

**Priority 2.1**: Extend Keywords
- File: `src/compiler/lexer.rs:4-43` (Token enum)
- File: `src/compiler/lexer.rs:190-212` (keyword matching)
- Implement Proposal 3 - add 20+ MBSE keywords
- Update parser to handle new keywords
- Create migration guide for existing models

**Priority 2.2**: Dot Notation Support
- File: `src/compiler/lexer.rs:100-130`
- Implement Proposal 4
- Add architectural connection syntax
- Update all examples

**Priority 2.3**: String Content Validation
- File: `src/compiler/parser.rs` (string validation)
- Allow special characters in strings
- Support technical notation
- Update documentation

**Estimated Effort**: 5-7 days

---

### Phase 3: User Experience (Week 3)

**Priority 3.1**: Better Error Messages
- Parse compiler errors into structured JSON
- Provide actionable suggestions ("Did you mean...?")
- Include line/column numbers
- Add error codes for documentation lookup

**Priority 3.2**: MCP Server Improvements
- File: `mcp-server/src/arclang_mcp/compiler/wrapper.py`
- Add progress feedback via streaming output
- Parse structured errors from compiler
- Make timeout configurable (default 60s)
- Add retry logic for transient failures

**Priority 3.3**: Integration Tests
- Create comprehensive test suite
- Test all export formats
- Test MCP server tools end-to-end
- Add CI/CD pipeline checks

**Estimated Effort**: 5-7 days

---

### Phase 4: Visualization (Week 4)

**Priority 4.1**: Complete Export Formats
- Implement Mermaid diagram export
- Implement PlantUML export
- Implement SVG generation
- Generate PDF reports (via HTML+Chromium)

**Priority 4.2**: Native Diagram Generation
- Implement ArcViz HTML visualization
- Create interactive diagrams with zoom/pan
- Add component filtering
- Export to image formats

**Priority 4.3**: Documentation
- Complete syntax reference guide
- Create migration guide from other tools
- Add video tutorials
- Update all examples

**Estimated Effort**: 7-10 days

---

## 🧪 TESTING REQUIREMENTS

### Lexer Tests (`tests/lexer_tests.rs`)

```rust
#[test]
fn test_version_string() {
    let input = r#"version: "1.0.0""#;
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 4);  // version : "1.0.0" EOF
}

#[test]
fn test_decimal_in_string() {
    let input = r#"description: "0.5m accuracy""#;
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    // Should not fail
}

#[test]
fn test_component_dot_notation() {
    let input = "Component.Interface";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0], Token::Identifier("Component".into()));
    assert_eq!(tokens[1], Token::Dot);
    assert_eq!(tokens[2], Token::Identifier("Interface".into()));
}

#[test]
fn test_number_with_underscores() {
    let input = "speed: 1_000_000";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[2], Token::Number(1000000.0));
}

#[test]
fn test_arrow_operator() {
    let input = "A -> B";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1], Token::Arrow);
}

#[test]
fn test_safety_standard_with_space() {
    let input = r#"standard: "ISO 26262""#;
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();
    assert!(result.is_ok());
}
```

### Export Tests (`tests/export_tests.rs`)

```rust
use std::path::PathBuf;
use std::fs;

#[test]
fn test_export_html_creates_file() {
    let test_model = "test_model.arc";
    let output = "test_output.html";
    
    // Create test model
    fs::write(test_model, r#"
        operational_analysis "Test" {
            actor "User" { id: "ACT-001" }
        }
    "#).unwrap();
    
    // Run export
    let result = export_model(test_model, output, ExportFormat::HTML);
    
    assert!(result.is_ok());
    assert!(PathBuf::from(output).exists());
    
    // Cleanup
    fs::remove_file(test_model).ok();
    fs::remove_file(output).ok();
}

#[test]
fn test_export_capella_xml() {
    // Similar to above
    let output = "test.capella.xml";
    // ... test implementation
    
    // Verify XML is valid
    let content = fs::read_to_string(output).unwrap();
    assert!(content.contains("<?xml version="));
    assert!(content.contains("capella:Project"));
}

#[test]
fn test_all_export_formats() {
    let formats = vec![
        ExportFormat::Capella,
        ExportFormat::JSON,
        ExportFormat::HTML,
        ExportFormat::Mermaid,
    ];
    
    for format in formats {
        let output = format!("test.{:?}", format).to_lowercase();
        let result = export_model("test.arc", &output, format);
        assert!(result.is_ok(), "Failed for format: {:?}", format);
        assert!(PathBuf::from(&output).exists());
    }
}
```

### MCP Integration Tests (`mcp-server/tests/test_integration.py`)

```python
import pytest
from pathlib import Path
from arclang_mcp.compiler import ArcLangCompiler

@pytest.mark.asyncio
async def test_compile_valid_model():
    compiler = ArcLangCompiler({})
    result = await compiler.compile(Path("tests/fixtures/simple.arc"))
    assert result["success"] == True
    assert "output" in result

@pytest.mark.asyncio
async def test_compile_returns_structured_error():
    compiler = ArcLangCompiler({})
    result = await compiler.compile(Path("tests/fixtures/invalid.arc"))
    
    assert result["success"] == False
    assert "errors" in result
    assert isinstance(result["errors"], list)
    
    if result["errors"]:
        error = result["errors"][0]
        assert "line" in error
        assert "column" in error
        assert "message" in error

@pytest.mark.asyncio
async def test_export_diagram():
    compiler = ArcLangCompiler({})
    result = await compiler.export_diagram(
        Path("tests/fixtures/simple.arc"),
        format_type="html"
    )
    
    assert result["success"] == True
    assert Path(result["output_path"]).exists()

@pytest.mark.asyncio
async def test_timeout_handling():
    compiler = ArcLangCompiler({"timeout": 1})  # 1 second
    # Create a model that takes longer
    result = await compiler.compile(Path("tests/fixtures/large.arc"))
    assert "timeout" in result["stderr"].lower()
```

---

## 📈 SUCCESS METRICS

### Lexer Improvements Success Criteria
- ✓ Can parse `version: "1.0.0"` without errors
- ✓ Supports `Component.Interface` notation
- ✓ Accepts technical strings: `"ISO 26262"`, `"0.5m"`, `"-40°C to 85°C"`
- ✓ Number separators work: `1_000_000`
- ✓ All 20+ new MBSE keywords recognized
- ✓ Arrow operator `->` for connections

### Export Functionality Success Criteria
- ✓ All 10 export formats produce output files
- ✓ HTML export opens in browser correctly
- ✓ Capella XML imports into Capella tool successfully
- ✓ SVG diagrams render in browsers
- ✓ Mermaid/PlantUML formats validated by respective tools
- ✓ PDF generation works (via headless browser)

### MCP Server Integration Success Criteria
- ✓ Compiler errors parsed into structured JSON format
- ✓ Progress feedback visible for compilations >5 seconds
- ✓ All 12 MCP tools work perfectly in Claude Desktop
- ✓ Zero permission/path errors
- ✓ Timeout configurable and respected
- ✓ Error messages are actionable

### User Experience Success Criteria
- ✓ Users can copy-paste from ISO/DO/IEC standards docs
- ✓ All example models compile without syntax changes
- ✓ Error messages point to exact line and column
- ✓ Documentation matches actual compiler syntax
- ✓ First-time users can create valid models in <30 minutes
- ✓ Migration from Capella takes <1 day for typical projects

---

## 🔗 FILES TO MODIFY (Complete List)

### Rust Compiler Core

1. **`src/compiler/lexer.rs`** (Priority: HIGH)
   - Lines 4-43: Add Token variants (Dot, Arrow, Model, Metadata, etc.)
   - Lines 100-130: Add Dot and Arrow token matching
   - Lines 164-180: Rewrite `read_number()` per Proposal 1
   - Lines 190-212: Extend keyword matching for 20+ new keywords

2. **`src/compiler/parser.rs`** (Priority: HIGH)
   - Update to handle new tokens
   - Add grammar rules for dot notation
   - Relax string content validation

3. **`src/cli/commands.rs`** (Priority: CRITICAL)
   - Lines 1-200: Add complete `ExportCommand` implementation
   - Add `DiagramCommand` implementation
   - Add structured error output

4. **`src/cli/mod.rs`** (Priority: HIGH)
   - Lines 320-360: Implement `run_export()` fully
   - Lines 264-279: Clarify ExportFormat vs DiagramFormat

5. **`src/compiler/export/mod.rs`** (Priority: CRITICAL - CREATE)
   - New module for export functionality
   - Implement Capella XML export
   - Implement HTML export
   - Implement Mermaid/PlantUML export

### Python MCP Server

6. **`mcp-server/src/arclang_mcp/compiler/wrapper.py`** (Priority: MEDIUM)
   - Lines 189-219: Add error parsing from JSON
   - Add progress feedback parsing
   - Make timeout configurable (default 60s)
   - Add structured error response format

7. **`mcp-server/src/arclang_mcp/server.py`** (Priority: LOW)
   - Already fixed in commit 9a3994a
   - May need tool description updates

8. **`mcp-server/src/arclang_mcp/tools/core.py`** (Priority: MEDIUM)
   - Update error handling
   - Add progress indicators
   - Format structured outputs better

### Tests (Priority: HIGH - CREATE)

9. **`tests/lexer_tests.rs`** (create if missing)
   - All lexer edge cases (30+ tests)
   - Regression tests for Issue #1

10. **`tests/export_tests.rs`** (create)
    - Export format validation
    - Output file checks
    - Capella XML schema validation

11. **`mcp-server/tests/test_integration.py`** (create)
    - End-to-end MCP tool tests
    - Error handling tests
    - Timeout tests

12. **`mcp-server/tests/fixtures/`** (create directory)
    - `simple.arc` - minimal valid model
    - `invalid.arc` - model with syntax errors
    - `large.arc` - model for timeout testing
    - `complete.arc` - full-featured model

### Documentation

13. **`docs/SYNTAX_REFERENCE.md`** (create)
    - Complete keyword reference
    - Syntax examples
    - Migration guide

14. **`docs/EXPORT_FORMATS.md`** (create)
    - Supported formats
    - Usage examples
    - Format-specific options

15. **`mcp-server/QUICKSTART.md`** (update)
    - Reflect new syntax support
    - Update error handling section

16. **`CHANGELOG.md`** (update)
    - Document breaking changes
    - List new features
    - Migration notes

---

## 💡 RECOMMENDATIONS

### Immediate Actions (This Week)

1. **Fix Export Commands** ⚠️ BLOCKING
   - This is preventing real-world usage
   - Start with HTML and Capella XML
   - Estimated: 2 days

2. **Update All Examples**
   - Ensure every .arc file in repo compiles
   - Fix adaptive_cruise_control.arc
   - Estimated: 1 day

3. **Add Integration Tests**
   - Catch these issues before they ship
   - Set up CI/CD pipeline
   - Estimated: 1 day

### Short Term (This Month)

4. **Implement Lexer Enhancements**
   - Proposals 1, 3, 4
   - Most impactful for usability
   - Estimated: 5 days

5. **Complete Visualization Suite**
   - At least: HTML, Mermaid, SVG
   - Interactive diagrams
   - Estimated: 7 days

6. **Document Syntax Completely**
   - Comprehensive reference guide
   - Migration guide from Capella
   - Video tutorials
   - Estimated: 3 days

### Long Term (Next Quarter)

7. **IDE Integration**
   - VS Code extension
   - Real-time validation
   - Syntax highlighting
   - Autocomplete

8. **Capella Roundtrip**
   - Import Capella models
   - Edit in ArcLang
   - Export back to Capella
   - Verify no data loss

9. **Tool Qualification**
   - For safety-critical usage
   - DO-330 compliance
   - ISO 26262 Part 8
   - Certification evidence

---

## 🎯 CONCLUSION

### Current State Assessment

**Strengths:**
- Solid Rust architecture
- Good parser foundation
- Active development
- Clear vision for MBSE

**Critical Gaps:**
1. Export functionality broken (BLOCKS adoption)
2. Syntax too restrictive (BLOCKS usability)
3. Missing MBSE keywords (BLOCKS workflow)
4. Poor error messages (BLOCKS productivity)

### Quick Win Opportunities

**High Impact, Low Effort:**
1. Fix export implementation (2 days, massive value)
2. Extend lexer for decimal strings (1 day, huge usability gain)
3. Update MCP error handling (1 day, better UX)

**Medium Impact, Medium Effort:**
4. Add full keyword set (3 days, natural syntax)
5. Dot notation support (2 days, architectural modeling)
6. Structured error output (2 days, developer productivity)

### Strategic Recommendations

**Week 1 Sprint**: Focus exclusively on export functionality
- Goal: `arclang export` works for HTML, Capella, JSON
- Success: Real diagrams in user hands

**Week 2 Sprint**: Syntax enhancements
- Goal: Natural MBSE syntax supported
- Success: Example models compile as-is

**Week 3 Sprint**: Polish and testing
- Goal: Production-ready quality
- Success: Zero critical bugs, comprehensive tests

**Week 4 Sprint**: Visualization excellence
- Goal: Best-in-class diagram generation
- Success: Users prefer ArcLang diagrams over Capella

---

## 📋 PRIORITY ACTION ITEMS

### P0 - CRITICAL (Start Today)
- [ ] Implement `export` command for HTML format
- [ ] Implement `export` command for Capella XML format
- [ ] Fix build output file extension (.xml not .json)

### P1 - HIGH (Start This Week)
- [ ] Enhanced number lexing (Proposal 1)
- [ ] Extend keyword set (Proposal 3)
- [ ] Add dot notation (Proposal 4)
- [ ] Create comprehensive lexer tests
- [ ] Update all example models

### P2 - MEDIUM (Start Next Week)
- [ ] Structured error output from compiler
- [ ] MCP server error parsing
- [ ] Progress feedback for long operations
- [ ] Export tests for all formats
- [ ] Integration test suite

### P3 - NORMAL (Start Week 3)
- [ ] Complete documentation
- [ ] Mermaid/PlantUML export
- [ ] SVG diagram generation
- [ ] Interactive HTML visualizations
- [ ] Migration guide

### P4 - LOW (Future)
- [ ] PDF export via headless browser
- [ ] VS Code extension
- [ ] Tool qualification
- [ ] Capella import functionality

---

**Report Completion**: Part 2 of 2  
**Total Issues Identified**: 11 critical/major, 5+ moderate  
**Estimated Fix Time**: 4 weeks (1 developer)  
**ROI**: High - fixes unlock real-world MBSE adoption  
**Next Step**: Begin P0 work immediately

**Contact**: malek@arclang.dev, bilel@arclang.dev  
**Repository**: https://github.com/Mbaroudi/arclang
