# Guide d'Exécution des Tests Selenium

## ⚠️ Prérequis OBLIGATOIRES

Avant de lancer les tests, l'application DOIT être démarrée.

### 1️⃣ Démarrer la base de données

```bash
npm run db:up
```

### 2️⃣ Démarrer le backend (API)

Dans un premier terminal:
```bash
cd apps/api
npm run dev
```

Attendez de voir: `Server running on http://localhost:4001`

### 3️⃣ Démarrer le frontend (Web)

Dans un deuxième terminal:
```bash
cd apps/web
npm run dev
```

Attendez de voir: `ready started server on 0.0.0.0:3000`

### 4️⃣ Vérifier que tout fonctionne

```bash
# Vérifier frontend (doit retourner du HTML)
curl http://localhost:3000

# Vérifier backend (doit retourner {"message":"OK"})
curl http://localhost:4001/health
```

## 🚀 Exécution des Tests

### Tous les tests (31 tests)
```bash
npm run test:selenium
```

### Tests par catégorie

```bash
# Tests d'authentification (4 tests)
npm run test:selenium:auth

# Tests de l'éditeur (5 tests)
npm run test:selenium:editor

# Tests du visualizer (5 tests)
npm run test:selenium:visualizer

# Tests du chat AI (6 tests)
npm run test:selenium:chat

# Tests du visualizer 7D (11 tests)
npm run test:selenium:7d
```

## 📊 Résultats

Les tests génèrent automatiquement:
- **Rapport**: `tests/selenium/reports/test-report-{timestamp}.txt`
- **Screenshots** (si échec): `tests/selenium/screenshots/`

## 🐛 Résolution des Problèmes

### ERR_CONNECTION_REFUSED
```
Error: unknown error: net::ERR_CONNECTION_REFUSED
```
**Solution**: L'application n'est pas démarrée. Suivez les étapes 1-3 ci-dessus.

### ChromeDriver version mismatch
```
Error: This version of ChromeDriver only supports Chrome version X
```
**Solution**: 
```bash
# Vérifier version de Chrome
google-chrome --version  # ou chrome://version

# Installer la bonne version de chromedriver
npm uninstall chromedriver
npm install --save-dev chromedriver@{VERSION}
```

### Port déjà utilisé
```
Error: listen EADDRINUSE :::3000
```
**Solution**:
```bash
# Trouver et tuer le processus
lsof -ti:3000 | xargs kill -9
lsof -ti:4001 | xargs kill -9
```

### Tests timeout
```
Error: Element not found within 30000ms
```
**Solution**: Augmenter le timeout dans `tests/selenium/config.ts`:
```typescript
export const TEST_CONFIG = {
  timeout: 60000,  // 60 secondes au lieu de 30
  ...
};
```

## 🎯 Commande Tout-en-Un

Pour démarrer ET tester (nécessite 3 terminaux):

**Terminal 1 - Base de données**:
```bash
npm run db:up
```

**Terminal 2 - Backend + Frontend**:
```bash
npm run dev
```

**Terminal 3 - Tests** (attendre 10s que l'app démarre):
```bash
sleep 10 && npm run test:selenium
```

## 📈 Objectif

**But**: 31/31 tests passés ✅

État actuel:
- ✓ ChromeDriver configuré
- ✓ Tests créés
- ⏳ Application à démarrer
- ⏳ Tests à exécuter

## 🔍 Vérification Rapide

Script pour vérifier si tout est prêt:

```bash
#!/bin/bash
echo "🔍 Vérification de l'environnement..."

# Vérifier port 3000
if curl -s http://localhost:3000 > /dev/null; then
    echo "✅ Frontend OK (port 3000)"
else
    echo "❌ Frontend non accessible (port 3000)"
fi

# Vérifier port 4001
if curl -s http://localhost:4001/health > /dev/null; then
    echo "✅ Backend OK (port 4001)"
else
    echo "❌ Backend non accessible (port 4001)"
fi

# Vérifier Chrome
if which google-chrome > /dev/null || which chrome > /dev/null; then
    echo "✅ Chrome installé"
else
    echo "❌ Chrome non trouvé"
fi

echo ""
echo "🚀 Prêt pour les tests? Lancez: npm run test:selenium"
```

Sauvegardez ce script dans `check-env.sh` et exécutez:
```bash
chmod +x check-env.sh
./check-env.sh
```
