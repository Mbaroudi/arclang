# Architecture Explorer - Guide d'Utilisation

L'Architecture Explorer est un outil puissant pour visualiser et explorer vos architectures ArcLang de manière interactive.

## 🚀 Génération Rapide

```bash
arclang explorer <fichier.arc>
```

### Exemple
```bash
arclang explorer examples/data_platform_migration.arc
```

Cela génère automatiquement :
- `data_platform_migration_explorer.html` - Document interactif complet
- `data_platform_migration_explorer.json` - Données d'architecture au format JSON

## 📝 Options de Commande

### Spécifier un fichier de sortie
```bash
arclang explorer mon_fichier.arc --output mon_architecture.html
```

### Ouvrir automatiquement dans le navigateur
```bash
arclang explorer mon_fichier.arc --open
```

### Combinaison d'options
```bash
arclang explorer mon_fichier.arc -o architecture.html --open
```

## ✨ Fonctionnalités de l'Explorer

### 1. **Diagramme Interactif**
- Visualisation automatique avec D3.js + Dagre
- Zoom et pan avec la souris
- Contrôles de zoom intégrés :
  - 🔄 Reset - Retour à la vue initiale
  - 📐 Fit - Ajuster à l'écran
  - 🔍 Zoom In/Out - Zoom manuel
- Aucun chevauchement de composants garantis

### 2. **Navigation Flottante (Gauche)**
Une table des matières toujours visible pour naviguer rapidement :
- 🗺️ Diagram
- 📋 Requirements
- 🔧 Components
- 🔗 Interfaces
- ⚙️ Functions
- 🔍 Traceability

**Raccourcis clavier :**
- `T` - Réduire/agrandir la table des matières
- `Escape` - Retour en haut de page

### 3. **Sections Expandables**
- **Requirements** - Groupés par catégorie, avec priorité et traçabilité
- **Components** - Détails complets par couche avec interfaces et fonctions
- **Interfaces** - Flux de connexion entre composants
- **Functions** - Entrées/sorties de chaque fonction
- **Traceability** - Matrice complète de traçabilité

### 4. **Recherche en Temps Réel**
Filtres de recherche dans chaque section :
- Requirements
- Components
- Functions

### 5. **Export Multiple**
Boutons flottants en bas à droite :
- **📄 Export PDF** - Document PDF complet avec diagramme
- **💾 Export HTML** - Document HTML autonome

### 6. **Bouton Retour en Haut**
- Apparaît automatiquement après scroll
- Un clic = retour instantané en haut

## 🎯 Adaptation Automatique

Le template s'adapte automatiquement à **n'importe quelle architecture** :

### Métadonnées Dynamiques
- **Titre** - Extrait automatiquement du premier composant
- **Description** - Générée à partir des statistiques
- **Statistiques** - Nombre de requirements, components, interfaces, functions

### Support Multi-Architecture
```bash
# Architecture plateforme de données
arclang explorer platform_architecture.arc

# Architecture système embarqué
arclang explorer embedded_system.arc

# Architecture microservices
arclang explorer microservices.arc

# Architecture IoT
arclang explorer iot_system.arc
```

Toutes généreront le même type d'explorer avec leurs propres données.

## 📊 Contenu Généré

### Fichier HTML (self-contained)
- **139 KB** environ
- Toutes les données embarquées
- Aucune dépendance externe requise
- Fonctionne offline
- Peut être partagé par email

### Fichier JSON
- Format structuré
- Métadonnées complètes
- Données du diagramme (nodes, edges, layers)
- Requirements avec traçabilité
- Components avec détails
- Interfaces et fonctions

## 🎨 Interface Utilisateur

### Design Professionnel
- Thème Material Design
- Animations fluides
- Responsive design
- Ombres et profondeur
- Transitions douces

### Couleurs
- Bleu principal : `#1976d2`
- Arrière-plan : `#f5f7fa`
- Cartes : Blanc avec ombres
- Hover effects partout

### Accessibilité
- Navigation au clavier
- Sections avec scroll-margin
- Contraste optimal
- Feedback visuel immédiat

## 💡 Cas d'Usage

### 1. Revue d'Architecture
Ouvrez l'explorer lors de revues d'équipe pour :
- Naviguer rapidement entre sections
- Zoomer sur des composants spécifiques
- Vérifier la traçabilité requirements → components

### 2. Documentation Projet
Générez et partagez l'explorer comme documentation :
- Auto-contenu (un seul fichier)
- Toujours à jour avec le code
- Facile à parcourir pour nouveaux membres

### 3. Présentation Client
Exportez en PDF pour présentation :
- Diagramme haute résolution
- Requirements et components listés
- Format professionnel

### 4. Analyse d'Architecture
Utilisez la recherche et les filtres pour :
- Trouver des composants spécifiques
- Identifier les requirements non tracés
- Vérifier les interfaces manquantes

## 🔧 Workflow Typique

```bash
# 1. Développement de l'architecture
vim mon_architecture.arc

# 2. Génération de l'explorer
arclang explorer mon_architecture.arc --open

# 3. Navigation et vérification dans le navigateur
# - Vérifier le diagramme
# - Valider les requirements
# - Vérifier la traçabilité

# 4. Export pour documentation
# - Cliquer sur "Export PDF" dans le navigateur
# - Ou "Export HTML" pour partager

# 5. Intégration CI/CD
arclang explorer mon_architecture.arc -o docs/architecture.html
```

## 📦 Intégration CI/CD

### GitHub Actions
```yaml
- name: Generate Architecture Explorer
  run: |
    arclang explorer architecture.arc -o docs/explorer.html
    
- name: Deploy to GitHub Pages
  uses: peaceiris/actions-gh-pages@v3
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./docs
```

### GitLab CI
```yaml
generate_explorer:
  script:
    - arclang explorer architecture.arc -o public/explorer.html
  artifacts:
    paths:
      - public/
```

## 🎓 Exemples

Le repository contient un exemple complet :
```bash
# Architecture Data Platform Migration
arclang explorer examples/data_platform_migration.arc --open
```

Cet exemple montre :
- 42 requirements
- 24 components sur 8 layers
- 19 interfaces
- 94 functions
- Traçabilité complète

## 🚀 Prochaines Étapes

1. Générez votre premier explorer
2. Explorez toutes les fonctionnalités
3. Partagez avec votre équipe
4. Intégrez dans votre workflow

## 📞 Support

Pour toute question ou problème :
- GitHub Issues : https://github.com/arclang/arclang/issues
- Documentation : https://docs.arclang.io

---

**Note** : L'Architecture Explorer est un template générique qui s'adapte automatiquement à toute architecture ArcLang. Aucune configuration supplémentaire requise!
