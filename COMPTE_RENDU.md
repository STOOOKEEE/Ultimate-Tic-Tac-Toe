# Compte-Rendu : Intelligence Artificielle - Ultimate Tic Tac Toe

**Auteur :** Armand Sechon  
**Langage :** Rust  
**Algorithme :** Minimax avec élagage Alpha-Beta, Iterative Deepening et Table de Transposition.

## 1. Introduction
Ce projet consiste en l'implémentation d'un bot capable de jouer de manière autonome à l'Ultimate Tic Tac Toe. L'objectif principal était de concevoir une IA alliant **profondeur de réflexion** et **rapidité d'exécution** pour maximiser les points lors des tournois.

## 2. Choix Techniques
### 2.1 Le langage Rust
Le choix de **Rust** a été motivé par le besoin de performance brute. Contrairement à Python, Rust est un langage compilé qui permet d'explorer des millions de positions par seconde. Dans un jeu où le score dépend de la vitesse, Rust offre un avantage compétitif majeur.

### 2.2 Zobrist Hashing & Table de Transposition
Pour optimiser les calculs, nous avons implémenté une **Table de Transposition** (cache). Chaque état du plateau est haché via la technique de **Zobrist Hashing** (attribution d'une signature unique 64 bits par position).
*   **Gain :** L'IA ne recalcule jamais une position déjà analysée, ce qui permet de gagner 2 à 3 niveaux de profondeur dans le même laps de temps.

## 3. Algorithme de Recherche
### 3.1 Minimax avec Élagage Alpha-Beta
L'algorithme explore l'arbre des coups possibles. L'élagage Alpha-Beta permet d'ignorer les branches de l'arbre dont on sait mathématiquement qu'elles ne seront pas choisies par un joueur optimal.

### 3.2 Iterative Deepening
Plutôt que de fixer une profondeur arbitraire, l'IA utilise l'**Iterative Deepening**. Elle calcule la profondeur 1, puis 2, puis 3... jusqu'à ce que les 2 secondes imparties soient écoulées. Cela garantit une réponse optimale quel que soit le temps restant.

### 3.3 Move Ordering (Tri des coups)
Pour maximiser l'efficacité de l'élagage Alpha-Beta, nous trions les coups à chaque niveau :
1.  **Le Centre** (Priorité max)
2.  **Les Coins**
3.  **Les Bords**
Plus l'algorithme trouve un "bon" coup tôt, plus il peut couper de branches inutiles.

## 4. Heuristique et Entraînement
### 4.1 La fonction d'évaluation
Notre heuristique évalue le plateau selon plusieurs critères pondérés :
*   **Macro-Victoires (1060 pts)** : Le gain d'une petite grille.
*   **Potentiel (16 pts)** : Deux pions alignés dans une grille.
*   **Contrôle du centre (x1.22)** : Bonus multiplicateur pour la grille centrale.
*   **Malus de destination (-150 pts)** : L'IA évite d'envoyer l'adversaire dans une grille où il est sur le point de gagner.

### 4.2 Optimisation par Algorithme Génétique
Les paramètres ci-dessus n'ont pas été choisis au hasard. Nous avons développé un mode `train` faisant s'affronter des versions "mutantes" de l'IA. Après des centaines de matchs automatisés à une profondeur de 10, nous avons extrait les poids statistiquement les plus performants.

## 5. Conclusion
Grâce à la combinaison d'un langage haute performance et de structures de données optimisées (Zobrist/TT), cette IA est capable d'analyser jusqu'à la profondeur **15 ou 20** en situation de tournoi, tout en respectant scrupuleusement les contraintes de temps.
