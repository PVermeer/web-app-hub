use common::{assets, config, utils::OnceLockExt};
use gtk::License;
use libadwaita::AboutDialog;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use tracing::error;

static CREDITS_DOCUMENTATION: &str = include_str!("../../../../credits/documentation.yml");
static CREDITS_TRANSLATIONS: &str = include_str!("../../../../credits/translations.yml");

#[derive(Serialize, Deserialize)]
pub struct CreditsYaml {
    name: String,
    language: Option<String>,
    link: Option<String>,
}

pub fn get_dialog() -> AboutDialog {
    let license = match config::LICENSE.get_value().as_str() {
        "GPL-3.0" => License::Gpl30,
        "GPL-3.0-only" => License::Gpl30Only,
        _ => {
            error!("Could not convert license");
            License::Unknown
        }
    };

    AboutDialog::builder()
        .application_icon(config::APP_ID.get_value())
        .application_name(config::APP_NAME.get_value())
        .version(config::VERSION.get_value())
        .developer_name(config::DEVELOPER.get_value())
        .license_type(license)
        .issue_url(config::ISSUES_URL.get_value())
        .release_notes(parse_release_notes_xml())
        .copyright(format!("© 2025 {}", config::DEVELOPER.get_value()))
        .documenters(parse_documenters())
        .translator_credits(parse_translators())
        .build()
}

fn parse_release_notes_xml() -> String {
    let metainfo = assets::get_meta_info();
    let mut release_xml = String::new();

    let mut release_version = String::new();
    let mut release_count = 1;

    for line in metainfo.lines() {
        let line = line.trim();
        if line.starts_with("<release") {
            if release_count >= 5 {
                break;
            }

            let start_pattern = r#"version=""#;
            let end_pattern = r#"" date="#;
            let Some(version_start) = line.find(start_pattern) else {
                continue;
            };
            let Some(version_end) = line.find(end_pattern) else {
                continue;
            };
            let version_str = &line[version_start + start_pattern.len()..version_end];
            let (Ok(version), Ok(app_version)) = (
                Version::parse(version_str),
                Version::parse(config::VERSION.get_value()),
            ) else {
                continue;
            };
            if version != app_version {
                let _ = write!(release_xml, "<p><em>Previous version {version}</em></p>");
                release_count += 1;
            }

            let _ = write!(release_version, "{version}");
            continue;
        } else if line.starts_with("</release>") {
            release_version.clear();
            continue;
        }
        if release_version.is_empty() {
            continue;
        }

        if line.starts_with("<p>")
            || line.starts_with("<ul>")
            || line.starts_with("<ol>")
            || line.starts_with("<li>")
            || line.starts_with("</p>")
            || line.starts_with("</ul>")
            || line.starts_with("</ol>")
            || line.starts_with("</li>")
        {
            let _ = writeln!(release_xml, "{line}");
        }
    }

    release_xml
}

fn parse_documenters() -> Vec<String> {
    let yaml: Vec<CreditsYaml> = serde_yaml::from_str(CREDITS_DOCUMENTATION).unwrap_or_default();

    yaml.into_iter()
        .map(|credit| format!("{} {}", credit.name, credit.link.unwrap_or_default()))
        .collect()
}

fn parse_translators() -> String {
    let yaml: Vec<CreditsYaml> = serde_yaml::from_str(CREDITS_TRANSLATIONS).unwrap_or_default();
    let mut credits = String::new();

    for credit in yaml {
        let _ = writeln!(
            credits,
            "{} ({}) {}",
            credit.name,
            credit.language.unwrap_or_default(),
            credit.link.unwrap_or_default()
        );
    }

    credits.trim().to_string()
}
