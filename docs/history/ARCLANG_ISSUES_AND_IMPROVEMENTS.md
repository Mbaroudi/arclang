# ArcLang Compiler & MCP Server - Issues and Improvements Report

**Date**: October 20, 2025  
**Version**: ArcLang 1.0.0  
**Context**: Issues discovered during MCP server integration and real-world MBSE usage

---

## Executive Summary

During the development and testing of the ArcLang MCP server, we discovered **critical compatibility issues** between the compiler's lexer, the syntax requirements of MBSE practitioners, and the MCP server expectations. This document catalogs all issues and proposes comprehensive improvements.

---

## 🔴 CRITICAL ISSUES

### 1. Lexer Rejects Decimal Points in Non-Number Contexts

**File**: `src/compiler/lexer.rs:172`

**Problem**:
```rust
while !self.is_at_end() && (self.current_char().is_ascii_digit() || self.current_char() == '.') {
    number_str.push(self.current_char());
    self.advance();
}
```

The lexer treats `.` (decimal point) as ONLY valid in numbers. This causes:

**Failures**:
- `version: "1.0.0"` → **Lexer error: Unexpected character: '.'**
- `"ISO 26262"` (space) → **Lexer error**  
- `safety_standard: "ISO 26262"` → **Lexer error**
- Component connections like `SensingSubsystem.IObjectDetection` → **Lexer error**

**Impact**: HIGH  
- Cannot write natural version numbers (1.0.0, 2.1.3)
- Cannot use industry-standard names with spaces
- Cannot express technical values with decimal points
- Breaks intuitive MBSE syntax

**Current Workarounds** (user friction):
- `version: "1_0_0"` (ugly, non-standard)
- `safety_standard: "ISO_26262"` (doesn't match standards documentation)
- No component.interface syntax (major architectural limitation)

---

### 2. Export Commands Don't Generate Output Files

**Files**: `src/cli/mod.rs:343`, `src/cli/commands.rs` (export implementation missing)

**Problem**:
```bash
arclang export model.arc -o diagram.html -f html
# Says: [INFO] Exporting to HTML format...
# But: NO FILE CREATED
```

**Testing Results**:
- ✗ `--format html` → No output
- ✗ `--format capella` → No output (but claims success)
- ✗ `--format json` → No output
- ✗ `--format svg` → No output
- ✗ `--format pdf` → No output
- ✓ `arclang build` → DOES create XML output (but named .json)

**Impact**: CRITICAL  
The core value proposition of Arc Lang is **interoperability with Capella** and **visualization**. Without working export, the tool is severely limited.

**Root Cause**:
The `ExportFormat` enum exists (lines 264-279 in `cli/mod.rs`) with many formats:
```rust
pub enum ExportFormat {
    Capella, JSON, YAML, XML, Markdown, Mermaid, PlantUML,
    ArcViz, ArcVizSmart, ArcVizChannel, ArcVizPerfect, ArcVizUltimate,
    HTML, PDF,
}
```

But the `run_export()` implementation is **stubbed** or **incomplete**.

---

### 3. MCP Server Format Mismatch

**File**: `mcp-server/src/arclang_mcp/server.py:138`

**Problem**:
MCP server defined formats that DON'T exist in compiler:
```python
"enum": ["arc-viz-ultimate", "mermaid", "plant-uml"]  # WRONG
```

Compiler actually supports:
```rust
[possible values: capella, json, yaml, xml, markdown, html, pdf]
```

**Impact**: MEDIUM  
- MCP server tools fail with "invalid format" errors
- User confusion (docs say one thing, compiler says another)
- Breaks AI-assisted workflows

**Fixed**: Yes (commit 9a3994a), but highlights **lack of integration testing**

---

### 4. Build Output File Extension Mismatch

**Problem**:
```bash
arclang build model.arc
# Creates: model.json  (file extension)
# Contains: <?xml version="1.0"?>  (XML content!)
```

**Impact**: LOW but CONFUSING  
- File is actually Capella XML, not JSON
- Breaks file type detection
- Users open with wrong tools

---

##Human: contunie