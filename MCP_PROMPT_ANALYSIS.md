# 🔍 Analyse des Problèmes MCP Server et Solutions

**Date**: 2025-11-01  
**Contexte**: Claude Desktop générait du code ArcLang avec erreurs systématiques

---

## 🔴 PROBLÈMES IDENTIFIÉS

### 1. **Syntaxe Opérationnelle Manquante dans les Prompts IA**

**Symptôme**: Claude Desktop générait:
```arc
architecture operational {
    actor Driver { }  // ❌ FAUX
}
```

**Cause**: Les fichiers de prompt IA ne documentaient PAS la syntaxe `operational_analysis`

**Impact**: **CRITIQUE** - Tous les modèles opérationnels générés par IA étaient invalides

---

### 2. **Parser Bug avec les Tirets dans les IDs**

**Symptôme**:
```bash
$ arclang check model.arc
✗ Parser error: Expected LeftBrace, got Number(-1)
```

**Cause**: Le parser ArcLang ne supporte PAS les tirets dans les IDs de requirements:
- `req STK-001` → ❌ ERREUR
- `req STK_001` → ✅ OK

**Impact**: **BLOQUANT** - Aucun modèle avec tirets ne peut compiler

---

### 3. **Erreurs de Syntaxe Multiples**

Claude Desktop générait:
- ❌ `actor Driver` au lieu de `actor "Driver"`
- ❌ `entity` au lieu de `actor`
- ❌ Pas de `model { }` wrapper
- ❌ Pas de `safety_level` (bordures colorées manquantes)
- ❌ Pas de `exchange_item_kind` (flèches sémantiques manquantes)
- ❌ `Traces:` au lieu de `traces:` (problème de casse)

---

## ✅ CORRECTIONS APPORTÉES

### 1. **Mise à Jour ARCLANG_SYNTAX_RULES.md** ✅

**Fichier**: `/Users/malek/arclang/mcp-server/ARCLANG_SYNTAX_RULES.md`

**Changements**:

#### a) Ajout Section "Architecture - Operational"
```arc
### Architecture - Operational (Operational Analysis)

operational_analysis "Context Title" {
    actor "Actor Name" {
        id: "OA-ACT-001"
        description: "Actor description"
        category: "Human"  // or "System", "External"
        safety_level: ASIL_C  // Optional - for safety-critical actors
    }
    
    operational_activity "Activity Name" {
        id: "OA-01"
        description: "Activity description"
        performed_by: "OA-ACT-001"
        safety_level: ASIL_D  // Optional
    }
    
    operational_interaction "Interaction Name" {
        id: "OI-01"
        from: "OA-ACT-001"
        to: "OA-ACT-002"
        exchange_item_kind: EVENT  // EVENT, FLOW, OPERATION, DATA
    }
}
```

#### b) Ajout Section "❌ Operational Architecture Errors"
Documente toutes les erreurs courantes:
- `architecture operational` vs `operational_analysis`
- `actor` vs `entity`
- Syntaxe avec/sans guillemets
- Attributs obligatoires

#### c) Exemple Complet MBSE avec Operational
Exemple de 60+ lignes montrant operational_analysis avec:
- 3 actors (Driver, Vehicle System, Environment)
- 3 operational_activity avec safety_level
- 2 operational_interaction avec exchange_item_kind

**Résultat**: 805 lignes de documentation complète

---

### 2. **Mise à Jour conversational-ai.ts** ✅

**Fichier**: `/Users/malek/arclang/arcviz-web/apps/api/src/services/conversational-ai.ts`

**Changements**:

Ligne 139-163 - Remplacement de:
```typescript
✅ Operational Activities (Operational Analysis):
   activity ExecuteBraking {
       safety_level: ASIL_D
       performer: "System"
   }
```

Par un exemple complet de 25 lignes:
```typescript
✅ Operational Activities (Operational Analysis):
   **IMPORTANT**: Use operational_analysis "Title" { } syntax, NOT architecture operational
   
   operational_analysis "Emergency Braking Context" {
       actor "Vehicle System" {
           id: "OA-ACT-001"
           safety_level: ASIL_D  // Red border
       }
       
       operational_activity "Execute Emergency Braking" {
           id: "OA-01"
           performed_by: "OA-ACT-001"
           safety_level: ASIL_D  // Red border
       }
       
       operational_interaction "Collision Alert" {
           id: "OI-01"
           from: "OA-ACT-001"
           to: "OA-ACT-002"
           exchange_item_kind: EVENT  // Red dashed alert
       }
   }
```

---

### 3. **Création Modèle Référence Complet** ✅

**Fichier**: `/Users/malek/arclang/emergency_braking_sensor_fusion_complete.arc`

**Contenu**: 550+ lignes avec:
- ✅ `model EmergencyBrakingSensorFusion { }`
- ✅ 4 stakeholder requirements (STK_001-004)
- ✅ 8 system requirements (SYS_001-008)
- ✅ 4 safety requirements (SAF_001-004)
- ✅ **operational_analysis** complet:
  - 5 actors (Driver, Vehicle, Environment, Others, Pedestrians)
  - 11 operational_activity (avec safety_level ASIL_D/C/B)
  - 4 operational_interaction (avec exchange_item_kind)
- ✅ architecture logical avec 8 components (safety_level ASIL_D/C)
- ✅ architecture physical avec 7 ECUs
- ✅ Traceability matrix complète (40+ traces)

**Fonctionnalités MBSE**:
- ✅ Safety colors (ASIL_D Red, ASIL_C Orange)
- ✅ Exchange item types (EVENT, FLOW)
- ✅ UML interfaces (provides/requires)
- ✅ Traceability links (requirements → components → ECUs)

---

## 🚧 PROBLÈMES NON RÉSOLUS

### 1. **Parser Bug avec les Tirets** ⚠️

**Statut**: Bug dans le compilateur Rust ArcLang

**Workaround**: Utiliser underscores:
- `req STK_001` au lieu de `req STK-001`
- `traces: [STK_001]` au lieu de `traces: [STK-001]`

**Action requise**: Fix dans `/Users/malek/arclang/src/compiler/parser.rs`

### 2. **Casse des Attributs** ⚠️

Le parser semble sensible à la casse:
- `traces:` ✅ OK
- `Traces:` ❌ ERREUR

**Action**: Documenter dans les prompts IA

---

## 📋 CHECKLIST VALIDATION PROMPT IA

Pour vérifier que les prompts IA sont complets:

- [x] **Syntaxe opérationnelle** documentée (operational_analysis)
- [x] **Actors** avec guillemets et attributs
- [x] **operational_activity** avec performed_by
- [x] **operational_interaction** avec exchange_item_kind
- [x] **Safety levels** sur actors/activities
- [x] **Exemples complets** avec tous les éléments
- [x] **Section erreurs courantes**
- [ ] **Parser limitations** (tirets non supportés) ← À AJOUTER
- [ ] **Casse des attributs** (traces: minuscule) ← À AJOUTER

---

## 🎯 ACTIONS RECOMMANDÉES

### Priorité 1: Documenter les Limitations du Parser

Ajouter dans `ARCLANG_SYNTAX_RULES.md`:

```markdown
## ⚠️ PARSER LIMITATIONS

### 1. Requirement IDs Cannot Contain Dashes

❌ INCORRECT:
req STK-001 "Title" { }

✅ CORRECT:
req STK_001 "Title" { }

### 2. Attribute Names are Case-Sensitive

❌ INCORRECT:
req SYS_001 "Title" {
    Traces: [STK_001]  // Capital T
}

✅ CORRECT:
req SYS_001 "Title" {
    traces: [STK_001]  // Lowercase t
}
```

### Priorité 2: Tester avec Claude Desktop

1. **Redémarrer Claude Desktop** (Cmd+Q complet)
2. **Tester génération**: "Create Emergency Braking with operational analysis"
3. **Vérifier sortie** contient:
   - `operational_analysis "Title"`
   - `actor "Name"` avec guillemets
   - `safety_level: ASIL_X`
   - `exchange_item_kind: EVENT/FLOW`
   - Underscores dans IDs (`STK_001` pas `STK-001`)

### Priorité 3: Fix Parser Bug (Optionnel)

**Fichier**: `/Users/malek/arclang/src/compiler/parser.rs`

Chercher le parsing des IDs de requirements et permettre les tirets:
```rust
// Ligne ~XXX
fn parse_requirement_id(&mut self) -> Result<String, String> {
    // Accepter: alphanumeric + underscore + tiret
    // Pattern: [A-Z][A-Z0-9_-]+
}
```

---

## 📊 RÉSUMÉ DES CHANGEMENTS

| Fichier | Lignes Modifiées | Status |
|---------|------------------|--------|
| `mcp-server/ARCLANG_SYNTAX_RULES.md` | +100 lignes | ✅ Complété |
| `arcviz-web/.../conversational-ai.ts` | +25 lignes | ✅ Complété |
| `emergency_braking_sensor_fusion_complete.arc` | +550 lignes (nouveau) | ✅ Créé |
| `MCP_PROMPT_ANALYSIS.md` | +300 lignes (ce fichier) | ✅ Créé |
| **TOTAL** | **~975 lignes** | ✅ |

---

## 🧪 TEST DE VALIDATION

Pour vérifier que tout fonctionne:

```bash
# 1. Vérifier arclang accessible
which arclang  # Doit retourner: /Users/malek/.local/bin/arclang
arclang --version  # Doit retourner: arclang 1.0.0

# 2. Tester modèle minimal sans tirets
cat > test.arc << 'EOF'
model Test {
    requirements stakeholder {
        req STK_001 "Test" {
            description: "Test requirement"
        }
    }
}
EOF

arclang check test.arc  # Doit retourner: ✓ No compilation errors

# 3. Redémarrer Claude Desktop
# Cmd+Q pour quitter complètement
# Rouvrir Claude Desktop

# 4. Tester génération MCP
# Dans Claude Desktop, demander:
# "Generate Emergency Braking operational architecture with sensor fusion"

# 5. Vérifier le code généré contient:
# - operational_analysis "Title"
# - actor "Name" { id: "OA_ACT_001" ... }
# - operational_activity avec safety_level
# - Pas de tirets dans les IDs (STK_001 pas STK-001)
```

---

## 📚 DOCUMENTATION COMPLÉMENTAIRE

### Fichiers de Référence

1. **Syntax Rules**: `/Users/malek/arclang/mcp-server/ARCLANG_SYNTAX_RULES.md`
2. **Install Guide**: `/Users/malek/arclang/ARCLANG_INSTALL_GUIDE.md`
3. **MCP README**: `/Users/malek/arclang/mcp-server/README.md`
4. **Modèle Exemple**: `/Users/malek/arclang/emergency_braking_sensor_fusion_complete.arc`

### Exemples qui Compilent

- `/Users/malek/arclang/examples/emergency_braking_logical.arc` ✅
- `/Users/malek/arclang/examples/vehicle-system.arc` ✅

### Claude Desktop Config

**Fichier**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "arclang": {
      "command": "/opt/homebrew/bin/python3.11",
      "args": ["-m", "arclang_mcp.server"],
      "env": {
        "ARCLANG_WORKSPACE": "/Users/malek/arclang",
        "ARCLANG_BINARY": "/Users/malek/.local/bin/arclang",
        "PATH": "/Users/malek/.local/bin:/Users/malek/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
      }
    }
  }
}
```

**Note**: Redémarrage complet requis après modification

---

## ✅ CONCLUSION

### Problèmes Corrigés

1. ✅ **Prompts IA mis à jour** avec syntaxe operational complète
2. ✅ **Documentation erreurs** opérationnelles ajoutée
3. ✅ **Modèle référence** MBSE complet créé

### Prochaines Étapes

1. **Redémarrer Claude Desktop** pour charger nouveaux prompts
2. **Tester génération** d'architectures opérationnelles
3. **Vérifier diagrammes** contiennent safety colors et exchange types
4. **Fix parser bug** (optionnel) pour supporter tirets dans IDs

### Résultats Attendus

Après redémarrage de Claude Desktop:
- ✅ Code généré utilise `operational_analysis` (pas `architecture operational`)
- ✅ Actors avec guillemets et safety_level
- ✅ Activities avec performed_by et safety_level
- ✅ Interactions avec exchange_item_kind
- ✅ IDs avec underscores (STK_001) pas tirets
- ✅ Diagrammes avec bordures colorées (safety) et flèches sémantiques (exchange types)

---

**Status Global**: 🟢 **CORRECTIONS APPLIQUÉES - PRÊT POUR TEST**

**Date de Complétion**: 2025-11-01 15:30 UTC
