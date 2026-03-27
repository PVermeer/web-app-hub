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
## [0.9.4] - 2026-03-06

### 🐛 Bug Fixes

- *(web-app-view)* Reset button disabled on new web app

### ⚙️ Miscellaneous Tasks

- *(release)* V0.9.4
## [0.9.3] - 2026-03-06

### 🐛 Bug Fixes

- *(web-app-view)* Reset button now resets after save

### 📚 Documentation

- *(readme)* Update readme for profile_setup_keybind key in browser config

### ⚙️ Miscellaneous Tasks

- *(release)* V0.9.3
## [0.9.2] - 2026-03-05

### 🐛 Bug Fixes

- *(info)* Zen browser setup keybind shows again
- *(browsers)* Correct Zen browser config keybind

### 🚜 Refactor

- *(browsers)* Changed all_browsers to installed browsers
- *(browsers)* Changed get method names to get_browser
- *(browsers)* Update get_index to check for installation
- *(browsers)* Remove no-browser from installed
- *(browsers)* Match flatpak_id with installation

### ⚙️ Miscellaneous Tasks

- *(browsers)* Add all_browsers
- *(release)* V0.9.2
## [0.9.1] - 2026-03-05

### 🐛 Bug Fixes

- *(web-apps)* Load web apps that failed validation with warnings

### 💼 Other

- Revert "chore(release): commit including untracked"

This reverts commit 185ea8422bb97e2f626720fe5b610c43cd21adf5.

### 🚜 Refactor

- *(info)* Replace hardcoded browsers keybinds
- *(utils)* Add run command sync with env

### ⚙️ Miscellaneous Tasks

- *(release)* Load env from file
- *(release)* Commit including untracked
- *(ci)* Remove apt package cache
- *(ci)* Fix dep install
- *(release)* V0.9.1
