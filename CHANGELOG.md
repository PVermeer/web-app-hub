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
## [0.2.2] - 2026-01-10

### 🐛 Bug Fixes

- *(desktop-file)* Also try to create profile dir when copying profile config
- *(browsers)* Remove unneeded flatpak install type

### ⚙️ Miscellaneous Tasks

- *(release)* V0.2.2
## [0.2.1] - 2026-01-08

### 🐛 Bug Fixes

- *(icon-picker)* Custom icon updates the icon list again

### ⚙️ Miscellaneous Tasks

- *(release)* V0.2.1
## [0.2.0] - 2026-01-07

### 🚀 Features

- *(icon-picker)* When using a path in url, base url is also checked for icons

### 🚜 Refactor

- *(icon-fetcher)* Move icon fetching to own module + optimisations

### ⚙️ Miscellaneous Tasks

- *(desktop_file)* Better error message
- *(release)* Remove 'v' before version in metainfo
- *(release)* V0.2.0
## [0.1.2] - 2026-01-05

### 🐛 Bug Fixes

- *(browsers)* Chromium browsers now match the correct desktop app when using a path in the url

### 💼 Other

- *(release)* Update submodules

### ⚙️ Miscellaneous Tasks

- *(firefox)* Typo in config
- *(release)* Make sure screenshots order is preserved
- *(release)* Added --dry-run option
- *(release)* V0.1.2
