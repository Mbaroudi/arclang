# ArcViz Platform - Selenium Test Suite

Suite de tests automatisés end-to-end avec Selenium WebDriver pour la plateforme ArcViz.

## 📋 Tests Inclus

### 1. Tests d'Authentification (`01-auth.test.ts`)
- ✓ Inscription d'un nouvel utilisateur
- ✓ Connexion avec identifiants valides
- ✓ Connexion avec identifiants invalides
- ✓ Déconnexion

### 2. Tests de l'Éditeur (`02-editor.test.ts`)
- ✓ Chargement de l'interface éditeur
- ✓ Écriture de code ArcLang
- ✓ Sauvegarde du code dans localStorage
- ✓ Chargement du code sauvegardé
- ✓ Navigation vers le visualizer

### 3. Tests du Visualizer (`03-visualizer.test.ts`)
- ✓ Chargement de la page visualizer
- ✓ Vérification des types de diagrammes disponibles
- ✓ Génération de diagramme operational
- ✓ Génération de diagramme functional
- ✓ Export de diagramme en SVG

### 4. Tests du Chat AI (`04-chat.test.ts`)
- ✓ Ouverture de l'interface chat
- ✓ Envoi de message à l'IA
- ✓ Réception de réponse AI
- ✓ Test action "Générer diagramme"
- ✓ Test action "Insérer code"
- ✓ Fermeture de l'interface chat

### 5. Tests du Visualizer 7D (`05-7d-visualizer.test.ts`)
- ✓ Activation de la vue 7D
- ✓ Navigation dimension Operational
- ✓ Navigation dimension System
- ✓ Navigation dimension Logical
- ✓ Navigation dimension Physical
- ✓ Navigation dimension EPBS
- ✓ Navigation dimension Requirements
- ✓ Navigation dimension Cross-cutting
- ✓ Test du rendu SVG
- ✓ Test de l'interaction avec les nœuds
- ✓ Désactivation de la vue 7D

## 🚀 Installation

```bash
npm install
```

Les dépendances suivantes seront installées :
- `selenium-webdriver` - Framework de test
- `chromedriver` - Driver Chrome
- `ts-node` - Exécution TypeScript
- `@types/selenium-webdriver` - Types TypeScript

## 🏃 Exécution des Tests

### Tous les tests
```bash
npm run test:selenium
```

### Tests individuels
```bash
npm run test:selenium:auth          # Tests d'authentification
npm run test:selenium:editor        # Tests de l'éditeur
npm run test:selenium:visualizer    # Tests du visualizer
npm run test:selenium:chat          # Tests du chat AI
npm run test:selenium:7d            # Tests du visualizer 7D
```

## ⚙️ Configuration

### Variables d'environnement

Créez un fichier `.env` à la racine :

```env
TEST_BASE_URL=http://localhost:3000
TEST_API_URL=http://localhost:4001
```

### Configuration des tests

Modifiez `tests/selenium/config.ts` pour ajuster :
- `timeout` - Délai d'attente (défaut: 30s)
- `testUser` - Utilisateur de test
- Options Chrome (headless, taille fenêtre, etc.)

## 📊 Rapports

Les rapports de tests sont générés automatiquement :

- **Rapport texte** : `tests/selenium/reports/test-report-{timestamp}.txt`
- **Screenshots** : `tests/selenium/screenshots/` (en cas d'échec)

### Format du rapport

```
================================================================================
  SELENIUM TEST REPORT - ArcViz Platform
================================================================================

Total Tests: 35
✓ Passed: 32
✗ Failed: 2
⊘ Skipped: 1
Duration: 145.32s

================================================================================
Test Results:
================================================================================

1. ✓ Auth: Register new user (3.45s)
2. ✓ Auth: Login with valid credentials (2.12s)
3. ✗ Auth: Login with invalid credentials (1.89s)
   Error: Element not found
   Screenshot: tests/selenium/screenshots/auth-failed-1234567890.png
...
```

## 🔧 Mode Headless

Par défaut, les tests s'exécutent en mode headless (sans interface graphique).

Pour désactiver le mode headless (voir Chrome pendant les tests) :

```typescript
// Dans tests/selenium/config.ts
const options = new chrome.Options();
// options.addArguments('--headless');  // Commenter cette ligne
```

## 🐛 Dépannage

### Chrome ne démarre pas
```bash
# Vérifier l'installation de Chrome
which google-chrome

# Réinstaller chromedriver
npm install --save-dev chromedriver
```

### Timeouts
```typescript
// Augmenter le timeout dans config.ts
export const TEST_CONFIG = {
  timeout: 60000,  // 60 secondes au lieu de 30
  ...
};
```

### Erreurs de connexion
Assurez-vous que :
- Le frontend est démarré : `npm run dev` (port 3000)
- Le backend est démarré : `cd apps/api && npm run dev` (port 4001)
- La base de données est disponible : `npm run db:up`

## 📝 Écrire de nouveaux tests

Créez un nouveau fichier dans `tests/selenium/` :

```typescript
import { WebDriver, By } from 'selenium-webdriver';
import { createDriver, TEST_CONFIG, waitForElement, takeScreenshot } from './config';

export async function runMyTests(reporter: any) {
  let driver: WebDriver | null = null;
  const tests = [
    { name: 'My test', fn: testMyFeature },
  ];

  for (const test of tests) {
    const startTime = Date.now();
    try {
      driver = await createDriver();
      await test.fn(driver);
      const duration = Date.now() - startTime;
      reporter.addResult(`My Suite: ${test.name}`, 'passed', duration);
    } catch (error: any) {
      const duration = Date.now() - startTime;
      let screenshot;
      if (driver) {
        screenshot = await takeScreenshot(driver, test.name);
      }
      reporter.addResult(`My Suite: ${test.name}`, 'failed', duration, error.message, screenshot);
    } finally {
      if (driver) {
        await driver.quit();
      }
    }
  }
}

async function testMyFeature(driver: WebDriver) {
  await driver.get(`${TEST_CONFIG.baseUrl}/my-page`);
  // Vos assertions ici
}
```

Puis ajoutez-le dans `run-all-tests.ts` :

```typescript
import { runMyTests } from './06-my-test';

// Dans la fonction main()
await runMyTests(reporter);
```

## 📚 Ressources

- [Selenium WebDriver Docs](https://www.selenium.dev/documentation/webdriver/)
- [ChromeDriver](https://chromedriver.chromium.org/)
- [TypeScript Selenium](https://www.npmjs.com/package/selenium-webdriver)

## 🤝 Contribution

Pour ajouter de nouveaux tests :

1. Créez un nouveau fichier de test dans `tests/selenium/`
2. Suivez la structure des tests existants
3. Ajoutez votre suite de tests dans `run-all-tests.ts`
4. Testez avec `npm run test:selenium`
5. Commitez les changements

## 📄 Licence

MIT
