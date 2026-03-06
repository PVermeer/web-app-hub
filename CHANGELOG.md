## [0.9.4] - 2026-03-06

### 🐛 Bug Fixes

- *(web-app-view)* Reset button disabled on new web app
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
## [0.9.0] - 2026-03-03

### 🚀 Features

- *(browsers)* Add more alternative system binary names

### 🚜 Refactor

- *(browsers)* Try to find the right system browser instead of listing multiple
- *(web-app-view)* Don't disable isolation if browser is not found

### ⚙️ Miscellaneous Tasks

- *(browsers)* Update yaml config name
- *(release)* V0.9.0
