# Butaca

[![Dernière version](https://img.shields.io/github/v/release/Alexxlml/butaca?include_prereleases&label=version)](https://github.com/Alexxlml/butaca/releases/latest)
[![Téléchargements](https://img.shields.io/github/downloads/Alexxlml/butaca/total)](https://github.com/Alexxlml/butaca/releases)
[![Téléchargements (dernière)](https://img.shields.io/github/downloads/Alexxlml/butaca/latest/total)](https://github.com/Alexxlml/butaca/releases/latest)
[![Licence : MIT](https://img.shields.io/github/license/Alexxlml/butaca)](LICENSE)
![Plateforme : Windows](https://img.shields.io/badge/plateforme-Windows%2010%20%7C%2011-0078D4)
![Fait avec Rust](https://img.shields.io/badge/fait%20avec-Rust-CE422B)

[English](README.md) | [Español](README.es.md) | **Français** | [Deutsch](README.de.md) | [Italiano](README.it.md)

**Discord Rich Presence pour Kodi (non officiel).** Butaca affiche sur votre profil Discord ce que vous regardez sur *n'importe quel* Kodi de votre réseau local (un boîtier CoreELEC, un Android TV, un Raspberry Pi…) sous la forme d'une carte « Regarde … ».

<p>
  <img src="docs/img/card-movie.png" alt="Carte Discord pendant un film" width="300">
  &nbsp;
  <img src="docs/img/card-episode.png" alt="Carte Discord pendant un épisode" width="300">
  &nbsp;
  <img src="docs/img/card-paused.png" alt="Carte Discord en pause" width="300">
</p>

*Cartes d'un film, d'un épisode de série et d'un épisode en pause. Discord affiche le titre, l'affiche, une barre de progression et, pour les épisodes, la saison et le numéro d'épisode.*

## Pourquoi tourne-t-il sur le PC et pas sur Kodi ?

Discord n'expose son interface Rich Presence que sur l'ordinateur où il s'exécute. Butaca tourne donc sur votre PC Windows, demande à Kodi par le réseau ce qui est en lecture et le publie sur le Discord local. Kodi lui-même n'est jamais modifié.

C'est un seul petit `.exe` (sans installateur, sans .NET ni autre runtime) qui reste dans la zone de notification.

## Fonctionnalités

- Fonctionne avec n'importe quel Kodi accessible depuis votre PC ; aucun module complémentaire Kodi n'est nécessaire.
- Cartes pour les films, les épisodes et les « autres vidéos », avec affiche, barre de progression et état de pause.
- Boutons **Trouver Kodi sur le réseau** (mDNS/Zeroconf) et **Tester la connexion**.
- **Mise à jour instantanée :** Butaca écoute les notifications de Kodi (WebSocket, port 9090) : lecture, pause et déplacement dans la vidéo apparaissent aussitôt sur Discord. Si ce port n'est pas disponible, il interroge Kodi toutes les quelques secondes.
- Clé [TMDb](https://www.themoviedb.org/) facultative pour l'affiche, l'année et les réalisateurs lorsque Kodi ne les a pas.
- Interface et texte Discord en 5 langues : English, Español, Français, Deutsch et Italiano.
- Démarrage avec Windows (facultatif), instance unique, journal avec rotation, paramètres appliqués sans redémarrer.
- Le mot de passe de Kodi est stocké dans le **Gestionnaire d'informations d'identification de Windows**, pas dans un fichier.
- Pas de télémétrie, pas de compte, pas de publicité.

## Installation

Butaca nécessite **Windows 10 ou 11 (x64)** et l'**application de bureau Discord** (pas la version web).

1. Téléchargez `Butaca-vX.Y.Z-x64.exe` depuis la [page des Releases](https://github.com/Alexxlml/butaca/releases/latest).
2. *(Facultatif)* Vérifiez le téléchargement avec le fichier `.sha256` publié à côté :
   ```powershell
   Get-FileHash .\Butaca-vX.Y.Z-x64.exe -Algorithm SHA256
   ```
3. Placez le `.exe` où vous voulez et ouvrez-le. La fenêtre de configuration s'ouvre d'elle-même la première fois.

> **Windows SmartScreen :** l'exécutable n'est pas signé numériquement ; Windows peut donc afficher un avertissement « Windows a protégé votre ordinateur ». Choisissez *Informations complémentaires → Exécuter quand même* si vous faites confiance au téléchargement (comparez le SHA-256 ci-dessus, ou compilez-le vous-même, voir plus bas).

## Configurer Kodi

Dans Kodi, ouvrez **Paramètres → Services → Contrôle** et activez :

- **Autoriser le contrôle à distance via HTTP** (port `8080` par défaut).
- **Définissez un nom d'utilisateur et un mot de passe.** Kodi vous le recommande lors de l'activation du contrôle à distance ; suivez ce conseil. Sinon, n'importe qui sur votre réseau pourrait contrôler votre Kodi. Butaca stocke le mot de passe dans le Gestionnaire d'informations d'identification de Windows.
- **Autoriser le contrôle à distance par des applications sur d'autres systèmes.**
- *(Uniquement pour le bouton « Trouver Kodi sur le réseau »)* **Annoncer ces services à d'autres systèmes.**

Ensuite, dans les paramètres de Butaca, saisissez l'adresse IP de votre Kodi (ou appuyez sur **Trouver Kodi sur le réseau**), le nom d'utilisateur et le mot de passe, puis appuyez sur **Tester la connexion**. Enregistrez, c'est terminé.

Butaca ne modifie jamais les paramètres de Kodi. Il a été testé avec Kodi 22 sur CoreELEC.

La mise à jour instantanée utilise le WebSocket de Kodi sur le port `9090`, activé en même temps que le contrôle à distance depuis d'autres systèmes. **Tester la connexion** indique s'il est disponible ; sinon, Butaca continue de fonctionner par sondage. Pour le désactiver ou changer le port, modifiez `use_websocket` et `websocket_port` dans `config.json`.

## Utilisation

- Butaca s'exécute dans la **zone de notification**. Double-cliquez sur l'icône pour ouvrir les paramètres.
- Clic droit sur l'icône pour le menu : état, Paramètres…, Mettre la présence en pause / Reprendre la présence, Ouvrir le dossier des journaux, Rechercher des mises à jour…, À propos (affiche la version installée), Quitter.
- À la fermeture, votre statut Discord est effacé.
- Si vous lancez Butaca une deuxième fois, cela ouvre simplement les paramètres de l'instance déjà active.

**Paramètres**

| Option | Effet |
|---|---|
| Adresse, port, utilisateur et mot de passe de Kodi | Comment joindre Kodi. L'adresse reste vide tant que vous ne l'avez pas saisie. |
| Clé API TMDb (facultative) | Meilleures affiches, année et réalisateurs. [Obtenir une clé gratuite](https://www.themoviedb.org/settings/api). |
| Démarrer avec Windows | Ajoute Butaca aux applications de démarrage de votre utilisateur. |
| Afficher un statut quand rien n'est en lecture | Affiche une carte neutre quand Kodi est inactif. Désactivé par défaut (le statut est effacé). |
| Langue | English, Español, Français, Deutsch ou Italiano. Elle change l'interface **et** le texte que Butaca publie sur Discord (par exemple « En pause » ou « Saison 01, episode 05 »). Par défaut : anglais. |

Les fichiers se trouvent dans `%APPDATA%\Butaca\` : `config.json` (sans le mot de passe) et `butaca.log` (avec rotation, 1 Mo × 3).

## Mettre à jour et désinstaller

**Mettre à jour :** menu de la zone de notification → *Rechercher des mises à jour…* ouvre la page de la dernière version. Téléchargez le nouvel `.exe`, quittez Butaca (menu → Quitter) et remplacez l'ancien fichier. Vos paramètres sont conservés. Comparez la version affichée dans *À propos* avec celle de la page des Releases.

**Désinstaller :** décochez *Démarrer avec Windows* dans les paramètres, quittez Butaca et supprimez l'`.exe`. Pour ne laisser aucune trace, supprimez aussi le dossier `%APPDATA%\Butaca\` et, si vous avez enregistré un mot de passe Kodi, l'entrée `kodi-password.Butaca` dans le *Gestionnaire d'informations d'identification* de Windows.

## Comment l'affiche est choisie

1. L'illustration que Kodi possède déjà, si c'est une URL `https` publique.
2. [TMDb](https://www.themoviedb.org/), si vous renseignez une clé API.
3. L'icône propre à Butaca.

Discord ne peut pas afficher les chemins locaux ni les illustrations `smb://` ; elles sont donc ignorées. Les URL des affiches sont téléchargées par Discord, pas par Butaca.

## Confidentialité et sécurité

- Butaca communique avec : votre Kodi (HTTP et WebSocket sur votre réseau local), Discord (IPC local sur votre PC) et, **uniquement si vous renseignez une clé**, TMDb.
- Ce que vous regardez est publié sur votre profil Discord, comme le veut le principe du Rich Presence. Utilisez *Mettre la présence en pause* dans le menu de la zone de notification pour l'arrêter.
- Le mot de passe de Kodi est stocké dans le Gestionnaire d'informations d'identification de Windows. La clé TMDb est enregistrée en clair dans `config.json` ; c'est une clé gratuite liée à votre propre compte.
- L'authentification HTTP de Kodi circule non chiffrée sur votre réseau. Gardez le contrôle à distance de Kodi dans votre réseau domestique et ne l'exposez jamais sur Internet.
- Le binaire ne contient aucune clé API ; la seule valeur intégrée est l'identifiant public de l'application Discord.

## Dépannage

| Symptôme | À vérifier |
|---|---|
| « Discord n'est pas ouvert » | Ouvrez l'application **de bureau** Discord, connecté. Butaca et Discord doivent tourner avec le même utilisateur Windows et le même niveau de privilèges (tous deux normaux, pas « en tant qu'administrateur »). |
| « En attente de Kodi » | Kodi est éteint, en veille, ou l'adresse est incorrecte. Utilisez **Tester la connexion**. |
| « Nom d'utilisateur ou mot de passe incorrect » | Utilisez les identifiants de la page *Services → Contrôle* de Kodi. |
| « Trouver Kodi sur le réseau » ne trouve rien | Activez *Annoncer ces services à d'autres systèmes* dans Kodi, ou saisissez l'IP à la main. |
| Rien n'apparaît sur Discord | Dans Discord, *Paramètres → Confidentialité de l'activité* → *Partager mon activité* doit être activé. |

Le journal (menu de la zone de notification → *Ouvrir le dossier des journaux*) enregistre chaque changement d'état.

## Compiler depuis les sources

Il faut [Rust](https://rustup.rs/) (stable, chaîne d'outils MSVC) et le SDK Windows.

```powershell
git clone https://github.com/Alexxlml/butaca.git
cd butaca
cargo build --release
```

L'exécutable est `target\release\butaca.exe`. `cargo test` lance les tests unitaires.

## Feuille de route

- Télécharger et installer les mises à jour depuis l'application.

## Mentions légales

Kodi® is a registered trademark of the XBMC Foundation. Discord is a trademark of Discord Inc. This project is not affiliated with, endorsed by, or sponsored by either.

This product uses the TMDB API but is not endorsed or certified by TMDB.

Butaca est distribué sous [licence MIT](LICENSE). Les licences tierces figurent dans `THIRD-PARTY-LICENSES.txt`, publié avec chaque version.

## Crédits

- Le protocole IPC de Discord suit [service.discord.richpresence](https://github.com/Hiumee/service.discord.richpresence) de Hiumee (MIT).
- L'expérience d'application en zone de notification s'inspire d'[AMWin-RP](https://github.com/PKBeam/AMWin-RP).
