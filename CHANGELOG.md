## [1.2.0] - 2026-07-30

### 🚀 Features

- *(browser)* Added brave-origin browser

### 🐛 Bug Fixes

- *(browser)* Added aur brave-origin-bin to brave-origin

### 💼 Other

- *(deps)* Bump quinn-proto in the cargo group across 1 directory (#64)
- Revert "chore(release): v1.2.0"

This reverts commit f80487b1e31314c17473fac0d43c0294f014236f.

### ⚙️ Miscellaneous Tasks

- *(release)* V1.2.0
## [1.1.2] - 2026-07-27

### 🐛 Bug Fixes

- *(icon-picker)* Icons are now first ordered by transparency

### ⚙️ Miscellaneous Tasks

- *(release)* V1.1.2
## [1.1.1] - 2026-07-18

### 🐛 Bug Fixes

- *(icon-picker)* Don't stop fetching on html fetch error
- *(icon-picker)* Try to remove duplicate icons

### 💼 Other

- Changed environment check to optional

### 🚜 Refactor

- *(desktop-file)* Make updated clippy happy
- *(browsers)* Changed gtk IconTheme to trait implementation

### ⚡ Performance

- *(icon-picker)* Load html urls in parallel

### 🧪 Testing

- *(desktop-file)* Added brave test case
- *(desktop-file)* Updated to chromium / firefox tests

### ⚙️ Miscellaneous Tasks

- *(dev)* Fix dns resolve lockup in dev-container
- *(dev)* Fixup dev web apps
- *(dev)* Added some more dev web apps
- *(dev)* Only do dev env things in dev container
- *(release)* V1.1.1
## [1.1.0] - 2026-07-06

### 🚀 Features

- *(icon-picker)* Added google favicon api lookup

### 💼 Other

- *(app)* Only do dev things on debug builds

### ⚙️ Miscellaneous Tasks

- *(release)* V1.1.0
## [1.0.1] - 2026-06-05

### 🐛 Bug Fixes

- *(icon-picker)* Improved icon fetcher (#58)
- *(icon-picker)* Fix for mobile smallscreen (#59)

### ⚙️ Miscellaneous Tasks

- *(ci)* Added timout on release action
- *(release)* V1.0.1
