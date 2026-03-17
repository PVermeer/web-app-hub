use crate::application::{App, pages::web_apps::web_app_view::WebAppView};
use anyhow::anyhow;
use common::desktop_file::{DesktopFile, category::Category};
use gtk::{
    InputPurpose, Label, ListItem, SignalListItemFactory, gio,
    glib::{BoxedAnyObject, object::Cast},
    prelude::{BoxExt, EditableExt, GtkWindowExt, ListItemExt},
};
use libadwaita::{
    ComboRow, EntryRow, PreferencesDialog, PreferencesGroup, PreferencesPage, Toast,
    prelude::{
        AdwDialogExt, ComboRowExt, EntryRowExt, PreferencesDialogExt, PreferencesGroupExt,
        PreferencesPageExt,
    },
};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
use tracing::error;

pub struct OptionalSettings {
    init: OnceCell<bool>,
    app: Rc<App>,
    desktop_file: Rc<RefCell<DesktopFile>>,
    pref_page: PreferencesPage,
    optional_pref_group: PreferencesGroup,
    description_row: EntryRow,
    category_row: ComboRow,
}
impl OptionalSettings {
    pub fn new(app: &Rc<App>, desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let pref_page = PreferencesPage::new();
        let optional_pref_group = Self::build_optional_pref_group();
        let description_row = Self::build_description_row(desktop_file);
        let category_row = Self::build_category_row(desktop_file);

        Rc::new(Self {
            init: OnceCell::from(false),
            app: app.clone(),
            desktop_file: desktop_file.clone(),
            pref_page,
            optional_pref_group,
            description_row,
            category_row,
        })
    }

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        if let Some(is_init) = self.init.get()
            && *is_init
        {
            return;
        }

        self.pref_page.add(&self.optional_pref_group);

        self.optional_pref_group.add(&self.description_row);
        self.optional_pref_group.add(&self.category_row);

        self.connect_description_row(web_app_view);
        self.connect_category_row(web_app_view);

        let _ = self.init.set(true);
    }

    pub fn show_dialog(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) -> PreferencesDialog {
        self.init(web_app_view);

        let dialog = PreferencesDialog::builder()
            .title(t!("web_apps.web_app_view.optional.dialog.title"))
            .height_request(350)
            .can_close(false)
            .build();
        dialog.add(&self.pref_page);

        let self_clone = self.clone();
        dialog.connect_close_attempt(move |dialog| {
            if self_clone.is_applied_all() {
                dialog.force_close();
                return;
            }
            self_clone
                .app
                .window
                .adw_window
                .set_focus(None::<&gtk::Widget>);

            let apply_toast = Toast::builder()
                .title(t!("forms.apply_changes"))
                .button_label(t!("forms.force_close"))
                .build();

            let dialog_clone = dialog.clone();
            apply_toast.connect_button_clicked(move |_toast| dialog_clone.force_close());
            dialog.add_toast(apply_toast);
        });

        dialog.present(Some(&self.app.window.adw_window));
        dialog
    }

    fn build_optional_pref_group() -> PreferencesGroup {
        PreferencesGroup::builder()
            .title(t!("web_apps.web_app_view.optional.dialog.menu_group.title"))
            .description(t!(
                "web_apps.web_app_view.optional.dialog.menu_group.subtitle"
            ))
            .build()
    }

    fn build_description_row(desktop_file: &Rc<RefCell<DesktopFile>>) -> EntryRow {
        let description = desktop_file.borrow().get_description().unwrap_or_default();

        EntryRow::builder()
            .title(t!(
                "web_apps.web_app_view.optional.dialog.menu_group.description.title"
            ))
            .text(description)
            .show_apply_button(true)
            .input_purpose(InputPurpose::FreeForm)
            .build()
    }

    fn build_category_row(desktop_file: &Rc<RefCell<DesktopFile>>) -> ComboRow {
        let all_categories = Category::get_all();

        // Some weird factory setup where the list calls factory methods...
        // First create all data structures, then set data from ListStore.
        // Why is this so unnecessary complicated? ¯\_(ツ)_/¯
        let list = gio::ListStore::new::<BoxedAnyObject>();
        for category in all_categories {
            let boxed = BoxedAnyObject::new(category);
            list.append(&boxed);
        }
        let factory = SignalListItemFactory::new();
        factory.connect_bind(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<ListItem>() else {
                error!(?list_item, "Failed to downcast list item");
                return;
            };
            let Some(category_item_boxed) = list_item
                .item()
                .and_then(|item| item.downcast::<BoxedAnyObject>().ok())
            else {
                error!(?list_item, "Failed to downcast boxed list item");
                return;
            };

            let category = category_item_boxed.borrow::<Category>();
            let box_container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let icon = category.get_icon();

            box_container.append(&icon);
            box_container.append(&Label::new(Some(&Self::category_to_string_ui(*category))));

            list_item.set_child(Some(&box_container));
        });

        let combo_row = ComboRow::builder()
            .title(t!(
                "web_apps.web_app_view.optional.dialog.menu_group.category.title"
            ))
            .subtitle(t!(
                "web_apps.web_app_view.optional.dialog.menu_group.category.subtitle"
            ))
            .model(&list)
            .factory(&factory)
            .build();

        if let Some(current_category) = desktop_file.borrow().get_category()
            && let Some(index) = all_categories
                .iter()
                .position(|category| current_category == category.to_string())
            && let Ok(index) = index.try_into()
        {
            combo_row.set_selected(index);
        } else if let Some(index) = all_categories
            .iter()
            .position(|category| Category::Network == *category)
            && let Ok(index) = index.try_into()
        {
            combo_row.set_selected(index);
        }

        combo_row
    }

    fn category_to_string_ui(category: Category) -> String {
        match category {
            Category::AudioVideo => t!(
                "web_apps.web_app_view.optional.dialog.menu_group.category.categories.audio_video"
            )
            .to_string(),
            Category::Audio => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.audio")
                    .to_string()
            }
            Category::Video => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.video")
                    .to_string()
            }
            Category::Development => t!(
                "web_apps.web_app_view.optional.dialog.menu_group.category.categories.development"
            )
            .to_string(),
            Category::Education => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.education")
                    .to_string()
            }
            Category::Game => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.game")
                    .to_string()
            }
            Category::Graphics => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.graphics")
                    .to_string()
            }
            Category::Network => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.network")
                    .to_string()
            }
            Category::Office => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.office")
                    .to_string()
            }
            Category::Science => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.science")
                    .to_string()
            }
            Category::Settings => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.settings")
                    .to_string()
            }
            Category::System => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.system")
                    .to_string()
            }
            Category::Utility => {
                t!("web_apps.web_app_view.optional.dialog.menu_group.category.categories.utility")
                    .to_string()
            }
        }
    }

    fn connect_description_row(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();
        let apply_icon = WebAppView::build_validate_icon();
        let applied_text = Rc::new(RefCell::new(
            self.desktop_file
                .borrow()
                .get_description()
                .unwrap_or_default(),
        ));

        WebAppView::connect_entry_row_focus_controller_for_apply(
            &self.description_row,
            &applied_text,
            &apply_icon,
            None,
        );

        self.description_row.connect_apply(move |entry_row| {
            self_clone
                .desktop_file
                .borrow_mut()
                .set_description(&entry_row.text());
            web_app_view_clone.on_desktop_file_change();
        });
    }

    fn connect_category_row(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        let desktop_file_clone = self.desktop_file.clone();
        let web_app_view_clone = web_app_view.clone();

        self.category_row
            .connect_selected_item_notify(move |combo_row| {
                let selected_item = combo_row.selected_item();
                let Some(selected_item) = selected_item else {
                    return;
                };
                let Ok(category_item_boxed) = selected_item.downcast::<BoxedAnyObject>() else {
                    web_app_view_clone.on_error(
                        "Failed to set category",
                        Some(&anyhow!("Failed to downcast category from list")),
                    );
                    return;
                };
                let category = category_item_boxed.borrow::<Category>();
                desktop_file_clone.borrow_mut().set_category(&category);
                web_app_view_clone.on_desktop_file_change();
            });
    }

    fn is_applied_all(self: &Rc<Self>) -> bool {
        let description_borrow = self
            .desktop_file
            .borrow()
            .get_description()
            .unwrap_or_default();
        let current_description = self.description_row.text();

        description_borrow == current_description
    }
}
