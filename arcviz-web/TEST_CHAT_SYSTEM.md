# Test du Système de Chat ArcLang

## ✅ Checklist d'Installation

### 1. Base de Données
- [x] Schema Prisma mis à jour
- [x] Migration créée: `20251029121951_add_chat_system`
- [x] Prisma Client généré

### 2. Backend (API)
- [x] Services créés:
  - `src/services/conversational-ai.ts`
  - `src/services/learning.ts`
- [x] Routes créées: `src/routes/chat.ts`
- [x] Plugin services: `src/plugins/services.ts`
- [x] Intégration dans `src/index.ts`

### 3. Frontend (Web)
- [x] Composants chat créés (7 composants)
- [x] Hook `useChat` créé
- [x] Page chat: `app/chat/page.tsx`
- [x] Layout chat: `app/chat/layout.tsx`

## 🚀 Démarrage

### 1. Démarrer l'API
```bash
cd /Users/malek/Arclang/arcviz-web/apps/api
npm run dev
```

### 2. Démarrer le Frontend
```bash
cd /Users/malek/Arclang/arcviz-web/apps/web
npm run dev
```

### 3. Accéder au Chat
Ouvrir: http://localhost:3002/chat

## 🧪 Scénarios de Test

### Test 1: Création de Conversation
1. Aller sur `/chat`
2. Vérifier que l'interface s'affiche
3. Vérifier le message de bienvenue

**Résultat attendu**: Interface chat avec suggestions d'exemples

### Test 2: Premier Message
1. Taper: "Generate a system context diagram"
2. Cliquer "Send"
3. Attendre la réponse de l'AI

**Résultat attendu**: 
- Message utilisateur affiché
- Indicateur de chargement
- Réponse AI avec contenu généré

### Test 3: Feedback Positif
1. Sur une réponse AI, cliquer 👍
2. Vérifier que le bouton devient vert

**Résultat attendu**: Feedback enregistré en base de données

### Test 4: Feedback Négatif + Correction
1. Sur une réponse AI, cliquer 👎
2. Cliquer "Correct"
3. Remplir le formulaire:
   - Type: "Semantic Error"
   - Feedback: "The function SF-009 should be allocated to LC-003"
   - Corrected Code: (modifier le code)
4. Cliquer "Submit Correction"

**Résultat attendu**:
- Nouvelle réponse AI générée avec correction
- Pattern d'erreur enregistré dans `error_patterns`

### Test 5: Apprentissage
1. Faire plusieurs corrections similaires
2. Créer une nouvelle conversation
3. Demander le même type de diagramme

**Résultat attendu**: L'AI évite les erreurs précédentes (prompt enrichi)

## 📊 Vérification Base de Données

```sql
-- Vérifier les conversations
SELECT * FROM conversations ORDER BY created_at DESC LIMIT 5;

-- Vérifier les messages
SELECT id, role, LEFT(content, 50) as content, created_at 
FROM messages 
ORDER BY created_at DESC LIMIT 10;

-- Vérifier les feedbacks
SELECT m.id, m.role, f.rating, f.helpful 
FROM messages m 
LEFT JOIN feedbacks f ON f.message_id = m.id 
WHERE f.id IS NOT NULL;

-- Vérifier les corrections
SELECT id, issue_type, LEFT(user_feedback, 50) as feedback, resolved 
FROM corrections 
ORDER BY created_at DESC LIMIT 5;

-- Vérifier les patterns appris
SELECT error_signature, solution_pattern, frequency, success_rate 
FROM error_patterns 
ORDER BY frequency DESC LIMIT 10;
```

## 🐛 Dépannage

### Erreur: "ANTHROPIC_API_KEY not configured"
```bash
cd /Users/malek/Arclang/arcviz-web/apps/api
echo "ANTHROPIC_API_KEY=your_key_here" >> .env
```

### Erreur: "Unauthorized"
Vérifier que l'authentification fonctionne:
```bash
# Créer un utilisateur test
curl -X POST http://localhost:4000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123","name":"Test User"}'
```

### Erreur: "Cannot find module '@/hooks/use-chat'"
```bash
cd /Users/malek/Arclang/arcviz-web/apps/web
npm install
```

## 📝 API Endpoints Disponibles

### Conversations
- `POST /api/chat/conversations` - Créer conversation
- `GET /api/chat/conversations` - Lister conversations
- `GET /api/chat/conversations/:id` - Récupérer conversation
- `DELETE /api/chat/conversations/:id` - Supprimer conversation

### Messages
- `POST /api/chat/conversations/:id/messages` - Envoyer message

### Feedback & Corrections
- `POST /api/chat/messages/:id/feedback` - Soumettre feedback
- `POST /api/chat/messages/:id/correct` - Soumettre correction

## 🎯 Prochaines Étapes (Phase 2)

### Semaine 2-3: Améliorations
- [ ] Intégration avec diagram-service pour génération SVG
- [ ] WebSocket pour réponses en streaming
- [ ] Export de conversations en PDF
- [ ] Dashboard des métriques d'apprentissage
- [ ] Sidebar avec historique des conversations
- [ ] Recherche dans les conversations
- [ ] Support pour images/diagrammes dans le chat
- [ ] Autocomplétion intelligente

### Semaine 4: Learning Avancé
- [ ] Analyse automatique des patterns d'erreurs
- [ ] Suggestions proactives basées sur l'historique
- [ ] Préférences utilisateur personnalisées
- [ ] Export des learnings pour autres utilisateurs
- [ ] A/B testing des prompts

## 📈 Métriques à Suivre

1. **Engagement**:
   - Nombre de conversations par jour
   - Longueur moyenne des conversations
   - Temps de réponse AI

2. **Qualité**:
   - Ratio 👍/👎
   - Taux de corrections
   - Taux de résolution des corrections

3. **Apprentissage**:
   - Nombre de patterns uniques appris
   - Évolution du taux de corrections (devrait diminuer)
   - Success rate par type d'erreur

## 🎉 Succès!

Si tous les tests passent, vous avez maintenant:
- ✅ Un système de chat conversationnel complet
- ✅ Un système de feedback utilisateur
- ✅ Un système d'apprentissage automatique
- ✅ Une base pour l'amélioration continue de l'AI

**Félicitations! 🚀**
