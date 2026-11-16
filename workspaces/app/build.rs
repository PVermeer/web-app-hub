#![allow(clippy::unwrap_used)] // TODO change these to expects
#![allow(clippy::expect_used)]

use anyhow::{Context, Result};
use common::{
    app_dirs::AppDirs,
    config::{self},
    utils::{self, OnceLockExt},
};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::{Path, PathBuf},
};

#[allow(unused_macros)]
macro_rules! debug {
    ($val:expr $(,)?) => {{
        let value = &$val;
        let output = format!("{:#?}", value);

        for line in output.lines() {
            println!("cargo:warning=Debug: {}", line);
        }

        value
    }};
}

fn main() -> Result<()> {
    println!("cargo:warning=Debug: App build script is running!");
    config::init();
    let app_dirs = AppDirs::new()?;
    let is_in_dev_container =
        option_env!("RUN_IN_VSCODE_DEVCONTAINER").is_some_and(|value| value == "true");

    if cfg!(debug_assertions) && is_in_dev_container {
        create_config_symlinks(&app_dirs);
        create_data_symlinks(&app_dirs);
        create_cache_symlinks(&app_dirs);
        copy_dev_web_apps(&app_dirs);
        copy_dev_custom_browsers(&app_dirs);
    }

    install_app_desktop_file(&app_dirs)?;
    install_app_icon(&app_dirs)?;

    check_translations();

    Ok(())
}

fn create_config_symlinks(app_dirs: &AppDirs) {
    let config_path = dev_config_path();
    let _ = utils::files::create_symlink(&config_path, &app_dirs.app_config);
}

fn create_data_symlinks(app_dirs: &AppDirs) {
    let data_path = dev_data_path();

    let _ = utils::files::create_symlink(&data_path, &app_dirs.app_data);
    let _ =
        utils::files::create_symlink(&data_path.join("applications"), &app_dirs.user_applications);
}

fn create_cache_symlinks(app_dirs: &AppDirs) {
    let cache_path = dev_cache_path();
    let _ = utils::files::create_symlink(&cache_path, &app_dirs.app_cache);
}

fn copy_dev_web_apps(app_dirs: &AppDirs) {
    let dev_desktop_files = dev_assets_path().join("desktop-files");
    let dev_icons = dev_assets_path().join("icons");
    let user_applications_dir = &app_dirs.user_applications;
    let user_icons_dir = &app_dirs.app_data_icons;

    for desktop_file in &utils::files::get_entries_in_dir(&dev_desktop_files).unwrap() {
        let id = desktop_file
            .file_name()
            .to_string_lossy()
            .split('-')
            .next_back()
            .unwrap()
            .to_string();

        let mut exists = false;
        for file in &utils::files::get_entries_in_dir(user_applications_dir).unwrap() {
            if file.file_name().to_string_lossy().ends_with(&id) {
                exists = true;
            }
        }
        if exists {
            continue;
        }

        fs::copy(
            desktop_file.path(),
            user_applications_dir.join(desktop_file.file_name()),
        )
        .unwrap();
    }

    for icon in &utils::files::get_entries_in_dir(&dev_icons).unwrap() {
        let mut exists = false;
        for file in &utils::files::get_entries_in_dir(user_icons_dir).unwrap() {
            if file.file_name() == icon.file_name() {
                exists = true;
            }
        }
        if exists {
            continue;
        }

        fs::copy(icon.path(), user_icons_dir.join(icon.file_name())).unwrap();
    }
}

fn copy_dev_custom_browsers(app_dirs: &AppDirs) {
    let dev_desktop_files = dev_assets_path().join("custom-browsers");
    let app_config_dir = &app_dirs.app_config;

    let copy_options = fs_extra::dir::CopyOptions {
        overwrite: true,
        content_only: true,
        ..fs_extra::dir::CopyOptions::default()
    };
    fs_extra::dir::copy(dev_desktop_files, app_config_dir, &copy_options).unwrap();

    let firefox_shared_profile_path = &app_dirs
        .user_flatpak
        .join("org.mozilla.firefox")
        .join("data")
        .join(config::APP_NAME_HYPHEN.get_value())
        .join("profiles")
        .join("shared-profile");

    fs::create_dir_all(firefox_shared_profile_path).unwrap();
}

fn install_app_desktop_file(app_dirs: &AppDirs) -> Result<()> {
    let file_name = desktop_file_name();
    let desktop_file = assets_path().join("desktop").join(&file_name);
    let save_file = app_dirs.user_applications.join(file_name);

    fs::copy(desktop_file, save_file).context("Desktop file copy failed")?;
    Ok(())
}

fn install_app_icon(app_dirs: &AppDirs) -> Result<()> {
    let file_name = icon_file_name();
    let icon_file = assets_path().join("desktop").join(&file_name);
    let save_dir = app_dirs
        .user_data
        .join("icons")
        .join("hicolor")
        .join("256x256")
        .join("apps");
    if !save_dir.is_dir() {
        fs::create_dir_all(&save_dir).context("Failed to create icon dir")?;
    }

    let save_file = save_dir.join(file_name);

    fs::copy(icon_file, save_file).context("Icon copy failed")?;
    Ok(())
}

fn project_path() -> PathBuf {
    Path::new("").join("..").join("..").canonicalize().unwrap()
}

fn assets_path() -> PathBuf {
    project_path().join("assets")
}

fn dev_config_path() -> PathBuf {
    project_path().join("dev-config")
}

fn dev_data_path() -> PathBuf {
    project_path().join("dev-data")
}

fn dev_cache_path() -> PathBuf {
    project_path().join("dev-cache")
}

fn dev_assets_path() -> PathBuf {
    project_path().join("dev-assets")
}

fn desktop_file_name() -> String {
    let app_id = config::APP_ID.get_value();
    let extension = "desktop";
    let file_name = format!("{app_id}.{extension}");

    file_name
}

fn icon_file_name() -> String {
    let app_id = config::APP_ID.get_value();
    let extension = "png";
    let file_name = format!("{app_id}.{extension}");

    file_name
}

fn check_translations() {
    #[derive(Serialize)]
    struct TranslationCheckYaml {
        keys_not_translated: HashMap<String, HashSet<String>>,
        extra_keys_in_translations: HashMap<String, HashMap<String, String>>,
    }

    let mut used_in_source = HashMap::new();
    let source_path = Path::new(".")
        .canonicalize()
        .expect("Failed to create source path for i18n");

    let config =
        rust_i18n_support::I18nConfig::load(&source_path).expect("Failed to load i18n config");

    rust_i18n_extract::iter::iter_crate(&source_path.to_string_lossy(), |path, source| {
        rust_i18n_extract::extractor::extract(&mut used_in_source, path, source, config.clone())
    })
    .expect("Failed to extract source files for i18n");

    let mut messages: Vec<_> = used_in_source.iter().collect();
    messages.sort_by_key(|(_k, m)| m.index);

    let translations_path = Path::new(&source_path).join(&config.load_path);
    let output_file_name = "TODO.yml";
    let output_file = translations_path.join(output_file_name);
    let _ = fs::remove_file(&output_file);

    let mut missing_keys: HashMap<String, HashSet<String>> = HashMap::new();
    let mut extra_keys: HashMap<String, HashMap<String, String>> = HashMap::new();

    let ignore_file = |file_name: &str| file_name.ends_with(output_file_name);
    let translations =
        rust_i18n_support::load_locales(&translations_path.to_string_lossy(), ignore_file);

    for key in used_in_source.keys() {
        for (locale, translations) in &translations {
            if !translations.contains_key(key) {
                missing_keys
                    .entry(locale.clone())
                    .or_default()
                    .insert(key.clone());
            }
        }
    }

    for (locale, translations) in translations {
        for (key, value) in translations {
            if key.starts_with('_') {
                continue;
            }

            if !used_in_source.contains_key(&key) {
                extra_keys
                    .entry(locale.clone())
                    .or_default()
                    .insert(key, value);
            }
        }
    }

    let translation_todo_yml = serde_yaml::to_string(&TranslationCheckYaml {
        keys_not_translated: missing_keys,
        extra_keys_in_translations: extra_keys,
    })
    .expect("Failed to parse result to yaml in i18n");

    fs::write(&output_file, &translation_todo_yml).expect("Failed to write yml to fs for i18n");
}
