"""
ArcLang Syntax Guide Resource for MCP Server.
This resource is exposed to AI clients to enforce correct syntax.
"""

ARCLANG_SYNTAX_RULES = """
# ArcLang Syntax Rules - MANDATORY FOR ALL AI CLIENTS

## Model Declaration (REQUIRED FORMAT)
```arc
model ModelName {
    // ModelName must be IDENTIFIER (no quotes, no spaces)
}
```

## Requirements (REQUIRED FORMAT)
```arc
requirements stakeholder {
    req REQ-ID "Title" {
        description: "Text"
        priority: Critical
        safety_level: ASIL_B
    }
}
```

## Architecture (REQUIRED FORMAT)
```arc
architecture logical {
    component ComponentName "Display Name" {
        description: "Text"
        provides interface IName {
            signals: ["Signal: Type"]
        }
    }
    connect ComponentA.IInterface -> ComponentB
}
```

## PROHIBITED SYNTAX
❌ model "Name with Spaces" { }      → Use: model ModelName { }
❌ system "Name" { }                  → Use: model Name { }
❌ requirement "REQ-001" { }          → Use: req REQ-001 "Title" { }
❌ logical_architecture { }           → Use: architecture logical { }
❌ component "Name" { }               → Use: component Name "Display" { }
❌ port "name" { }                    → Use: provides interface IName { }
❌ function "name" { }                → Use: description attribute
❌ Top-level requirements { }         → Wrap in model { }

## Key Rules
1. Always wrap everything in: model Name { }
2. Use 'architecture logical' not 'logical_architecture'
3. Use 'requirements stakeholder/system/safety' with subtype
4. Use 'req ID "Title"' not 'requirement "ID"'
5. Component names are identifiers, display names are strings
6. Interfaces use 'provides/requires' not 'port'

ALL GENERATED CODE WILL BE VALIDATED. FOLLOW THESE RULES EXACTLY.
"""

def get_syntax_rules() -> str:
    """Return the mandatory syntax rules for AI clients."""
    return ARCLANG_SYNTAX_RULES
