mod application;

use application::App;
use common::{
    config::{self},
    utils::{self, OnceLockExt},
};
use libadwaita::gio::prelude::{ApplicationExt, ApplicationExtManual};
use rust_i18n::locale;
use tracing::{Level, debug, info};
use tracing_subscriber::{FmtSubscriber, util::SubscriberInitExt};

#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");

fn init_logging() {
    let mut log_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::INFO
    };
    log_level = utils::env::get_log_level().unwrap_or(log_level);
    // Disable > info logging for external crates
    let filter = format!(
        "{}={log_level},common={log_level}",
        config::APP_NAME_UNDERSCORE.get_value()
    );

    let logger = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(filter)
        .finish();
    logger.init();
}

fn init_locale() {
    if let Some(language) = utils::env::get_language() {
        debug!(locale = language, "Trying to use user locale");

        let supported_languages = rust_i18n::available_locales!();
        let language_stem = language
            .split('_')
            .next()
            .map(std::string::ToString::to_string)
            .unwrap_or_default();
        let mut locale = String::new();

        // Create fallback (e.g. `en_GB` -> `en` when en_GB does not exists)
        for lang in supported_languages {
            if lang == language {
                locale = lang.to_string();
                break;
            }
            let lang_stem = lang
                .split('_')
                .next()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            if language_stem == lang_stem {
                locale = lang_stem;
            }
        }

        if !locale.is_empty() {
            rust_i18n::set_locale(&locale);
        }
    }
    info!(locale = &*locale(), "Init locale");
}

fn main() {
    if cfg!(debug_assertions) {
        println!("======== Running debug build ========");
    }

    config::init();
    init_logging();
    info!("Version: {}", config::VERSION.get_value());
    init_locale();

    config::log_all_values_debug();

    let adw_application = libadwaita::Application::builder()
        .application_id(config::APP_ID.get_value())
        .build();

    adw_application.connect_activate(|adw_application| {
        App::new(adw_application).init();
    });

    adw_application.run();
}
