# Cold Search – Moteur de recherche haute performance pour données statiques

**Cold Search** est un moteur de recherche conçu pour interroger rapidement de grandes quantités de données statiques (« froides ») avec une latence minimale. Il repose sur une architecture **zéro‑copie** (memory‑mapping) et sépare strictement la phase d’indexation (lourde) de la phase de recherche (légère).

## Fonctionnalités

- **Recherche texte** : recherche par préfixe ou terme exact via des automates à états finis (FST).
- **Recherche géographique** : indexation spatiale H3 (grille hexagonale) + calcul de distance rapide via coordonnées ECEF.
- **Filtres avancés** : filtrage par catégories (bitmap), par plages numériques (range), par dates.
- **API HTTP complète** : endpoints REST pour la recherche, récupération directe, santé du serveur.
- **Hot‑reload** : rechargement à chaud des collections sans redémarrage du serveur.
- **Multi‑collection** : plusieurs jeux de données peuvent être servis simultanément.
- **Zero‑copy** : les données sont mappées en mémoire (`mmap`), aucune désérialisation n’est nécessaire.

## Architecture

Le projet est organisé en trois composants Rust :

- **`cold_search_core`** : bibliothèque partagée contenant les structures de données, les algorithmes de recherche et le parsing de filtres.
- **`indexer`** : outil CLI qui ingère des données CSV/Parquet via DuckDB et génère les fichiers d’index optimisés.
- **`server`** : serveur HTTP (Axum) qui expose les index en lecture seule avec hot‑reload.

Pour une description détaillée de l’architecture, consultez [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## Installation

### Prérequis

- **Rust** ≥ 1.75 (installable via [rustup](https://rustup.rs/)).
- **Cargo** (inclus avec Rust).
- **DuckDB** (optionnel) : la crate `duckdb` utilisée par l’indexer est compilée avec la fonctionnalité `bundled`, donc aucune installation système n’est requise.

### Construction des binaires

Clonez le dépôt et compilez en mode release :

```bash
git clone <url-du-projet>
cd cold_search
cargo build --release
```

Les binaires produits se trouvent dans `target/release/` :
- `indexer` – outil d’indexation.
- `server` – serveur HTTP.

Vous pouvez également installer les binaires globalement avec `cargo install --path indexer` et `cargo install --path server` (après avoir ajusté les chemins).

---

## Configuration YAML (profil d’indexation)

L’indexeur utilise un fichier YAML (version `1.0`) qui décrit la source des données, les colonnes à stocker et les index à construire.

### Structure minimale

```yaml
version: "1.0"
collection: "nom_de_la_collection"

source:
  path: "chemin/vers/le/fichier.csv"
  query: |
    SELECT id, nom, geom
    FROM read_csv('{path}')

storage:
  primary_key: "id"
  columns: ["id", "nom", "geom"]

indexes:
  - column: "nom"
    type: "text"
    stopwords: ["de", "du", "le"]
```

### Sections détaillées

#### `source`
- `path` : chemin vers le fichier source (CSV, Parquet, etc.). Peut être compressé (`.gz`).
- `query` : requête SQL DuckDB qui transforme les données. La clause `FROM` peut référencer `{path}`. Les colonnes sélectionnées doivent correspondre à celles listées dans `storage.columns`.

#### `storage`
- `primary_key` : nom de la colonne qui sert d’identifiant unique (doit figurer dans `columns`).
- `columns` : liste des colonnes à stocker dans le fichier binaire final (FlatBuffers). Seules ces colonnes seront accessibles via l’API.

#### `indexes`

Chaque entrée de la liste `indexes` définit un index sur une colonne.

##### Types d’index disponibles

| Type | Description | Options spécifiques |
|------|-------------|---------------------|
| `text` | Index textuel (FST) pour recherche par préfixe ou terme exact. | `stopwords` : liste de mots ignorés lors de l’indexation. |
| `bitmap` | Index bitmap (RoaringBitmap) pour les valeurs discrètes (catégories, tags). | – |
| `geo` | Index spatial H3 pour les points géographiques (colonne de type géométrie DuckDB). | `resolution` : résolution H3 (0–15, défaut 8). |
| `range` | Index de plage pour les valeurs numériques continues (entiers, flottants). | `bins` : nombre de segments pour la discrétisation (défaut 20). |
| `date` | Index de plage spécialisé pour les dates (stockées en texte ISO `YYYY‑MM‑DD`). | `bins` : nombre de segments (défaut 20). |
| `datetime` | Index de plage pour les timestamps (texte ISO `YYYY‑MM‑DD HH:MM:SS`). | `bins` : nombre de segments (défaut 20). |

##### Exemples d’index

```yaml
indexes:
  # Texte avec stopwords
  - column: "description"
    type: "text"
    stopwords: ["le", "la", "les", "de", "du", "des"]

  # Bitmap pour une catégorie
  - column: "category"
    type: "bitmap"

  # Geo avec résolution H3 personnalisée
  - column: "geom"
    type: "geo"
    resolution: 8

  # Range pour un prix (20 segments par défaut)
  - column: "price"
    type: "range"
    bins: 20

  # Date (20 segments par défaut)
  - column: "date"
    type: "date"
    bins: 20
```

### Validation

Le profil est validé à l’exécution : la clé primaire doit être présente dans `columns`, et les colonnes indexées doivent être compatibles avec leur type (par exemple, une colonne géométrique pour `geo`).

---

## Indexation des données

Une fois le profil YAML écrit, lancez l’indexeur :

```bash
./target/release/indexer \
  --profile /chemin/vers/profile.yaml \
  --output /répertoire/de/sortie
```

**Options** :
- `--profile` (obligatoire) : chemin vers le fichier YAML.
- `--output` (obligatoire) : répertoire où seront écrits les fichiers d’index.
- `--input` (facultatif) : surcharge le chemin `source.path` défini dans le profil.
- `--debug` : active les logs de débogage.

**Fichiers générés** :
Dans le répertoire de sortie, vous trouverez :
- `data.bin` & `data.offsets` : données sérialisées (FlatBuffers).
- `*.fst` : index textuels.
- `*.idx` : index bitmap, H3 et range (RoaringBitmap).
- `coords.ecef` : coordonnées cartésiennes pour le calcul rapide de distance.
- `*.fgb` : géométries FlatGeoBuf (si présentes).
- `metadata.json` & `runtime_config.json` : métadonnées de la collection.

---

## Lancement du serveur

Le serveur HTTP expose les collections indexées. Il peut surveiller plusieurs répertoires racines et recharger automatiquement les collections modifiées (hot‑reload).

### Commande de base

```bash
./target/release/server [répertoires…]
```

**Arguments** :
- `répertoires` : une liste de répertoires à scanner (optionnel). Par défaut, le serveur utilise `data_out` et `data_bano`.
- `--http_addr` : adresse d’écoute HTTP (défaut `127.0.0.1:3001`).
- `--debug` : active les logs de débogage.

**Exemples** :

```bash
# Serveur avec les répertoires par défaut
./target/release/server

# Surveiller deux dossiers personnalisés
./target/release/server /chemin/collections1 /chemin/collections2

# Changer le port d’écoute
./target/release/server --http_addr 0.0.0.0:8080
```

### Hot‑Reload

Le serveur utilise `notify` pour détecter les changements dans les répertoires racines (création, modification, suppression de dossiers d’index). Lorsqu’un événement pertinent est détecté, il recharge atomiquement les collections via `ArcSwap`, sans interrompre les requêtes en cours.

- **Rechargement atomique** : les nouvelles requêtes voient immédiatement la nouvelle version des collections.
- **Gestion des erreurs** : si une collection ne peut pas être chargée (fichiers corrompus), elle est ignorée et un message d’erreur est loggé.
- **Aucun downtime** : le serveur reste disponible pendant le rechargement.

---

## API HTTP

Le serveur expose les endpoints suivants. Toutes les réponses sont au format JSON (ou GeoJSON).

### `GET /health`

Retourne l’état du serveur et la liste des collections chargées.

**Exemple de réponse** :
```json
[
  {
    "name": "bano",
    "size": 12345,
    "indexes": [
      {"name": "full_name", "type": "text", "field": "full_name"},
      {"name": "geom", "type": "geo", "field": "geom"}
    ]
  }
]
```

### `GET /:collection/get/:id`

Récupère un document par son identifiant (clé primaire ou ID interne).

**Paramètres** :
- `collection` : nom de la collection (dossier d’index).
- `id` : identifiant du document (chaîne ou nombre).

**Exemple** : `GET /bano/get/12345`

### `GET /:collection/search`

Recherche combinée (texte, spatial, filtres).

**Paramètres de requête** :

| Paramètre | Type | Description |
|-----------|------|-------------|
| `q` | string | Texte à rechercher (recherche par préfixe). |
| `lat`, `lon`, `radius` | float | Recherche radiale : centre (latitude, longitude) et rayon en mètres. |
| `min_lat`, `min_lon`, `max_lat`, `max_lon` | float | Bounding box : rectangle géographique. |
| `filter_by` | string | Expression de filtrage (ex: `category:maison && price:>100`). |
| `top_n` | integer | Nombre maximum de résultats à retourner (défaut 100). |
| `group_by` | string | Champ de regroupement (agrégation par valeur). |
| `output` | string | Format de sortie : `geojson` (par défaut) ou `json`. |

**Syntaxe de `filter_by`** :
- `field:value` – égalité pour un index bitmap.
- `field:>value`, `field:<value`, `field:>=value`, `field:<=value` – plages pour un index range.
- `field:[value1,value2]` – IN (plusieurs valeurs).
- `field:![value1,value2]` – NOT IN.
- `field:!=value` – NOT EQUAL.
- `field:[min..max]` – intervalle inclusif (range).
- Les filtres multiples sont combinés avec `&&` (ET logique). Il n’y a pas de support pour `OR` ni pour `NOT` unaire.

Exemples :
  - `category:maison && price:>100`
  - `zip_code:[75000,69000]`
  - `price:[50..200]`

**Exemples de requêtes** :

1. Recherche textuelle simple :
   ```
   GET /bano/search?q=rue%20de%20la
   ```

2. Recherche géographique avec rayon :
   ```
   GET /bano/search?lat=48.8566&lon=2.3522&radius=1000
   ```

3. Combinaison texte + géo + filtre :
   ```
   GET /bano/search?q=restaurant&lat=48.86&lon=2.35&radius=500&filter_by=category:restaurant%20&&%20price:>20
   ```

4. Bounding box :
   ```
   GET /bano/search?min_lat=48.85&min_lon=2.34&max_lat=48.87&max_lon=2.36
   ```

5. Regroupement (group‑by) :
   ```
   GET /bano/search?group_by=category&top_n=5
   ```

**Réponse** :

En format `json`, un tableau d’objets avec les champs stockés et les métadonnées ajoutées (`__distance` pour la distance en mètres, `lat`/`lon` si coordonnées disponibles).

En format `geojson`, une FeatureCollection GeoJSON où chaque feature contient les propriétés du document.

---

## Exemple complet

### 1. Créer un profil YAML

`profile.yaml` :
```yaml
version: "1.0"
collection: "restaurants"

source:
  path: "restaurants.csv"
  query: |
    SELECT
      id,
      name,
      category,
      ST_Point(lon, lat) as geom,
      price
    FROM read_csv('{path}')

storage:
  primary_key: "id"
  columns: ["id", "name", "category", "geom", "price"]

indexes:
  - column: "name"
    type: "text"
    stopwords: ["le", "la", "les"]
  - column: "category"
    type: "bitmap"
  - column: "geom"
    type: "geo"
    resolution: 8
  - column: "price"
    type: "range"
    bins: 10
```

### 2. Générer les index

```bash
./target/release/indexer --profile profile.yaml --output data_restaurants
```

### 3. Démarrer le serveur

```bash
./target/release/server data_restaurants --http-addr 127.0.0.1:3001
```

### 4. Interroger l’API

Recherche des restaurants « pizza » dans un rayon de 1 km :

```bash
curl "http://localhost:3001/restaurants/search?q=pizza&lat=48.8566&lon=2.3522&radius=1000"
```

---

## Notes techniques

- **Performance** : la recherche utilise des intersections de bitmaps Roaring (très rapides) et une expansion adaptative des anneaux H3 (early‑out).
- **Mémoire** : les fichiers d’index sont mappés en mémoire via `mmap` ; la mémoire physique utilisée dépend de la taille des index et de la charge du noyau.
- **Sécurité** : le serveur est conçu pour un usage interne (backend). Il n’inclut pas d’authentification ni de limitation de débit ; à ajouter selon les besoins.
- **Limitations** : les données sont immutables après indexation ; pour mettre à jour une collection, il faut regénérer les index et remplacer le dossier (le hot‑reload détectera le changement).

---

## Développement

Le projet est structuré en workspace Cargo. Pour exécuter les tests d’intégration :

```bash
python3 generate_all_tests.py
```

Les tests génèrent des données synthétiques, lancent l’indexeur et le serveur, puis valident les résultats via des requêtes HTTP.

---

## Licence

MIT

