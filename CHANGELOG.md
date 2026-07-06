## [1.1.0] - 2026-07-06

### 🚀 Features

- *(icon-picker)* Added google favicon api lookup

### 💼 Other

- *(app)* Only do dev things on debug builds
## [1.0.1] - 2026-06-05

### 🐛 Bug Fixes

- *(icon-picker)* Improved icon fetcher (#58)
- *(icon-picker)* Fix for mobile smallscreen (#59)

### ⚙️ Miscellaneous Tasks

- *(ci)* Added timout on release action
- *(release)* V1.0.1
## [1.0.0] - 2026-05-12

### 🚀 Features

- *(flatpak)* Update to gnome 50  runtime
- *(not-breaking)* [**breaking**] Update to stable version 1

### 💼 Other

- *(deps)* Bump rustls-webpki in the cargo group across 1 directory (#54)
- *(common)* Fixed cargo shear warnings
- *(deps)* Bump rand in the cargo (unsound)

### ⚙️ Miscellaneous Tasks

- *(dev)* Change cargo-machete with cargo-shear
- *(dev)* Update dev-container
- *(release)* V1.0.0
## [0.12.0] - 2026-04-21

### 🚀 Features

- *(browser)* Added system binaries for zen browser (#52)

### 💼 Other

- *(deps)* Bump rand in the cargo group across 1 directory (#49)
- *(deps)* Bump rand in the cargo group across 1 directory (#50)
- *(deps)* Bump rustls-webpki from 0.103.10 to 0.103.12 in the cargo group across 1 directory (#53)

### 🎨 Styling

- Fix pedantic clippy errors

### ⚙️ Miscellaneous Tasks

- *(dev)* Cleanup deps
- *(desktop-file)* Update rand api for updated version
- *(dev)* Update dev-container
- *(dev)* Added zen-browser system browser
- *(translations)* Added missing keys in translation files
- *(release)* Added fallback to vendored deps
- *(release)* Removed draft status on flathub pr
- *(release)* Increase timeout for vendored dependencies
- *(release)* Changed lockfile update to tree command
- *(release)* V0.12.0
## [0.11.0] - 2026-04-05

### 🚀 Features

- *(browser)* Added librewolf
- *(browser)* Added firefox-esr
- *(web-app-view)* Improve form navigation with keyboard

### 💼 Other

- Revert "refactor(desktop-file): use space instead of '=' for conditional replacement"

This reverts commit 03c26db58d6ef51713c43bce953868b960a9bb23.

### 🚜 Refactor

- *(desktop-file)* Use space instead of '=' for conditional replacement
- *(desktop-file)* Added exception for firefox-esr to use space instead of '=' for conditional replacements
- *(web-app-view)* More improvements for form navigation

### ⚙️ Miscellaneous Tasks

- *(browser)* Add binary names for librewolf
- *(dev)* Some dev optimisations
- *(release)* V0.11.0
