# Architecture de Cold Search

*(Dernière mise à jour : 26/01/2026)*

## 1. Introduction

**Cold Search** est un moteur de recherche haute performance conçu pour interroger des données statiques (« froides ») avec une latence minimale. Il repose sur une architecture **zéro‑copie** (zero‑copy) exploitant la mémoire‑mappe (`mmap`) et sépare strictement la phase lourde d’indexation de la phase légère de recherche.

Ce document décrit les principes architecturaux, les briques de base (structures de données, index), l’algorithme de recherche pas‑à‑pas, et la jonction entre les trois composants principaux du projet (`indexer`, `server`, `cold_search_core`).

## 2. Vue d’ensemble du système

Cold Search est organisé en trois composants Rust distincts, compilés séparément mais partageant la bibliothèque `cold_search_core` :

```
┌─────────────────────────────────────────────────────────────┐
│                      Données sources                        │
│                (CSV, Parquet, DuckDB, …)                    │
└───────────────────────┬─────────────────────────────────────┘
                        │
                ┌───────▼────────┐
                │    indexer      │
                │   (CLI ETL)     │
                │  • DuckDB       │
                │  • Transformation│
                │  • Génération   │
                │    des index    │
                └───────┬─────────┘
                        │
                ┌───────▼─────────────────────────────────────┐
                │            cold_search_core                  │
                │   (bibliothèque partagée)                    │
                │  • Structures de données                     │
                │  • Algorithmes de recherche                  │
                │  • Parsing de filtres                       │
                └───────┬─────────────────────────────────────┘
                        │
                ┌───────▼────────┐
                │     server      │
                │   (Daemon)      │
                │  • HTTP         │
                │  • Multi‑collection│
                │  • Hot‑reload   │
                └─────────────────┘
                        │
                ┌───────▼────────┐
                │     Client      │
                │   (HTTP/API)    │
                └─────────────────┘
```

### 2.1. Séparation indexation / recherche

- **Indexation (offline)** : le composant `indexer` lit les données sources, calcule les index (texte, spatial, bitmap, range) et produit un ensemble de fichiers binaires optimisés pour la lecture rapide.
- **Recherche (online)** : le `server` mappe ces fichiers en mémoire et répond aux requêtes sans aucune copie, en utilisant les algorithmes implémentés dans `cold_search_core`.

### 2.2. Principe zéro‑copie

Tous les fichiers d’index (`.bin`, `.offsets`, `.idx`, `.fst`, etc.) sont mappés en mémoire via `mmap`. Le noyau assure la mise en cache et le chargement à la demande. Aucune désérialisation n’est nécessaire pour parcourir les structures d’index (RoaringBitmap, FST, FlatGeoBuf).

## 3. Briques de base

### 3.1. Structures de données principales

#### `MetadataStore`
Stocke les documents bruts et leurs positions.
- **Fichiers** : `data.offsets` (offsets 8 octets), `data.bin` (données sérialisées FlatBuffers).
- **Mémoire‑mappe** : deux mappings (`offsets_mmap`, `data_mmap`).
- **Bitmap de tous les IDs** : `all_ids` (RoaringBitmap) contenant `1..num_docs`.

#### `IndexStore`
Regroupe tous les index d’une collection.
- **Index texte** : `Map<Fst>` (Finite State Transducer) par champ, permettant la recherche par préfixe et terme exact.
- **Index bitmap** : `HashMap<String, RoaringBitmap>` par valeur discrète (catégorie, tag, etc.).
- **Index H3 spatial** : `HashMap<u64, RoaringBitmap>` associant une cellule H3 (résolution configurable) aux IDs de documents situés dans cette cellule.
- **Index de plage (range)** : `RangeIndex` contenant les bornes min/max et un bitmap par segment pour les requêtes numériques/date.
- **Index géométrique FlatGeoBuf** : optionnel, utilisé pour affiner les recherches spatiales à l’intérieur de polygones.
- **Coordonnées ECEF** : tableau de `[f32; 3]` (coordonnées cartésiennes géocentriques) permettant le calcul rapide de distances sans conversion latitude/longitude.

#### `Engine`
Le moteur de recherche principal pour une collection.
- **Métadonnées** : `MetadataStore`.
- **Index** : `IndexStore`.
- **Configuration runtime** : `RuntimeConfig` (stopwords par champ).

### 3.2. Types d’index

| Type | Format fichier | Usage | Exemple |
|------|----------------|-------|---------|
| **Texte (FST)** | `.fst` | Recherche par préfixe, exact match | `q=restaur` → tous les documents contenant un mot commençant par « restaur » |
| **Bitmap (Roaring)** | `.idx` | Filtrage par valeur discrète | `category:maison` |
| **H3 spatial** | `.idx` | Recherche géographique approximative (cellules hexagonales) | `lat=48.8566, lon=2.3522, radius=1000` |
| **Range** | `.idx` | Filtrage par intervalle numérique/date | `price BETWEEN 100 AND 500` |
| **FlatGeoBuf** | `.fgb` | Affinage spatial précis (polygones) | `geom WITHIN polygon(...)` |
| **Coordonnées ECEF** | `coords.ecef` | Tri par distance rapide | `ORDER BY distance` |

### 3.3. Formats de sérialisation

- **FlatBuffers** (via `planus`) : schéma défini dans `schema/metadata.fbs`. Permet une lecture zéro‑copie des documents.
- **RoaringBitmap** : format compressé pour les ensembles d’IDs.
- **FST** : automate à états finis pour les dictionnaires de termes.
- **FlatGeoBuf** : format binaire pour les géométries (points, lignes, polygones).

## 4. Algorithme de recherche

La fonction `Engine::search` implémente la recherche combinée (texte, spatial, filtres). Elle suit une pipeline en sept étapes :

1. **Recherche texte** (si `query_text` est fourni) :
   - Normalisation du texte (`slugify`).
   - Consultation de l’index FST correspondant au champ.
   - Union des bitmaps des termes correspondants (recherche par préfixe ou exacte).

2. **Application des filtres bitmap** (si `filter_by` est fourni) :
   - Parsing de l’expression de filtrage (ex: `category:maison AND price > 100`).
   - Intersection des bitmaps des valeurs discrètes (bitmap) et des plages (range).

3. **Intersection initiale** :
   - Le bitmap résultant est l’intersection entre :
     * Le bitmap de la recherche texte (ou `all_ids` si pas de texte).
     * Le bitmap des filtres (ou `all_ids` si pas de filtre).
     * Le bitmap des IDs valides (pas de suppression).

4. **Expansion adaptative des anneaux H3** (si un centre géographique est donné) :
   - Estimation du nombre maximal d’anneaux (`k_max`) à partir du rayon `radius_m`.
   - Boucle de `k = 0` à `k_max` :
     a. Obtention des cellules H3 de l’anneau `k`.
     b. Récupération des bitmaps associés à ces cellules (union).
     c. Intersection avec le bitmap courant.
     d. Si le nombre d’IDs atteint `limit`, sortie anticipée (« early‑out »).

5. **Affinage spatial précis** (si `geometry` est fournie) :
   - Pour chaque ID restant, vérification de l’intersection ou de la distance via FlatGeoBuf (géométries indexées) ou calcul ECEF.
   - Filtrage des documents hors de la géométrie ou au‑delà du rayon.

6. **Regroupement (group‑by)** (si `group_by` est demandé) :
   - Agrégation par valeur d’un champ discret, avec limite par groupe.

7. **Tri, pagination et formatage** :
   - Tri par pertinence (texte), distance (géographique) ou champ numérique.
   - Application de `offset`/`limit`.
   - Récupération des documents bruts via `MetadataStore::get_item`.
   - Sérialisation en JSON ou GeoJSON.

### 4.1. Optimisations clés

- **Early‑out** : dès qu’un anneau H3 fournit assez de résultats (`>= limit`), les anneaux externes sont ignorés.
- **Bitmap Roaring** : les intersections/union sont extrêmement rapides (opérations bit‑à‑bit sur des mots compressés).
- **Coordonnées ECEF** : la distance euclidienne en coordonnées cartésiennes est une approximation précise et très rapide (3 multiplications + sqrt).
- **Mémoire‑mappe** : aucune copie des données, même pour les grands index.

## 5. Jonction entre les composants

### 5.1. `indexer`
- **Responsabilité** : transformation des données sources en fichiers d’index optimisés.
- **Workflow** :
  1. Chargement des données dans DuckDB (avec extension spatiale).
  2. Calcul des statistiques (quantiles pour les range indexes).
  3. Génération des slugs textuels, conversion des coordonnées en cellules H3 et en ECEF.
  4. Écriture des fichiers binaires :
     - `data.bin` / `data.offsets` (FlatBuffers)
     - `*.fst` (FST)
     - `*.idx` (RoaringBitmap pour bitmap, H3, range)
     - `coords.ecef` (vecteurs ECEF)
     - `*.fgb` (FlatGeoBuf, si géométries présentes)
  5. Écriture des métadonnées (`metadata.json`, `runtime_config.json`).

### 5.2. `cold_search_core`
- **Responsabilité** : fournir les structures de données et algorithmes de recherche.
- **Usage par `indexer`** : fonctions de transformation (slugify, conversion H3/ECEF).
- **Usage par `server`** : `Engine`, `MetadataStore`, `IndexStore`, parsing de filtres.
- **Indépendance** : la bibliothèque ne dépend ni de DuckDB ni d’Axum/Tonic, ce qui permet de la réutiliser dans d’autres contextes (CLI, tests unitaires).

### 5.3. `server`
- **Responsabilité** : exposer les index via HTTP/gRPC avec hot‑reload.
- **Chargement des collections** : scan des dossiers racines, chargement des `Engine` dans une `HashMap` partagée (`Arc<AppState>`).
- **Hot‑reload** : surveillance du système de fichiers (`notify`), rechargement atomique via `ArcSwap` lorsqu’un nouveau dossier d’index apparaît ou est mis à jour.

#### 5.3.1 Mécanisme de Hot‑Reload
Le serveur implémente un rechargement à chaud (hot‑reload) qui permet d’ajouter, de mettre à jour ou de supprimer des collections sans redémarrer le processus. Le mécanisme repose sur trois composants Rust :

1. **Surveillance du système de fichiers** : utilisation de la crate `notify` pour écouter les événements de création, modification, suppression et renommage dans les répertoires racines configurés (mode récursif). Les événements de simple accès (`Access`) sont ignorés pour éviter des rechargements intempestifs.

2. **Signal asynchrone** : lorsqu’un événement pertinent est détecté, un signal est envoyé via un canal MPSC (`tokio::sync::mpsc`) à une tâche dédiée qui orchestre le rechargement.

3. **Rechargement atomique** : la tâche de rechargement appelle `load_collections()` qui rescanne les répertoires racines et reconstruit une nouvelle `HashMap<String, Engine>`. Cette nouvelle map est ensuite publiée de manière atomique en utilisant `ArcSwap::store`. Les requêtes en cours continuent de s’exécuter sur l’ancienne version des collections, tandis que les nouvelles requêtes voient immédiatement la nouvelle version. Aucun verrou global n’est nécessaire, ce qui garantit une absence de blocage.

4. **Gestion des erreurs** : si le chargement d’une collection échoue (fichiers corrompus, permissions insuffisantes), cette collection est ignorée et un message d’erreur est loggé, mais les autres collections restent disponibles.

Le flux typique est le suivant :
```
Événement fichier (création d’un dossier `new_collection/`)
  → notify → canal MPSC
  → tâche de rechargement réveillée
  → `load_collections()` scanne tous les répertoires racines
  → construction d’une nouvelle HashMap
  → `state.collections.store(Arc::new(new_map))`
  → les prochaines requêtes vers `/new_collection/search` sont servies.
```

Cette approche garantit une disponibilité continue du serveur tout en permettant des mises à jour atomiques des données indexées.

- **Endpoints HTTP** :
  - `GET /:collection/search` (recherche combinée)
  - `GET /:collection/:key` (récupération directe par ID)
  - `GET /health` (état du serveur)
- **Service gRPC** : `MySearchService` (en cours de parité avec HTTP).

### 5.4. Flux d’une requête typique

```
Client HTTP
  → GET /pois/search?q=café&lat=48.86&lon=2.35&radius=500
  → Axum router
  → AppState::get_engine("pois")
  → Engine::search(
        query_text: "café",
        center: Some((48.86,2.35)),
        radius_m: 500,
        filter_by: None,
        …)
  → (exécution des 7 étapes ci‑dessus)
  → Vec<Item>
  → sérialisation GeoJSON
  → réponse HTTP
```

## 6. Conclusion

Cold Search est une architecture spécialisée pour la recherche sur données statiques, tirant parti de la mémoire‑mappe, d’index compressés (RoaringBitmap, FST) et d’algorithmes adaptatifs (H3 ring expansion). La séparation nette entre indexation et recherche, ainsi que le partage de la logique via `cold_search_core`, permet une maintenance aisée et des performances prévisibles.

