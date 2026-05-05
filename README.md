# Ultimate Tic Tac Toe - IA

Projet Rust d'Ultimate Tic Tac Toe pour le module de Fondements de l'IA.

## Objectif

Le programme permet de jouer a l'Ultimate Tic Tac Toe contre une IA, de faire jouer deux IA entre elles et de lancer des benchmarks rapides. L'IA utilise une seule methode de decision: le moteur fort inspire du depot public `jojo2504/ultimate-tic-tac-toe`, avec recherche Negamax/Alpha-Beta, approfondissement iteratif, table de transposition et evaluation NNUE via `databin/gen160_weights.bin`.

## Regles implementees

- Plateau global de 9 x 9 cases, organise en 9 grilles de morpion 3 x 3.
- Les coups sont saisis en `colonne ligne`, avec des valeurs de 1 a 9.
- `X` commence toujours la partie; au lancement, on choisit si l'IA ou le joueur prend `X`.
- Le coup precedent impose la petite grille du prochain coup.
- Si la petite grille imposee est deja gagnee ou pleine, le joueur peut jouer dans n'importe quelle petite grille encore jouable.
- Une petite grille gagnee ou pleine ne peut plus recevoir de coup.
- La partie s'arrete des qu'un joueur aligne 3 petites grilles gagnees sur le plateau global.
- Si le plateau est complet sans alignement global, le programme applique le departage du sujet: le joueur avec le plus de petites grilles gagnees remporte le combat; egalite si les deux scores sont identiques.

## Lancer le projet

```bash
cargo run --release
```

Le menu propose:

1. joueur contre IA
2. joueur contre joueur
3. IA contre IA
4. benchmark
5. mode tournoi, qui affiche seulement les coups de l'IA au format `colonne ligne`

## Tests

```bash
cargo test
```

Les tests couvrent notamment la conversion des coordonnees, la grille imposee, la liberation vers un choix libre quand la grille cible est decidee, l'interdiction de jouer dans une grille decidee, l'arret apres victoire globale et le departage sur plateau complet.

## Structure

- `src/game.rs`: etat du plateau, validation des coups, regles et tests de regles.
- `src/ai.rs`: selection du meilleur coup via le moteur fort uniquement.
- `src/strong.rs`: adaptateur entre le plateau principal et le moteur fort.
- `src/core.rs`, `src/movegen.rs`, `src/search.rs`, `src/network.rs`, `src/constants.rs`: moteur fort bitboard, generation des coups, recherche et evaluation NNUE.
- `src/coords.rs`: conversion entre saisie utilisateur 1..9 et coordonnees internes 0..8.
- `src/cli.rs`: interface texte et modes de jeu.
- `src/main.rs`: point d'entree.

## Notes d'implementation

Le moteur ne contient pas de dictionnaire de coups. Les coups sont generes legalement depuis l'etat courant, puis evalues par recherche a la volee. Le fichier de poids sert uniquement a l'evaluation du moteur fort; il doit etre present pour que l'IA joue.
