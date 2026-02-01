## [0.4.1] - 2026-02-01

### 🐛 Bug Fixes

- *(desktop-file)* Web apps update again on app update
## [0.4.0] - 2026-02-01

### 🚀 Features

- *(icon-picker)* Allow more image types
- *(desktop-file)* Set a default category
- *(desktop-file)* Add description
- *(web-app-view)* Optional settings for desktops with an app menu

### 🐛 Bug Fixes

- *(web-app-view)* Reset button is now disabled after saving a new web app
- *(web-apps)* App list is now sorted by name
- *(web-app-view)* Make sure "No browser" is selected when browser is missing
- *(firefox)* More reliable popups on firefox profile

### 🚜 Refactor

- *(desktop-file)* Keys enum to Key
- *(desktop-file)* Move deps to own files
- Removed all unwraps + more optimizations
- *(app-dirs)* Update dir names
- *(web-app-view)* Optional settings now save on apply

### ⚙️ Miscellaneous Tasks

- *(release)* Fix for last_released_version
- Format
- Update screenshots
- *(release)* V0.4.0
## [0.3.1] - 2026-01-26

### 🐛 Bug Fixes

- *(release)* Corrected last released version
- *(release)* Increment patch version for dry-run
- Use Adwaita icon theme on KDE

### 📚 Documentation

- *(readme)* Update README.md (#14)
- *(description)* Updated description text from #14

### ⚙️ Miscellaneous Tasks

- *(release)* V0.3.1
## [0.3.0] - 2026-01-22

### 🚀 Features

- Show update status + add release notes to about
- *(desktop-file)* Allow local ip as domain

### 🐛 Bug Fixes

- *(icon-picker)* Previous custom icon now shows when online fetch fails
- *(icon-picker)* Currently used icon is now also loaded
- *(web-app-view)* Url validator now also validates local ips

### 🚜 Refactor

- *(desktop-file)* Move validation to url package

### 📚 Documentation

- *(readme)* Added flathub link

### ⚙️ Miscellaneous Tasks

- *(screenshots)* Reorder
- Added copywrite
- *(release)* V0.3.0
## [0.2.2] - 2026-01-10

### 🐛 Bug Fixes

- *(desktop-file)* Also try to create profile dir when copying profile config
- *(browsers)* Remove unneeded flatpak install type

### ⚙️ Miscellaneous Tasks

- *(release)* V0.2.2
