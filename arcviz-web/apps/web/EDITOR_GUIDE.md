# ArcViz Editor - User Guide

## Features

### 🎨 Syntax Highlighting
- **Keywords**: `system_analysis`, `logical_architecture`, `component`, `requirement`, etc.
- **Attributes**: `id`, `description`, `safety_level`, `priority`, etc.
- **Safety Levels**: `ASIL_A/B/C/D`, `DAL_A/B/C/D`
- **Strings & Numbers**: Proper highlighting for values

### ⌨️ Keyboard Shortcuts
- **Ctrl/Cmd + S**: Save file
- **Shift + Alt + F**: Format document
- **Ctrl + Space**: Trigger autocomplete
- **F12**: Go to definition
- **Shift + F12**: Find all references

### 🔧 Editor Features
- **Minimap**: Quick navigation for large files
- **Line Numbers**: With syntax-aware folding
- **Bracket Matching**: Colorized bracket pairs
- **Auto-Indent**: Smart indentation
- **Word Wrap**: Enabled by default
- **Smooth Scrolling**: Enhanced UX

### 📊 Console Panel
- **Real-time Validation**: Errors, warnings, and info messages
- **Compilation Results**: Success/failure with statistics
- **Jump to Error**: Click on any issue to navigate to the line

### 🛠️ Toolbar Actions
1. **Save**: Save current file (Ctrl/Cmd + S)
2. **Compile**: Validate and compile ArcLang code
3. **Visualize**: Open interactive architecture diagram
4. **AI Assist**: Get AI-powered suggestions
5. **Export**: Export to Capella XML, Mermaid, PlantUML, JSON
6. **Import**: Import from Capella XML or existing ArcLang files
7. **Share**: Collaborate with team members

## ArcLang Syntax Quick Reference

### System Analysis
\`\`\`arclang
system_analysis "System Name" {
    requirement "REQ-001" {
        description: "Requirement description"
        priority: "Critical"
        safety_level: "ASIL_B"
        category: "Functional"
        verification_method: "Test"
    }
    
    system_function "Function Name" {
        id: "SF-001"
        description: "Function description"
        safety_level: "ASIL_B"
    }
}
\`\`\`

### Logical Architecture
\`\`\`arclang
logical_architecture "Architecture Name" {
    component "Component Name" {
        id: "LC-001"
        component_type: "Logical"
        description: "Component description"
        safety_level: "ASIL_B"
        
        function "Function Name" {
            id: "LF-001"
            description: "Function description"
        }
    }
    
    interface "Interface Name" {
        id: "LI-001"
        from: "LC-001"
        to: "LC-002"
        interface_type: "Data"
    }
}
\`\`\`

### Traceability
\`\`\`arclang
trace "LC-001" satisfies "REQ-001" {
    rationale: "Component implements the requirement"
}
\`\`\`

## Tips & Best Practices

1. **Use Meaningful IDs**: Follow naming conventions (REQ-, SF-, LC-, etc.)
2. **Add Descriptions**: Help team members understand your architecture
3. **Safety Levels**: Always specify for critical components
4. **Traceability**: Link requirements to components for compliance
5. **Save Often**: Use Ctrl/Cmd + S frequently
6. **Compile Regularly**: Catch errors early

## Common Issues

### Syntax Errors
- Missing closing braces `}`
- Incorrect attribute names
- Missing colons `:` after attributes
- Unquoted strings

### Validation Errors
- Missing required attributes (`id`, `description`)
- Invalid safety level values
- Circular dependencies in traces
- Undefined component references

## Next Steps

1. **Save Your Work**: Click Save or press Ctrl/Cmd + S
2. **Compile**: Validate your architecture
3. **Visualize**: Open the interactive diagram viewer
4. **Export**: Generate documentation or integrate with other tools
5. **Collaborate**: Share with your team

---

For more help, visit [docs.arcviz.io](https://docs.arcviz.io)
