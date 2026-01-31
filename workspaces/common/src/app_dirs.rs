use crate::{
    config::{self},
    utils::OnceLockExt,
};
use anyhow::{Context, Result};
use gtk::glib;
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};
use tracing::debug;

#[derive(Default)]
pub struct AppDirs {
    pub home: PathBuf,
    pub data: PathBuf,
    pub config: PathBuf,
    pub system_data: Vec<PathBuf>,
    pub user_data: PathBuf,
    pub user_config: PathBuf,
    pub system_icons: Vec<PathBuf>,
    pub applications: PathBuf,
    pub profiles: PathBuf,
    pub icons: PathBuf,
    pub browser_configs: PathBuf,
    pub browser_desktop_files: PathBuf,
    pub flatpak: PathBuf,
}
impl AppDirs {
    pub fn new() -> Result<Rc<Self>> {
        Rc::new(Self::default());

        let home = glib::home_dir();
        let user_data = glib::user_data_dir();
        let app_data = user_data.join(config::APP_NAME_HYPHEN.get_value());
        let user_config = glib::user_config_dir();
        let app_config = user_config.join(config::APP_NAME_HYPHEN.get_value());
        let system_data = glib::system_data_dirs();

        let system_icons = Self::build_system_icon_paths(&system_data);
        let applications = Self::build_applications_path(&user_data)?;
        let profiles = Self::build_profiles_path(&app_data)?;
        let icons = Self::build_icons_path(&app_data)?;
        let browser_configs = Self::build_browser_configs_path(&app_config)?;
        let browser_desktop_files = Self::build_browser_desktop_files_path(&app_config)?;
        let flatpak = Self::build_flatpak_path(&home);

        Ok(Rc::new(Self {
            home,
            data: app_data,
            config: app_config,
            system_data,
            user_data,
            user_config,

            system_icons,
            applications,
            profiles,
            icons,
            browser_configs,
            browser_desktop_files,
            flatpak,
        }))
    }

    fn build_system_icon_paths(system_data: &[PathBuf]) -> Vec<PathBuf> {
        let icons_dir_name = "icons";
        system_data
            .iter()
            .map(|path| path.join(icons_dir_name))
            .filter(|path| path.is_dir())
            .collect()
    }

    fn build_applications_path(user_data: &Path) -> Result<PathBuf> {
        let user_applications_path = user_data.join("applications");

        debug!(
            "Using system applications path: {}",
            user_applications_path.display()
        );

        if !user_applications_path.is_dir() {
            fs::create_dir_all(&user_applications_path).context(format!(
                "Could not create user applications dir: {}",
                user_applications_path.display()
            ))?;
        }

        Ok(user_applications_path)
    }

    fn build_profiles_path(app_data: &Path) -> Result<PathBuf> {
        let profiles_dir_name = "profiles";
        let profiles_path = app_data.join(profiles_dir_name);

        debug!("Using profile path: {}", profiles_path.display());

        if !profiles_path.is_dir() {
            fs::create_dir_all(&profiles_path).context(format!(
                "Could not create profiles dir: {}",
                profiles_path.display()
            ))?;
        }

        Ok(profiles_path)
    }

    fn build_icons_path(app_data: &Path) -> Result<PathBuf> {
        let icons_dir_name = "icons";
        let icons_path = app_data.join(icons_dir_name);

        debug!("Using icons path: {}", icons_path.display());

        if !icons_path.is_dir() {
            fs::create_dir_all(&icons_path).context(format!(
                "Could not create icons dir: {}",
                icons_path.display()
            ))?;
        }

        Ok(icons_path)
    }

    fn build_browser_configs_path(app_config: &Path) -> Result<PathBuf> {
        let browsers_dir_name = "browsers";
        let browser_configs_path = app_config.join(browsers_dir_name);

        debug!("Using browsers path: {}", browser_configs_path.display());

        if !browser_configs_path.is_dir() {
            fs::create_dir_all(&browser_configs_path).context(format!(
                "Could not create browsers dir: {}",
                browser_configs_path.display()
            ))?;
        }

        Ok(browser_configs_path)
    }

    fn build_browser_desktop_files_path(app_config: &Path) -> Result<PathBuf> {
        let browsers_desktop_files_dir_name = "desktop-files";
        let browser_desktop_files_path = app_config.join(browsers_desktop_files_dir_name);

        debug!(
            "Using browser desktop-files path: {}",
            browser_desktop_files_path.display()
        );

        if !browser_desktop_files_path.is_dir() {
            fs::create_dir_all(&browser_desktop_files_path).context(format!(
                "Could not create browser desktop-files dir: {}",
                browser_desktop_files_path.display()
            ))?;
        }

        Ok(browser_desktop_files_path)
    }

    fn build_flatpak_path(home: &Path) -> PathBuf {
        let flatpak_path = home.join(".var").join("app");

        debug!("Using flatpak path: {}", flatpak_path.display());

        flatpak_path
    }
}
