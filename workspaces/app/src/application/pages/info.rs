use crate::application::{
    App,
    pages::{NavPage, PrefPage},
};
use common::browsers;
use gtk::{Image, Orientation};
use libadwaita::{
    ActionRow, ExpanderRow, NavigationPage, PreferencesGroup, PreferencesPage,
    gtk::{self, Label, prelude::BoxExt},
    prelude::{ExpanderRowExt, PreferencesGroupExt, PreferencesPageExt},
};
use std::rc::Rc;

pub struct InfoPage {
    nav_page: NavigationPage,
    nav_row: ActionRow,
    prefs_page: PreferencesPage,
}
impl NavPage for InfoPage {
    fn get_navpage(&self) -> &NavigationPage {
        &self.nav_page
    }

    fn get_nav_row(&self) -> Option<&ActionRow> {
        Some(&self.nav_row)
    }
}
impl InfoPage {
    const CONTENT_MARGING: i32 = 12;

    pub fn new() -> Rc<Self> {
        let title = t!("info.title");
        let icon = "help-about-symbolic";

        let PrefPage {
            nav_page,
            nav_row,
            prefs_page,
            ..
        } = Self::build_nav_page(&title, icon).with_preference_page();

        Rc::new(Self {
            nav_page,
            nav_row,
            prefs_page,
        })
    }

    pub fn init(&self, app: &Rc<App>) {
        let info_pref_group = PreferencesGroup::new();
        let expandable_pref_group = PreferencesGroup::new();

        let general_info = Self::build_tips_row(app);
        let permisssions = Self::build_permissions_row();

        info_pref_group.add(&general_info);
        expandable_pref_group.add(&permisssions);

        self.prefs_page.add(&info_pref_group);
        self.prefs_page.add(&expandable_pref_group);
    }

    fn build_tips_row(app: &Rc<App>) -> ExpanderRow {
        let row = ExpanderRow::builder()
            .title(t!("info.tips.title"))
            .use_markup(false)
            .expanded(true)
            .build();
        row.add_prefix(&Image::from_icon_name("checkbox-checked-symbolic"));

        let content_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(Self::CONTENT_MARGING)
            .margin_end(Self::CONTENT_MARGING)
            .margin_top(Self::CONTENT_MARGING)
            .margin_bottom(Self::CONTENT_MARGING)
            .build();

        let zen_browser_keybind = app
            .browser_configs
            .get_browser_by_config_name("zen")
            .map(|browser| browser.profile_setup_keybind.clone().unwrap_or_default())
            .unwrap_or_default();

        let text_label = Label::builder()
            .use_markup(true)
            .wrap(true)
            .label(t!(
                "info.tips.text_pango",
                base_chromium_keybind = browsers::Base::Chromium
                    .get_profile_setup_keybind()
                    .unwrap_or_default(),
                base_firefox_keybind = browsers::Base::Firefox
                    .get_profile_setup_keybind()
                    .unwrap_or_default(),
                zen_keybind = zen_browser_keybind
            ))
            .build();

        content_box.append(&text_label);
        row.add_row(&content_box);

        row
    }

    fn build_permissions_row() -> ExpanderRow {
        let row = ExpanderRow::builder()
            .title(t!("info.permissions.title"))
            .build();
        row.add_prefix(&Image::from_icon_name("security-medium-rtl-symbolic"));

        let content_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .margin_start(Self::CONTENT_MARGING)
            .margin_end(Self::CONTENT_MARGING)
            .margin_top(Self::CONTENT_MARGING)
            .margin_bottom(Self::CONTENT_MARGING)
            .build();

        let text_label = Label::builder()
            .use_markup(true)
            .wrap(true)
            .label(t!("info.permissions.text_pango"))
            .build();

        content_box.append(&text_label);
        row.add_row(&content_box);

        row
    }
}
