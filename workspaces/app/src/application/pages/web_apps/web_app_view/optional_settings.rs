use crate::application::App;
use anyhow::{Error, Result, anyhow};
use common::desktop_file::{DesktopFile, category::Category};
use gtk::{
    InputPurpose, Label, ListItem, SignalListItemFactory, gio,
    glib::{BoxedAnyObject, object::Cast},
    prelude::{BoxExt, EditableExt, ListItemExt},
};
use libadwaita::{
    ComboRow, EntryRow, PreferencesDialog, PreferencesGroup, PreferencesPage,
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
    error: RefCell<Option<Error>>,
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
            error: RefCell::new(None),
            app: app.clone(),
            desktop_file: desktop_file.clone(),
            pref_page,
            optional_pref_group,
            description_row,
            category_row,
        })
    }

    pub fn init(self: &Rc<Self>) {
        if let Some(is_init) = self.init.get()
            && *is_init
        {
            return;
        }

        self.pref_page.add(&self.optional_pref_group);

        self.optional_pref_group.add(&self.description_row);
        self.optional_pref_group.add(&self.category_row);

        self.connect_description_row();
        self.connect_category_row();

        let _ = self.init.set(true);
    }

    pub fn show_dialog<Callback: Fn(Result<()>) + 'static>(
        self: &Rc<Self>,
        on_close: Option<Callback>,
    ) -> PreferencesDialog {
        self.init();

        let dialog = PreferencesDialog::builder()
            .title("Optional Settings")
            .height_request(300)
            .build();
        dialog.add(&self.pref_page);

        let self_clone = self.clone();

        dialog.connect_closed(move |_dialog| {
            if let Some(callback) = &on_close {
                if let Some(error) = self_clone.error.borrow_mut().take() {
                    callback(Err(error));
                } else {
                    callback(Ok(()));
                }
            }
        });

        dialog.present(Some(&self.app.window.adw_window));
        dialog
    }

    fn build_optional_pref_group() -> PreferencesGroup {
        PreferencesGroup::builder()
            .title("Optional")
            .description("Settings for desktops that use a categorized app menu")
            .build()
    }

    fn build_description_row(desktop_file: &Rc<RefCell<DesktopFile>>) -> EntryRow {
        let description = desktop_file.borrow().get_description().unwrap_or_default();

        EntryRow::builder()
            .title("Short app description")
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
            box_container.append(&Label::new(Some(category.to_string_ui())));

            list_item.set_child(Some(&box_container));
        });

        let combo_row = ComboRow::builder()
            .title("Category")
            .subtitle("Pick a category")
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
        }

        combo_row
    }

    fn connect_description_row(self: &Rc<Self>) {
        let self_clone = self.clone();

        self.description_row.connect_apply(move |entry_row| {
            self_clone
                .desktop_file
                .borrow_mut()
                .set_description(&entry_row.text());
        });
    }

    fn connect_category_row(self: &Rc<Self>) {
        let desktop_file_clone = self.desktop_file.clone();
        let self_clone = self.clone();

        self.category_row
            .connect_selected_item_notify(move |combo_row| {
                let selected_item = combo_row.selected_item();
                let Some(selected_item) = selected_item else {
                    return;
                };
                let Ok(category_item_boxed) = selected_item.downcast::<BoxedAnyObject>() else {
                    self_clone.set_error("Failed to downcast selected item in category_row", None);
                    return;
                };
                let category = category_item_boxed.borrow::<Category>();
                desktop_file_clone.borrow_mut().set_category(&category);
            });
    }

    fn set_error(self: &Rc<Self>, message: &str, error: Option<Error>) {
        if let Some(error) = error {
            let new_error = error.context(message.to_string());
            *self.error.borrow_mut() = Some(new_error);
        } else {
            *self.error.borrow_mut() = Some(anyhow!(message.to_string()));
        }
    }
}
