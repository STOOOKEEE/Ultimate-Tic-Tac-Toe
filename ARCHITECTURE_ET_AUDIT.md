# Architecture et audit du projet

Audit mis a jour le 28 avril 2026 sur le depot `Ultimate-Tic-Tac-Toe`.

## Synthese

Le projet est un binaire Rust interactif pour jouer a l'Ultimate Tic-Tac-Toe contre une IA ou en joueur contre joueur local. Il respecte le coeur du sujet demande dans `ASSIGNMENT_SUMMARY.md` :

- interface texte ;
- coups saisis en `colonne ligne`, avec des valeurs de `1` a `9` ;
- choix du mode de jeu ;
- choix du joueur qui commence ;
- affichage du coup joue par l'IA ;
- mode IA contre IA ;
- benchmark automatique contre une IA faible configurable ;
- regles de grille imposee ;
- Minimax avec elagage Alpha-Beta ;
- heuristique maison ;
- limite de temps par coup ;
- departage par nombre de petites grilles gagnees lorsque le plateau est complet sans alignement macro.

Le code compile en debug et en release. Les tests automatises, le formatage et Clippy passent.

Le README n'a pas ete regenere dans cette passe, conformement a la consigne de le traiter plus tard.

## Structure du depot

```text
.
+-- Cargo.toml
+-- Cargo.lock
+-- COMPTE_RENDU.md
+-- IA-5.pdf
+-- ASSIGNMENT_SUMMARY.md
+-- ARCHITECTURE_ET_AUDIT.md
+-- src/
    +-- main.rs
```

## Role des fichiers

- `Cargo.toml` : manifeste Cargo du projet.
- `Cargo.lock` : verrouillage des versions exactes.
- `COMPTE_RENDU.md` : compte rendu synchronise avec l'implementation actuelle.
- `IA-5.pdf` : sujet donne par le professeur.
- `ASSIGNMENT_SUMMARY.md` : resume detaille du sujet.
- `ARCHITECTURE_ET_AUDIT.md` : audit courant du projet.
- `src/main.rs` : implementation du jeu, de l'IA, de l'interface texte et des tests.

## Stack technique

- Langage : Rust
- Edition : Rust 2024
- Type : binaire CLI interactif
- Dependances directes :
  - `lazy_static` pour initialiser la table Zobrist globale ;
  - `rand` pour generer les valeurs aleatoires du hash Zobrist.

## Architecture applicative

### Modele de donnees

Le plateau est represente par `Board`.

```rust
pub struct Board {
    pub cells: [[CellState; 9]; 9],
    pub macros: [MacroState; 9],
    pub active_macro: Option<usize>,
    pub current_player: Player,
    pub hash: u64,
}
```

Responsabilites principales :

- `cells` stocke les 81 cases sous forme de 9 petites grilles de 9 cases.
- `macros` stocke l'etat de chaque petite grille : vide, gagnee par X, gagnee par O ou nulle.
- `active_macro` stocke la petite grille imposee pour le prochain coup.
- `current_player` indique le joueur courant.
- `hash` identifie l'etat pour la table de transposition.

### Regles de jeu

`Board::get_available_moves` genere uniquement les coups legaux :

- grille imposee ouverte : seuls les coups de cette grille sont autorises ;
- grille imposee terminee : le joueur peut jouer dans toute grille encore ouverte ;
- case occupee : jamais proposee.

`Board::make_move` verifie maintenant explicitement :

- `macro_idx < 9` ;
- `micro_idx < 9` ;
- petite grille encore ouverte ;
- case vide ;
- respect de la grille imposee.

Apres un coup valide, la methode met a jour la case, l'etat de la petite grille, la prochaine grille imposee, le joueur courant et le hash.

### Fin de partie

`Board::outcome` distingue :

- `MacroWin(Player)` : alignement de trois petites grilles ;
- `TieBreakWin` : plateau complet, pas d'alignement macro, mais un joueur a plus de petites grilles gagnees ;
- `Draw` : plateau complet et egalite parfaite au nombre de petites grilles gagnees ;
- `Ongoing` : partie en cours.

Cette logique correspond aux modalites du sujet.

## Architecture IA

### Heuristique

`HeuristicParams` regroupe les poids utilises par l'evaluation :

- `macro_win` : valeur d'une petite grille gagnee ;
- `macro_two` : menace de deux petites grilles alignees sur le macro-plateau ;
- `macro_one` : potentiel faible sur le macro-plateau ;
- `center_macro_mult` : multiplicateur de la petite grille centrale ;
- `micro_two` : deux pions alignes dans une petite grille ;
- `micro_one` : un pion dans une ligne encore ouverte ;
- `micro_center` : controle du centre local ;
- `forced_board_threat` : bonus ou malus si le joueur courant est envoye dans une grille dangereuse.

Le score reste interprete du point de vue de X :

- score positif : position favorable a X ;
- score negatif : position favorable a O.

### Recherche

La fonction `minimax` utilise :

- profondeur limitee ;
- maximisation pour X ;
- minimisation pour O ;
- elagage Alpha-Beta ;
- tri des coups par priorite centre, coins, bords ;
- table de transposition ;
- arret propre si le budget de temps expire.

La table de transposition stocke maintenant :

- profondeur ;
- score ;
- meilleur coup ;
- type de borne : exacte, inferieure ou superieure.

Une position interrompue par timeout n'est pas inseree dans la table.

### Iterative deepening

L'IA cherche successivement de la profondeur 1 jusqu'a `MAX_SEARCH_DEPTH` ou expiration du budget de 2 secondes. Le coup joue est le meilleur coup issu de la derniere profondeur terminee completement.

## Fonctionnalites disponibles

- Choix du mode de jeu : joueur contre IA, joueur contre joueur, IA contre IA ou benchmark.
- Partie humain contre IA.
- Partie joueur contre joueur locale.
- Choix du premier joueur.
- Affichage du plateau 9x9.
- Affichage de la grille imposee.
- Saisie `colonne ligne`.
- Validation des coups invalides.
- Coup IA affiche en colonne et ligne.
- Temps de calcul IA affiche.
- Profondeur complete affichee.
- Taille de cache affichee.
- Partie IA contre IA avec affichage des coups et des statistiques.
- Benchmark non interactif avec alternance de l'IA principale entre X et O.
- Resume benchmark : victoires, defaites, nuls, temps moyen par coup, profondeur moyenne.
- Resultat final lisible.
- Departage par petites grilles gagnees.

## Verification effectuee

Commandes lancees apres correction :

```bash
cargo fmt -- --check
cargo test
cargo clippy -- -D warnings
cargo build --release
```

Resultats :

- `cargo fmt -- --check` : reussi ;
- `cargo test` : reussi, 9 tests executes ;
- `cargo clippy -- -D warnings` : reussi ;
- `cargo build --release` : reussi.

## Tests presents

Les tests couvrent :

- conversion colonne/ligne vers representation interne ;
- grille imposee apres un coup ;
- restriction des coups a la grille imposee ;
- liberation si la grille cible est deja terminee ;
- rejet des indices invalides ;
- victoire locale ;
- victoire macro ;
- departage par nombre de petites grilles gagnees ;
- utilisation du poids `macro_two`.

## Points restant a traiter

Le projet est maintenant coherent avec le sujet sur le plan fonctionnel. Les points restants sont surtout de presentation ou d'organisation :

- recreer le README a la fin avec les instructions locales et Google Colab ;
- eventuellement separer `src/main.rs` en modules `board`, `search`, `evaluation` et `cli` ;
- eventuellement stabiliser le hash Zobrist en mode test avec une graine fixe ;
- eventuellement ajouter des adversaires de benchmark supplementaires, par exemple aleatoire ou ancienne version de l'IA.

## Conclusion

Le projet dispose maintenant d'une base propre pour le rendu : les regles essentielles sont testees, l'IA respecte la contrainte Minimax avec Alpha-Beta, les coups sont calcules a la volee, et les documents ne promettent plus de fonctionnalites absentes comme un mode `train`.
