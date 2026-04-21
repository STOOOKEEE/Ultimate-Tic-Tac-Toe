# Ultimate Tic Tac Toe - Bot IA (Rust)

Ce projet implémente un bot IA pour le jeu Ultimate Tic Tac Toe en Rust. Il utilise l'algorithme Minimax avec un élagage Alpha-Beta et une fonction heuristique développée pour évaluer les différents états du plateau (Macro-victoires, alignements potentiels, contrôle du centre, etc.).

## Lancer sur Google Colab (Google Cloud)

Pour exécuter ce code Rust sur Google Colab et profiter de la puissance de calcul offerte par leurs machines (pour l'équité des combats d'IA), suivez ces étapes :

1. Ouvrez un nouveau Notebook sur [Google Colab](https://colab.research.google.com/).
2. Dans une cellule de code, installez l'environnement Rust avec cette commande :
   ```bash
   !curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   ```
3. Exécutez cette cellule pour ajouter Cargo au `PATH` de votre environnement Python dans Colab :
   ```python
   import os
   os.environ['PATH'] += ":/root/.cargo/bin"
   ```
4. Transférez le contenu de ce projet (fichiers `Cargo.toml` et le dossier `src/`) vers l'espace de stockage de Colab (par exemple via l'onglet Fichiers à gauche).
5. Compilez le projet en version optimisée (`--release`) pour des performances maximales :
   ```bash
   !cargo build --release
   ```
6. Vous pouvez ensuite lancer le bot pour jouer ou l'interfacer avec l'autre IA :
   ```bash
   !cargo run --release
   ```

## Jouer en local

Assurez-vous d'avoir Rust installé (`rustup`). Puis dans ce dossier :
```bash
cargo run --release
```

## Modes Avancés

Le programme inclut des outils pour tester et améliorer l'IA :

- **Arena** : Fait s'affronter l'IA contre elle-même sur 10 parties pour observer son comportement global.
  ```bash
  cargo run --release -- arena
  ```
- **Train** : Lance un mini-algorithme génétique qui fait muter l'heuristique et garde la meilleure version. C'est l'outil idéal pour "entraîner" votre IA avant le tournoi.
  ```bash
  cargo run --release -- train
  ```

L'IA actuelle utilise les meilleurs paramètres trouvés lors de nos tests d'entraînement.