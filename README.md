# Web App Hub

A modern Web App Manager built with Rust, GTK, and the Adwaita design language. Web App Hub enables seamless management of web applications, each with its own icon and isolated browser profile.

[![Flathub](https://flathub.org/api/badge)](https://flathub.org/apps/org.pvermeer.WebAppHub)

## Features

- Seamless browser switching between all installed browsers
- Leverages your existing browser installations
- Profile isolation for enhanced privacy and organization
- Dedicated icons and dock indicators for each web application
- Extensible configuration system via YAML and desktop files

<img src="assets/screenshots/1-Web-App.png">

## Custom Browser Configuration

### Configuration Location

Browser configurations are located at:

```
~/.var/app/org.pvermeer.WebAppHub/config/web-app-hub
```

Default browser configurations are read-only and reset on application startup. To add custom browsers, create new configuration files in the appropriate directories. Example configurations are available in `assets/config`.

### Browser Config File

Create a `.yml` file in the `browsers` directory:

```yaml
name: Chromium
flatpak: org.chromium.Chromium # Optional: Flatpak app ID
system_bin: chromium-browser # Optional: System binary path
can_isolate: true # Supports profile isolation
can_start_maximized: true # Supports maximized launch
desktop_file_name_prefix: org.chromium.Chromium.chromium
base: chromium # Base browser type: chromium or firefox
issues: # Optional: Known limitations
  en: # Language code, see translations below
    - Does not remember window size and position
```

### Desktop File

Create a matching `.desktop` file in the `desktop-files` directory:

```desktop
[Desktop Entry]
Version=1.0
Type=Application
Terminal=false
Name=%{name}
Exec=%{command} --no-first-run --app="%{url}" --class=chrome-%{domain_path}-Default --name=chrome-%{domain_path}-Default %{is_isolated ? --user-data-dir} %{is_maximized ? --start-maximized}
X-MultipleArgs=false
Icon=%{icon}
StartupWMClass=chrome-%{domain_path}-Default
```

### Template Variables

The desktop file supports variable substitution using the `%{variable}` syntax.

#### Standard Variables

| Variable         | Description                                       |
| ---------------- | ------------------------------------------------- |
| `%{command}`     | Browser launch command (Flatpak or system binary) |
| `%{name}`        | Web application name                              |
| `%{url}`         | Complete application URL                          |
| `%{domain}`      | Domain portion of the URL                         |
| `%{domain_path}` | Sanitized domain and path combination             |
| `%{icon}`        | Path to the application icon                      |
| `%{app_id}`      | Generated application identifier                  |

#### Conditional Variables

Conditional variables use the syntax `%{condition ? value}` and are only included when the condition is met.

| Conditional                | Description                                                          |
| -------------------------- | -------------------------------------------------------------------- |
| `%{is_isolated ? --flag}`  | Expands to `--flag=<profile-path>` when profile isolation is enabled |
| `%{is_maximized ? --flag}` | Expands to `--flag` when start maximized is enabled                  |

### Profile Extras

The `profiles` directory can contain browser-specific subdirectories with additional files to be copied into isolated browser profiles.

**Directory structure:**

```
profiles/
├── firefox/          # Applied to all Firefox-based browsers
├── chromium/         # Applied to all Chromium-based browsers
└── brave/            # Applied only to Brave browser
```

**Important:** Base type (firefox/chromium) is only used if there is no specific browser configuration folder. Browser-specific folders take precedence over base type folders.

## How to Contribute

Contributions are welcome!

- Open a Pull Request (PR) against the main branch and explain your changes.

### Bug Reports & Feature Requests

Found an issue or have something you’d like added?

- Use the GitHub Issues section to report bugs or request features.
- Provide as much detail as possible (steps to reproduce, screenshots, etc.).

### Documentation

Good documentation is just as important as code!
You can contribute by improving:

- explanations and examples in this README
- translation files for supported languages

### Translations

Want to help translate the app?

- Add or update translations under the `workspaces/app/translations` folder.
- Add or update browser issue translations under the `assets/config/browsers` folder.
- Follow the existing translation file structure.
- Include your language code (e.g., `nl.yml` for Dutch or `nl_BE` to be more specific).
- `en.yml` Is the default translation file.
- You may use AI to translate the default `en.yml` file to your language. Please check and correct the translations.

Translation file names follow the list from `$ locale --all-locales` (e.g., `en_GB`). The first part (language code) is used as a fallback if the second part (country code) doesn't exist. `en.yml` Already exists, thus adding `en_GB.yml` will add British translation but `en_US` users will still get `en.yml` translations. This works the same for the translation key in the browser config files.

Don't forget to add your name to the credits in `workspaces/app/credits`.

### Style Guidelines

To keep contributions consistent:

- Follow existing code formatting (cargo fmt for Rust).
- Keep commit messages concise and descriptive.
- Test your changes before submitting (build and runtime).

## Building from Source

```sh
cargo build
```

**Flatpak:**

```sh
./flatpak/build.sh
```

## License

This project is licensed under the GPL-3.0 License. See the [LICENSE](LICENSE) file for details.
