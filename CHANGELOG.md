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
## [0.10.1] - 2026-03-27

### 🐛 Bug Fixes

- *(web-app-view)* Hide discard button on validation toast for new app

### ⚙️ Miscellaneous Tasks

- *(release)* V0.10.1
## [0.10.0] - 2026-03-27

### 🚀 Features

- *(web-app-view)* Added validation indicators for missing or not applied fields
- *(optional-settings)* Added validation indicators for missing or not applied fields
- *(web-app-view)* Added warning toasts on exit
- *(web-app-view)* Added validation on save button
- *(translations)* Update es translations (#35)

### 🐛 Bug Fixes

- *(desktop-file)* A valid url no longer gets rewritten

### 💼 Other

- *(deps)* Bump quinn-proto in the cargo group across 1 directory (#31)
- *(deps)* Bump rustls-webpki in the cargo group across 1 directory (#38)

### 🚜 Refactor

- *(desktop-file)* Move path checks to getters

### 🎨 Styling

- *(translations)* Format

### ⚙️ Miscellaneous Tasks

- *(desktop-file)* Remove legacy icon name lookup
- *(dev)* Move clippy config to source
- *(web-app-view)* Add error:  to error toasts
- *(optional-settings)* Changed is_applied_all to is_dirty
- *(web-app-view)* Show validation error instead of actual error on changes
- *(optional-settings)* Run apply check on apply
- *(web-app-view)* Changed validation error message
- *(assets)* Update screenshots
- *(translations)* Add browser install type translations
- *(release)* V0.10.0
## [0.9.4] - 2026-03-06

### 🐛 Bug Fixes

- *(web-app-view)* Reset button disabled on new web app

### ⚙️ Miscellaneous Tasks

- *(release)* V0.9.4
