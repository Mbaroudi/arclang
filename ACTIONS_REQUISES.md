# 🔧 Actions Requises - URGENT

## ❌ Problème 1: Chat Non Fonctionnel

### Cause
L'interface chat nécessite:
1. ✅ **API URL fixée** - Le hook pointe maintenant vers `http://localhost:4000`
2. ❌ **Utilisateur authentifié** - Les routes `/api/chat/*` vérifient `request.user?.id`
3. ❌ **API backend démarrée** - Port 4000

### Solution Immédiate

```bash
# 1. Créer utilisateur de test
cd /Users/malek/Arclang/arcviz-web/apps/api
npx ts-node prisma/seed-dev-user.ts

# 2. Démarrer l'API
npm run dev  # Port 4000

# 3. Dans le navigateur - Se connecter
# Ouvrir: http://localhost:3002/login
# Email: test@arclang.dev
# Password: test123

# 4. Tester le chat
# Ouvrir: http://localhost:3002/visualizer
# Cliquer sur le bouton flottant AI Assistant
```

### Alternative: Mode Sans Auth (Development Only)

Modifier `/apps/api/src/routes/chat.ts` ligne 42:

```typescript
// AVANT:
if (!userId) {
  return reply.code(401).send({ error: 'Unauthorized' });
}

// APRÈS (DEV ONLY):
const userId = (request as any).user?.id || 'dev-user-123';
```

---

## ❌ Problème 2: Diagrammes Obsolètes

### État Actuel
- ✅ **Operational Activity** - Mis à jour
- ✅ **Component Architecture** - Mis à jour
- ✅ **System Context** - Nouveau (créé avec 7 dimensions)
- ✅ **Allocation** - Nouveau (créé avec 7 dimensions)
- ❌ **Functional Dataflow** - Ancien (pas de légende, textes débordent)
- ❌ **Sequence** - Ancien
- ❌ **State Machine** - Ancien
- ❌ **Physical** - Ancien
- ❌ **Class** - Ancien
- ❌ **Tree** - Ancien
- ❌ **Capability** - Ancien
- ❌ **Functional Chain** - Ancien

### 7 Dimensions Validées à Appliquer

1. **Légende Externe** - En bas, hors du diagramme
2. **Texte Wrapping** - Max 2 lignes avec ellipsis
3. **Edges Bidirectionnelles** - Flèches aux 2 bouts
4. **Self-loops** - Arcs circulaires pour auto-références
5. **Colors Material Design** - Palette cohérente
6. **Spacing** - Marges: top=120, bottom=180
7. **Professional Labels** - Stéréotypes italiques

### Fichiers à Mettre à Jour

```
/arcviz-web/apps/diagram-service/src/renderers/
├── dataflow.ts        ❌ À METTRE À JOUR
├── sequence.ts        ❌ À METTRE À JOUR
├── state-machine.ts   ❌ À METTRE À JOUR
├── physical.ts        ❌ À METTRE À JOUR
├── class.ts           ❌ À METTRE À JOUR
├── tree.ts            ❌ À METTRE À JOUR
├── capability.ts      ❌ À METTRE À JOUR
├── functional-chain.ts ❌ À METTRE À JOUR (+ highlighting)
├── operational.ts     ✅ DÉJÀ MIS À JOUR
├── component.ts       ✅ DÉJÀ MIS À JOUR
├── system-context.ts  ✅ NOUVEAU
└── allocation.ts      ✅ NOUVEAU
```

### Templates à Suivre

**Utiliser comme référence:**
- `/allocation.ts` - Pour légende externe
- `/system-context.ts` - Pour layout professionnel
- `/operational.ts` ou `/component.ts` - Pour wrapping de texte

---

## 🎯 Plan d'Action Immédiat

### Étape 1: Débloquer le Chat (15 min)

```bash
# Option A: Avec Auth (Recommandé)
cd /Users/malek/Arclang/arcviz-web/apps/api
npx ts-node prisma/seed-dev-user.ts
npm run dev
# Puis se connecter sur http://localhost:3002/login

# Option B: Sans Auth (Dev rapide)
# Modifier chat.ts pour skip auth check
```

### Étape 2: Tester le Chat (5 min)

1. Ouvrir `http://localhost:3002/visualizer`
2. Cliquer bouton flottant
3. Taper: "Hello"
4. Vérifier réponse AI

### Étape 3: Mettre à Jour Diagrammes (2-3h)

**Priorité Haute:**
1. ✅ Dataflow (le plus utilisé)
2. ✅ Sequence
3. ✅ State Machine

**Priorité Moyenne:**
4. Physical
5. Class
6. Tree

**Priorité Basse:**
7. Capability
8. Functional Chain (+ feature highlighting)

### Étape 4: Tests de Validation

Pour chaque diagramme mis à jour, vérifier:
- ✅ Légende en bas (outside diagram)
- ✅ Pas de text overflow
- ✅ Edges bidirectionnelles si pertinent
- ✅ Self-loops si nécessaire
- ✅ Couleurs Material Design
- ✅ Margins cohérentes

---

## 📋 Checklist de Vérification

### Chat Interface
- [ ] API backend démarre sur port 4000
- [ ] Utilisateur de test créé
- [ ] Connexion réussie
- [ ] Bouton flottant visible
- [ ] Panneau s'ouvre au clic
- [ ] Input text fonctionnel
- [ ] Messages s'affichent
- [ ] Réponses AI arrivent

### Diagrammes
- [ ] Dataflow mis à jour
- [ ] Sequence mis à jour
- [ ] State Machine mis à jour
- [ ] Physical mis à jour
- [ ] Class mis à jour
- [ ] Tree mis à jour
- [ ] Capability mis à jour
- [ ] Functional Chain mis à jour
- [ ] Tous les diagrammes testés
- [ ] Aucune anomalie visuelle

---

## 🚨 Erreurs Connues à Éviter

### Chat
1. ❌ **401 Unauthorized** → Pas connecté
2. ❌ **CORS error** → API pas démarrée
3. ❌ **Network error** → Mauvaise URL API

### Diagrammes
1. ❌ **Text overflow** → Pas de wrapping
2. ❌ **Légende overlap** → Pas externe
3. ❌ **Edges doubles** → Pas bidirectionnelles
4. ❌ **Self-loop linéaire** → Pas d'arc circulaire

---

## ✅ Validation Finale

Une fois tout complété:

```bash
# Terminal 1: API
cd /Users/malek/Arclang/arcviz-web/apps/api
npm run dev

# Terminal 2: Web
cd /Users/malek/Arclang/arcviz-web/apps/web  
npm run dev

# Terminal 3: Tests
cd /Users/malek/Arclang/arcviz-web/apps/diagram-service
npm run test  # Si tests existent

# Navigateur:
# 1. http://localhost:3002/login
# 2. http://localhost:3002/visualizer
# 3. Tester chat + tous les diagrammes
```

---

## 📞 Support

Si problèmes persistent:

1. **Chat ne répond pas** → Vérifier console navigateur (F12)
2. **Diagrammes incorrects** → Vérifier logs `diagram-service`
3. **Erreurs TypeScript** → `npm run build` pour voir détails

**Priorité:** Débloquer le chat d'abord, puis mettre à jour les diagrammes.
