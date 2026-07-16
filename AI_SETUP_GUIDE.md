# Guide d'Activation de l'IA pour ArcViz Editor

## 📍 Emplacement de `generateWithAI()`

**Fichier**: `/Users/malek/Arclang/arcviz-web/apps/api/src/services/ai-generator.ts`

Cette fonction gère la génération de code ArcLang via Claude AI (Anthropic).

---

## 🚀 Configuration en 3 Étapes

### Étape 1: Obtenir une Clé API Claude

1. Aller sur: https://console.anthropic.com/
2. Créer un compte (si nécessaire)
3. Naviguer vers **API Keys**
4. Créer une nouvelle clé API
5. Copier la clé (commence par `sk-ant-api03-...`)

### Étape 2: Configurer la Clé API

**Option A**: Créer/modifier `.env` dans `/arcviz-web/apps/api/`

```bash
cd /Users/malek/Arclang/arcviz-web/apps/api
cp .env.example .env
nano .env
```

Ajouter:
```env
ANTHROPIC_API_KEY=sk-ant-api03-votre-vraie-clé-ici
```

**Option B**: Variable d'environnement temporaire

```bash
export ANTHROPIC_API_KEY=sk-ant-api03-votre-vraie-clé-ici
cd /Users/malek/Arclang/arcviz-web/apps/api
npm run dev
```

### Étape 3: Redémarrer le Serveur API

```bash
# Arrêter le serveur actuel
lsof -ti:4000 | xargs kill -9

# Redémarrer avec la nouvelle configuration
cd /Users/malek/Arclang/arcviz-web/apps/api
npm run dev
```

---

## ✅ Vérification

### Test 1: Vérifier que l'IA est activée

```bash
curl -X POST http://localhost:4000/api/ai/generate/requirement \
  -H "Content-Type: application/json" \
  -d '{"description": "Create cruise control system", "enforceSyntax": true}'
```

**Avec IA activée**: Code généré sera unique et contextuel  
**Sans IA (mode template)**: Code généré sera toujours le même template

### Test 2: Dans l'UI Web

1. Ouvrir http://localhost:3002/editor
2. Aller dans l'onglet **AI Assistant**
3. Entrer une description: `"Create an ASIL-D autonomous braking system"`
4. Cliquer **Requirement** ou **Component**
5. Si l'IA est activée, le code sera personnalisé et intelligent

---

## 🔧 Architecture de `generateWithAI()`

### Fichiers Impliqués

```
arcviz-web/apps/api/src/
├── services/
│   └── ai-generator.ts          # ⭐ Logique AI principale
│       ├── generateWithClaude()  # Appel API Claude
│       ├── generateWithAI()      # Fonction principale
│       ├── extractCodeBlock()    # Parse markdown
│       └── generateWithRetry()   # Boucle de régénération
│
└── routes/
    └── ai.ts                      # Routes HTTP
        ├── POST /api/ai/generate/requirement
        ├── POST /api/ai/generate/component
        ├── POST /api/ai/suggest/architecture
        ├── POST /api/ai/review
        └── POST /api/ai/validate-syntax
```

### Flux de Génération avec IA

```
1. Frontend → POST /api/ai/generate/requirement
   Body: { description: "...", enforceSyntax: true }

2. Backend (routes/ai.ts)
   ↓
   if (USE_AI) {  // Vérifie ANTHROPIC_API_KEY
     generateWithRetry(...)  // Avec validation automatique
   } else {
     return template  // Fallback
   }

3. ai-generator.ts → generateWithRetry()
   ↓
   Attempt 1: generateWithClaude(prompt + SYNTAX_RULES)
   ↓
   Validate with compiler
   ↓
   If errors → Attempt 2: generateWithClaude(prompt + previous errors)
   ↓
   Validate again
   ↓
   Max 3 attempts → Return valid code or throw error

4. Claude API
   - Model: claude-3-5-sonnet-20241022
   - Temperature: 0.3 (bas = syntaxe stricte)
   - System prompt: ARCLANG_SYNTAX_RULES (2000+ lignes)
   - Max tokens: 2000

5. Response
   {
     code: "req SYS-001 ...",
     validated: true,
     errors: []
   }
```

### Mécanisme de Retry

```typescript
// Dans ai-generator.ts ligne 130-175
export async function generateWithRetry(
  userPrompt: string,
  systemPrompt: string,
  validator: (code: string) => Promise<{ valid: boolean; errors: string[] }>,
  maxRetries: number = 3
): Promise<{ code: string; attempts: number }> {
  
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    // Si échec précédent, ajouter les erreurs au prompt
    const enhancedPrompt = attempt === 1
      ? userPrompt
      : `${userPrompt}\n\nPrevious attempt failed:\n${lastErrors.join('\n')}\nFix these errors.`
    
    // Générer
    const rawCode = await generateWithAI(enhancedPrompt, systemPrompt)
    const code = extractCodeBlock(rawCode)
    
    // Valider avec compilateur ArcLang
    const validation = await validator(code)
    
    if (validation.valid) {
      return { code, attempts: attempt }  // ✅ Succès
    }
    
    lastErrors = validation.errors  // ❌ Retry avec feedback
  }
  
  throw new Error('Failed after 3 attempts')
}
```

---

## 📊 Comparaison: Avec vs Sans IA

| Fonctionnalité | Sans IA (Templates) | Avec IA (Claude) |
|----------------|---------------------|------------------|
| **Code généré** | Templates statiques | Code contextuel unique |
| **Complexité** | Structures simples | Architectures complètes |
| **Personnalisation** | Nom uniquement | Description complète analysée |
| **Validation** | Syntaxe template OK | Validation + retry jusqu'à succès |
| **Safety levels** | Toujours ASIL_D | Adapté au contexte |
| **Functions** | 1 fonction générique | Multiples fonctions pertinentes |
| **Interfaces** | Signaux génériques | Signaux spécifiques au domaine |
| **Coût** | Gratuit | ~$0.003 par génération |

---

## 🔐 Sécurité des Clés API

### ✅ Bonnes Pratiques

```bash
# .env (gitignored)
ANTHROPIC_API_KEY=sk-ant-api03-...

# .env.example (committed)
ANTHROPIC_API_KEY=sk-ant-api03-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

### ❌ À Éviter

```typescript
// NE JAMAIS hardcoder la clé dans le code
const apiKey = "sk-ant-api03-..." // ❌ DANGEREUX
```

### 🔒 Vérification

```bash
# La clé ne doit PAS apparaître dans git
git grep "sk-ant-api03"  # Doit retourner 0 résultats
```

---

## 💰 Coûts d'Utilisation

### Claude 3.5 Sonnet (Modèle Utilisé)

- **Input**: $3.00 / million tokens
- **Output**: $15.00 / million tokens

### Estimation par Génération

| Action | Tokens Input | Tokens Output | Coût Unitaire |
|--------|--------------|---------------|---------------|
| Requirement | ~1500 | ~200 | $0.003 |
| Component | ~1500 | ~400 | $0.006 |
| Architecture | ~2000 | ~800 | $0.018 |
| Review | ~2000 | ~500 | $0.012 |

**Coût mensuel estimé** (100 générations/jour):
- Requirements: $9/mois
- Components: $18/mois
- Mix usage: ~$30-50/mois

---

## 🔄 Alternative: OpenAI GPT-4

Si vous préférez OpenAI au lieu de Claude:

### 1. Installer SDK OpenAI

```bash
cd /Users/malek/Arclang/arcviz-web/apps/api
npm install openai
```

### 2. Activer dans `ai-generator.ts`

Décommenter les lignes 63-90:

```typescript
import OpenAI from 'openai'

const openai = process.env.OPENAI_API_KEY 
  ? new OpenAI({ apiKey: process.env.OPENAI_API_KEY })
  : null

export async function generateWithOpenAI(options: AIGenerationOptions): Promise<string> {
  // ... (déjà implémenté, juste décommenter)
}
```

### 3. Modifier `generateWithAI()` ligne 105

```typescript
export async function generateWithAI(...): Promise<string> {
  // return generateWithClaude(...)  // Commenter
  return generateWithOpenAI(...)     // Décommenter
}
```

### 4. Configurer `.env`

```env
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## 🐛 Dépannage

### Problème: "AI generation not yet connected"

**Cause**: `ANTHROPIC_API_KEY` non configurée  
**Solution**: Suivre Étape 2 ci-dessus

### Problème: "Invalid API key"

**Cause**: Clé incorrecte ou expirée  
**Solution**: Régénérer une nouvelle clé sur console.anthropic.com

### Problème: "Rate limit exceeded"

**Cause**: Trop de requêtes  
**Solution**: Attendre 1 minute ou upgrader le plan Anthropic

### Problème: Code généré invalide même avec retry

**Cause**: Prompt système incomplet  
**Solution**: Vérifier que `ARCLANG_SYNTAX_RULES` est bien injecté (routes/ai.ts ligne 84-89)

### Logs de Debug

```bash
# Voir les logs du serveur API
tail -f /tmp/api-server.log

# Ou en mode verbose
LOG_LEVEL=debug npm run dev
```

---

## 📚 Ressources

- **Claude API Docs**: https://docs.anthropic.com/claude/reference/messages_post
- **ArcLang Syntax**: `/Users/malek/Arclang/SYNTAX_ENFORCEMENT_GUIDE.md`
- **API Routes**: `/Users/malek/Arclang/arcviz-web/apps/api/src/routes/ai.ts`
- **AI Generator**: `/Users/malek/Arclang/arcviz-web/apps/api/src/services/ai-generator.ts`

---

## ✅ Checklist de Déploiement Production

- [ ] Clé API Claude configurée dans `.env`
- [ ] `.env` ajouté à `.gitignore`
- [ ] `.env.example` committé (sans vraie clé)
- [ ] Rate limiting configuré (TODO)
- [ ] Monitoring des coûts API activé
- [ ] Logs de génération activés
- [ ] Fallback templates testés (si API down)
- [ ] Tests E2E avec vraie API réussis

---

**🎉 Une fois configuré, l'éditeur ArcViz génère du code ArcLang parfait automatiquement!**
