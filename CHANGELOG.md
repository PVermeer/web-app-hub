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
## [0.9.0] - 2026-03-03

### 🚀 Features

- *(browsers)* Add more alternative system binary names

### 🚜 Refactor

- *(browsers)* Try to find the right system browser instead of listing multiple
- *(web-app-view)* Don't disable isolation if browser is not found

### ⚙️ Miscellaneous Tasks

- *(browsers)* Update yaml config name
- *(release)* V0.9.0
## [0.8.0] - 2026-02-28

### 🚀 Features

- *(browsers)* Allow array of system bin in browser yaml schema

### 🐛 Bug Fixes

- *(browsers)* Support brave on nixos

### 🚜 Refactor

- *(browsers)* Order icon names

### 📚 Documentation

- *(readme)* Add array info to browser config

### ⚙️ Miscellaneous Tasks

- *(release)* V0.8.0
## [0.7.0] - 2026-02-19

### 🚀 Features

- *(translations)* Add de translations (#29)

### 🐛 Bug Fixes

- *(web-app-view)* Isolation toggle now disables on non-supported browser
- *(browsers)* Custom browsers based on existing browsers now separate as intended

### 💼 Other

- Use main cargo file for app info

### 🚜 Refactor

- *(web-app-view)* Change to ui string for error toast
- *(web-app-view)* Make sure toasts display correct on init

### ⚙️ Miscellaneous Tasks

- *(release)* Remove initial release
- *(dev)* Add language support to dev-container
- *(web-app-view)* Add change check after init
- *(error)* Commit error.rs thats is used for to_string_ui
- *(dev)* Add seahorse flatpak
- *(web-apps)* Add more debug logs
- *(dev)* Add test files for custom browsers with shared profile
- *(browsers)* Sort browsers by name
- *(dev)* Add task for build new release flatpak
- *(release)* V0.7.0
