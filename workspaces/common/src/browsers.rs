use crate::utils::{self, OnceLockExt};
use crate::{
    app_dirs::AppDirs,
    config::{self},
};
use anyhow::{Context, Result, bail};
use freedesktop_desktop_entry::DesktopEntry;
use gtk::{IconTheme, Image};
use serde::Deserialize;
use std::{cell::OnceCell, collections::HashMap, fs, path::Path, rc::Rc};
use std::{fmt::Write as _, path::PathBuf};
use tracing::{debug, error, info};

#[derive(PartialEq)]
pub enum Installation {
    Flatpak(String),
    System(String),
    None,
}

#[derive(PartialEq)]
pub enum Base {
    Chromium,
    Firefox,
    None,
}
impl Base {
    fn from_string(string: &str) -> Self {
        match string {
            "chromium" => Self::Chromium,
            "firefox" => Self::Firefox,
            _ => Self::None,
        }
    }

    pub fn get_profile_setup_keybind(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Chromium => Some("Ctrl+T".to_string()),
            Self::Firefox => Some("Alt".to_string()),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum StringOrVec {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct BrowserYaml {
    name: String,
    flatpak: Option<String>,
    system_bin: Option<StringOrVec>,
    #[serde(default)]
    can_isolate: bool,
    #[serde(default)]
    can_start_maximized: bool,
    desktop_file_name_prefix: String,
    base: String,
    profile_setup_keybind: Option<String>,
    #[serde(default)]
    issues: HashMap<String, Vec<String>>,
}

struct BrowserConfig {
    config_yaml: BrowserYaml,
    config_name: String,
    file_name: String,
    desktop_file: DesktopEntry,
}

pub struct Browser {
    pub id: String,
    pub name: String,
    pub installation: Installation,
    pub can_isolate: bool,
    pub can_start_maximized: bool,
    pub flatpak_id: Option<String>,
    pub executable: Option<String>,
    pub desktop_file: DesktopEntry,
    pub desktop_file_name_prefix: String,
    pub base: Base,
    pub issues: HashMap<String, Vec<String>>,
    pub config_name: String,
    pub profile_setup_keybind: Option<String>,
    configs: Rc<BrowserConfigs>,
    config: Option<Rc<BrowserConfig>>,
    icon_theme: Rc<IconTheme>,
    icon_names: Vec<String>,
    app_dirs: Rc<AppDirs>,
}
impl Browser {
    const FALLBACK_IMAGE: &str = "web-browser-symbolic";

    fn new(
        browser_config: &Rc<BrowserConfig>,
        installation: Installation,
        browser_configs: &Rc<BrowserConfigs>,
        icon_theme: &Rc<IconTheme>,
        app_dirs: &Rc<AppDirs>,
    ) -> Self {
        let icon_names = Self::get_icon_names_from_config(browser_config);
        let name = browser_config.config_yaml.name.clone();
        let can_isolate = browser_config.config_yaml.can_isolate;
        let can_start_maximized = browser_config.config_yaml.can_start_maximized;
        let flatpak_id = browser_config.config_yaml.flatpak.clone();
        let executable = if let Installation::System(system_bin) = &installation {
            Some(system_bin.clone())
        } else {
            None
        };
        let desktop_file = browser_config.desktop_file.clone();
        let desktop_file_name_prefix = browser_config.config_yaml.desktop_file_name_prefix.clone();
        let config_name = browser_config.config_name.clone();
        let base = Base::from_string(&browser_config.config_yaml.base);
        let issues = browser_config.config_yaml.issues.clone();
        let id = Self::create_id(&installation, &name);
        let profile_setup_keybind = Self::get_profile_setup_keybind(browser_config, &base);

        Self {
            id,
            name,
            installation,
            can_isolate,
            can_start_maximized,
            flatpak_id,
            executable,
            desktop_file,
            desktop_file_name_prefix,
            config_name,
            profile_setup_keybind,
            configs: browser_configs.clone(),
            config: Some(browser_config.clone()),
            icon_names,
            base,
            issues,
            icon_theme: icon_theme.clone(),
            app_dirs: app_dirs.clone(),
        }
    }

    pub fn is_flatpak(&self) -> bool {
        matches!(self.installation, Installation::Flatpak(_))
    }

    pub fn is_system(&self) -> bool {
        matches!(self.installation, Installation::System(_))
    }

    pub fn is_installed(&self) -> bool {
        !matches!(self.installation, Installation::None)
    }

    pub fn is_no_browser(&self) -> bool {
        self.name == BrowserConfigs::NO_BROWSER_NAME
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_name_with_installation(&self) -> String {
        let mut txt = String::new();
        let _ = write!(txt, "{}", self.name);

        match self.installation {
            Installation::Flatpak(_) => {
                let _ = write!(txt, " (Flatpak)");
            }
            Installation::System(_) => {
                let _ = write!(txt, " (System)");
            }
            Installation::None => {}
        }

        txt
    }

    pub fn get_run_command(&self) -> Result<String> {
        match &self.installation {
            Installation::Flatpak(id) => Ok(format!("flatpak run {id}")),
            Installation::System(executable) => Ok(executable.clone()),
            Installation::None => bail!("Browser is not installed"),
        }
    }

    pub fn get_icon(&self) -> Image {
        for icon in &self.icon_names {
            if !self.icon_theme.has_icon(icon) {
                continue;
            }
            let image = Image::from_icon_name(icon);
            if image.uses_fallback() {
                continue;
            }
            return image;
        }

        Image::from_icon_name(Self::FALLBACK_IMAGE)
    }

    pub fn get_profile_path(&self) -> Result<PathBuf> {
        if !self.can_isolate {
            bail!("Browser cannot isolate")
        }

        // Save in own app
        let system_browser_profile_path = || -> Result<PathBuf> {
            let path = self.app_dirs.app_data_profiles.join(self.get_install_id());
            Ok(path)
        };

        // Save in browser own location (for sandboxes)
        let flatpak_browser_profile_path = || -> Result<PathBuf> {
            let path = self
                .app_dirs
                .user_flatpak
                .join(&self.flatpak_id.clone().context("Not a flatpak browser?")?)
                .join("data")
                .join(config::APP_NAME_HYPHEN.get_value())
                .join("profiles");
            Ok(path)
        };

        let profile = match self.base {
            /*
               Firefox has a method to create profiles (-CreateProfile <name> and -P) but is poorly implemented.
               If firefox has never run it will set the created profile as default and
               never creates a default profile.
               Then there is --profile <path>, this works but will not create the path if it doesn't exists.
               So `--filesystem=~/.var/app:create` is needed to break in the sandbox to create the path if it doesn't exists.
               All a bit poorly implemented.

               Chromium based just created the provided profile path
            */
            Base::Chromium | Base::Firefox => match self.installation {
                Installation::Flatpak(_) => flatpak_browser_profile_path()?,
                Installation::System(_) => system_browser_profile_path()?,
                Installation::None => bail!("Browser is not installed"),
            },

            Base::None => {
                bail!("No base browser on 'Browser'")
            }
        };

        Ok(profile)
    }

    pub fn get_index(&self) -> Option<usize> {
        self.configs.get_index(self)
    }

    pub fn get_install_id(&self) -> String {
        Self::get_install_id_static(&self.installation)
    }

    pub fn get_all_ids(&self) -> Option<Vec<String>> {
        let mut ids = Vec::new();

        let system_bin_ids = self.get_all_executables().map(|bins| {
            bins.into_iter()
                .map(|bin| Self::create_id(&Installation::System(bin), &self.name))
                .collect::<Vec<_>>()
        });

        if let Some(mut system_bin_ids) = system_bin_ids {
            ids.append(&mut system_bin_ids);
        }

        if let Some(flatpak_id) = &self.flatpak_id {
            ids.push(Self::create_id(
                &Installation::Flatpak(flatpak_id.clone()),
                &self.name,
            ));
        }

        if ids.is_empty() {
            return None;
        }

        Some(ids)
    }

    fn get_all_executables(&self) -> Option<Vec<String>> {
        self.config.as_ref().and_then(|config| {
            config
                .config_yaml
                .system_bin
                .as_ref()
                .map(|system_bin| match system_bin {
                    StringOrVec::One(bin) => Vec::from([bin.clone()]),
                    StringOrVec::Many(bins) => bins.clone(),
                })
        })
    }

    fn get_install_id_static(installation: &Installation) -> String {
        match installation {
            Installation::Flatpak(id) => id.clone(),
            Installation::System(executable) => executable.clone(),
            Installation::None => "Not installed".to_string(),
        }
    }

    fn create_id(installation: &Installation, name: &str) -> String {
        let install_id = Self::get_install_id_static(installation);
        let name_sanitized = name.to_lowercase().replace(' ', "_");
        let id = format!("{install_id}_{name_sanitized}");

        id
    }

    fn get_icon_names_from_config(browser_config: &BrowserConfig) -> Vec<String> {
        let mut icon_names = Vec::new();

        if let Some(flatpak) = &browser_config.config_yaml.flatpak {
            icon_names.push(flatpak.trim().to_string());
        }

        if let Some(bins) = &browser_config.config_yaml.system_bin {
            match bins {
                StringOrVec::One(bin) => {
                    icon_names.push(bin.trim().to_string());
                }
                StringOrVec::Many(bins) => {
                    for bin in bins {
                        icon_names.push(bin.trim().to_string());
                    }
                }
            }
        }

        icon_names.push(browser_config.config_yaml.name.trim().to_string());
        utils::vec::dedup(&mut icon_names);

        icon_names
    }

    fn get_profile_setup_keybind(
        browser_config: &BrowserConfig,
        browser_base: &Base,
    ) -> Option<String> {
        if let Some(keybind) = &browser_config.config_yaml.profile_setup_keybind {
            if keybind.is_empty() {
                return None;
            }
            return Some(keybind.clone());
        }

        browser_base.get_profile_setup_keybind()
    }
}

pub struct BrowserConfigs {
    all_browsers: OnceCell<Vec<Rc<Browser>>>,
    installed_browsers: OnceCell<Vec<Rc<Browser>>>,
    uninstalled_browsers: OnceCell<Vec<Rc<Browser>>>,
    icon_theme: Rc<IconTheme>,
    app_dirs: Rc<AppDirs>,
}
impl BrowserConfigs {
    pub const NO_BROWSER_NAME: &str = "No browser";

    pub fn new(icon_theme: &Rc<IconTheme>, app_dirs: &Rc<AppDirs>) -> Rc<Self> {
        Rc::new(Self {
            all_browsers: OnceCell::new(),
            installed_browsers: OnceCell::new(),
            uninstalled_browsers: OnceCell::new(),
            icon_theme: icon_theme.clone(),
            app_dirs: app_dirs.clone(),
        })
    }

    pub fn init(self: &Rc<Self>) {
        self.set_browsers_from_files();
    }

    pub fn get_all_browsers(&self) -> &Vec<Rc<Browser>> {
        self.all_browsers.get().expect("Browsers are uninitialized")
    }

    pub fn get_installed_browsers(&self) -> &Vec<Rc<Browser>> {
        self.installed_browsers
            .get()
            .expect("Browsers are uninitialized")
    }

    pub fn get_flatpak_browsers(&self) -> Vec<Rc<Browser>> {
        let browsers = self.get_installed_browsers();
        browsers
            .iter()
            .filter(|browser| browser.is_flatpak())
            .cloned()
            .collect()
    }

    pub fn get_system_browsers(&self) -> Vec<Rc<Browser>> {
        let browsers = self.get_installed_browsers();
        browsers
            .iter()
            .filter(|browser| browser.is_system())
            .cloned()
            .collect()
    }

    pub fn get_uninstalled_browsers(&self) -> &Vec<Rc<Browser>> {
        self.uninstalled_browsers
            .get()
            .expect("Uninstalled browsers are uninitialized")
    }

    pub fn get_browser_by_id(&self, id: &str) -> Option<Rc<Browser>> {
        let browser = self
            .get_installed_browsers()
            .iter()
            .find(|browser| browser.id == id)
            .cloned();

        if browser.is_none() {
            info!("Browser is missing, trying to find it");

            let find_browser = self
                .get_system_browsers()
                .iter()
                .find(|browser| {
                    browser.get_all_ids().is_some_and(|browser_ids| {
                        browser_ids.iter().any(|browser_id| browser_id == id)
                    })
                })
                .cloned();

            if let Some(find_browser) = &find_browser {
                info!(find_browser.name, "Found browser");
            } else {
                info!("Browser not found");
            }

            return find_browser;
        }

        browser
    }

    pub fn get_browser_by_install_id(&self, install_id: &str) -> Option<Rc<Browser>> {
        self.get_installed_browsers()
            .iter()
            .find(|browser| browser.get_install_id() == install_id)
            .cloned()
    }

    pub fn get_browser_by_flatpak_id(&self, flatpak_id: &str) -> Option<Rc<Browser>> {
        self.get_flatpak_browsers()
            .iter()
            .find(|browser| browser.get_install_id() == flatpak_id)
            .cloned()
    }

    pub fn get_index(&self, browser: &Browser) -> Option<usize> {
        let browsers = if browser.is_installed() {
            self.get_installed_browsers()
        } else {
            self.get_uninstalled_browsers()
        };

        browsers
            .iter()
            .position(|browser_iter| browser_iter.id == browser.id)
    }

    pub fn add_icon_search_path(self: &Rc<Self>, path: &Path) {
        if !path.is_dir() {
            debug!("Not a valid icon path: {}", path.display());
            return;
        }

        debug!("Adding icon path to icon theme: {}", path.display());
        self.icon_theme.add_search_path(path);
    }

    pub fn build_no_browser(self: &Rc<Self>) -> Browser {
        Browser {
            id: String::default(),
            name: Self::NO_BROWSER_NAME.to_string(),
            installation: Installation::None,
            can_isolate: false,
            can_start_maximized: false,
            flatpak_id: None,
            executable: None,
            desktop_file: DesktopEntry::from_appid("No browser".to_string()),
            desktop_file_name_prefix: String::default(),
            config_name: String::default(),
            profile_setup_keybind: None,
            configs: self.clone(),
            config: None,
            icon_names: Vec::from(["dialog-warning-symbolic".to_string()]),
            base: Base::None,
            issues: HashMap::new(),
            icon_theme: self.icon_theme.clone(),
            app_dirs: self.app_dirs.clone(),
        }
    }

    fn set_browsers_from_files(self: &Rc<Self>) {
        let browser_configs = self.get_browsers_from_files();
        let mut all_browsers = Vec::new();
        let mut installed_browsers = Vec::new();
        let mut uninstalled_browsers = Vec::new();

        for browser_config in browser_configs {
            let mut is_installed = false;

            if let Some(flatpak) = &browser_config.config_yaml.flatpak {
                if Self::is_installed_flatpak(flatpak) {
                    info!(
                        "Found flatpak browser '{flatpak}' for config '{}'",
                        browser_config.file_name
                    );

                    let browser = Rc::new(Browser::new(
                        &browser_config,
                        Installation::Flatpak(flatpak.clone()),
                        self,
                        &self.icon_theme,
                        &self.app_dirs,
                    ));

                    if utils::env::is_flatpak_container()
                        && let Some(icon_search_path) = Self::get_icon_search_path_flatpak(flatpak)
                    {
                        self.add_icon_search_path(&icon_search_path);
                    }

                    installed_browsers.push(browser);
                    is_installed = true;
                } else {
                    debug!(
                        "Flatpak browser '{flatpak}' for '{}' is not installed",
                        browser_config.file_name
                    );
                }
            }

            if let Some(system_bins) = &browser_config.config_yaml.system_bin {
                let new_system_browser = |system_bin: &str| -> Option<Browser> {
                    if Self::is_installed_system(system_bin) {
                        info!(
                            "Found system browser '{system_bin}' for config '{}'",
                            browser_config.file_name
                        );

                        let browser = Browser::new(
                            &browser_config,
                            Installation::System(system_bin.to_string()),
                            self,
                            &self.icon_theme,
                            &self.app_dirs,
                        );

                        Some(browser)
                    } else {
                        debug!(
                            "System browser '{system_bin}' for '{}' is not installed",
                            browser_config.file_name
                        );
                        None
                    }
                };

                match system_bins {
                    StringOrVec::One(system_bin) => {
                        if let Some(browser) = new_system_browser(system_bin) {
                            installed_browsers.push(Rc::new(browser));
                            is_installed = true;
                        }
                    }
                    StringOrVec::Many(system_bins) => {
                        for system_bin in system_bins {
                            if let Some(browser) = new_system_browser(system_bin) {
                                installed_browsers.push(Rc::new(browser));
                                is_installed = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !is_installed {
                let browser = Rc::new(Browser::new(
                    &browser_config,
                    Installation::None,
                    self,
                    &self.icon_theme,
                    &self.app_dirs,
                ));
                uninstalled_browsers.push(browser);
            }
        }

        let _ = self.installed_browsers.set(installed_browsers.clone());
        let _ = self.uninstalled_browsers.set(uninstalled_browsers.clone());

        all_browsers.append(&mut installed_browsers);
        all_browsers.append(&mut uninstalled_browsers);
        let _ = self.all_browsers.set(all_browsers);
    }

    fn is_installed_flatpak(flatpak: &str) -> bool {
        let command = format!("flatpak info {flatpak}");
        let result = utils::command::run_command_sync(&command);

        match result {
            Err(error) => {
                error!("Could not run command '{command}'. Error: {error:?}");
                false
            }
            Ok(response) => response.success,
        }
    }

    fn is_installed_system(system_bin: &str) -> bool {
        let command = format!("which {system_bin}");
        let result = utils::command::run_command_sync(&command);

        match result {
            Err(error) => {
                error!("Could not run command '{command}'. Error: {error:?}");
                false
            }
            Ok(response) => response.success,
        }
    }

    fn get_icon_search_path_flatpak(flatpak: &str) -> Option<PathBuf> {
        if !utils::env::is_flatpak_container() {
            error!("Don't need to get icon search path when not in flatpak container");
            return None;
        }

        let command = format!("flatpak info --show-location {flatpak}");
        let result = utils::command::run_command_sync(&command);

        match result {
            Err(error) => {
                error!("Could not run command '{command}'. Error: {error:?}");
                None
            }
            Ok(response) => {
                if !response.success {
                    error!(
                        error = response.stderr,
                        "Could not get icon search path for: {flatpak}"
                    );
                    return None;
                }

                let path = Path::new(&response.stdout)
                    .join("export")
                    .join("share")
                    .join("icons");

                if !path.is_dir() {
                    error!("Invalid icon path for '{flatpak}': {}", path.display());
                    return None;
                }

                Some(path)
            }
        }
    }

    fn get_browsers_from_files(&self) -> Vec<Rc<BrowserConfig>> {
        debug!("Loading browsers config files");

        let mut browser_configs = Vec::new();
        let browser_config_files =
            utils::files::get_entries_in_dir(&self.app_dirs.app_config_browser_configs)
                .unwrap_or_default();

        for file in &browser_config_files {
            let file_name = file.file_name().to_string_lossy().to_string();
            let file_path = file.path();
            let Some(config_name) = file
                .path()
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
            else {
                debug!("Invalid file, failed to get file stem: '{file_name}'");
                continue;
            };

            let extension = file_path.extension().unwrap_or_default().to_string_lossy();
            debug!("Loading browser config: '{file_name}'");

            if extension != "yml" && extension != "yaml" {
                debug!("Not a yml file: '{file_name}'");
                continue;
            }

            let Ok(file_string) = fs::read_to_string(&file_path) else {
                error!("Failed to read to string: '{file_name}'");
                continue;
            };
            let browser: BrowserYaml = match serde_yaml::from_str(&file_string) {
                Ok(result) => result,
                Err(error) => {
                    error!("Failed to parse yml: '{file_name}'. Error: '{error:?}'");
                    continue;
                }
            };

            let desktop_file = match (|| -> Result<DesktopEntry> {
                let desktop_file_path = self
                    .app_dirs
                    .app_config_browser_desktop_files
                    .join(
                        file_path
                            .file_stem()
                            .context("Could not get the file stem")?,
                    )
                    .with_extension("desktop");

                debug!(
                    "Loading browser desktop file: '{}'",
                    desktop_file_path
                        .file_name()
                        .map_or("No file name???".to_string(), |file_name| file_name
                            .to_string_lossy()
                            .to_string())
                );

                let desktop_file = DesktopEntry::from_path(&desktop_file_path, None::<&[String]>)?;
                Ok(desktop_file)
            })() {
                Ok(result) => result,
                Err(error) => {
                    error!("Failed to parse .desktop file for: '{file_name}'. Error: '{error:?}'");
                    continue;
                }
            };

            let browser_config = BrowserConfig {
                config_yaml: browser,
                config_name,
                file_name,
                desktop_file,
            };
            browser_configs.push(Rc::new(browser_config));
        }

        browser_configs.sort_by(|a, b| a.config_yaml.name.cmp(&b.config_yaml.name));

        browser_configs
    }
}
