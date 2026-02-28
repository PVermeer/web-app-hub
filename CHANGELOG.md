## [0.8.0] - 2026-02-28

### 🚀 Features

- *(browsers)* Allow array of system bin in browser yaml schema

### 🐛 Bug Fixes

- *(browsers)* Support brave on nixos

### 🚜 Refactor

- *(browsers)* Order icon names

### 📚 Documentation

- *(readme)* Add array info to browser config
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
## [0.6.0] - 2026-02-15

### 🚀 Features

- *(translations)* Create Italian translation file it.yml (#26)
- *(translations)* Add es translations (#25)

### 🐛 Bug Fixes

- *(about)* Documenters now show in credits again

### 🚜 Refactor

- *(build)* Add path check to flatpak build script

### ⚙️ Miscellaneous Tasks

- *(release)* V0.6.0
## [0.5.0] - 2026-02-08

### 🚀 Features

- *(window)* App now remembers window size
- *(window)* App now remembers maximized window
- Add language support
- *(translations)* Added dutch
- *(about)* Added credits section

### 🐛 Bug Fixes

- *(web-app-view)* Set category default to Network / Internet

### 💼 Other

- *(deps)* Bump bytes in the cargo group across 1 directory (#22)
- *(deps)* Bump git2 in the cargo group across 1 directory (#23)
- *(deps)* Bump time in the cargo group across 1 directory (#24)

### 🚜 Refactor

- *(desktop-file)* Static method for is_owned check
- *(desktop-file)* Remove expect in method
- *(desktop-file)* Revert: set_defaults on construction
- *(desktop-file)* When loading web apps, skip non desktop files
- *(about)* About to own module, added translation support for app menu and about
- *(about)* Revert translation of about

### 📚 Documentation

- *(readme)* Added contributing section + renamed translation dir
- *(readme)* Fix typo

### 🎨 Styling

- *(browsers)* Allow longer method for expand content

### ⚙️ Miscellaneous Tasks

- Print version on info channel
- *(translation)* Actually translate issues to dutch
- *(translations)* Add more_info + fix some issues
- *(about)* Add language to credits
- *(translations)* Add web app category
- *(release)* V0.5.0
## [0.4.1] - 2026-02-01

### 🐛 Bug Fixes

- *(desktop-file)* Web apps update again on app update

### ⚙️ Miscellaneous Tasks

- *(release)* V0.4.1
