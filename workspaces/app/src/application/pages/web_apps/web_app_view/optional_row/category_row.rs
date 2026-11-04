use crate::application::pages::web_apps::web_app_view::{WebAppViewShared, form::FormField};
use anyhow::anyhow;
use common::desktop_file::{DesktopFile, category::Category};
use gtk::{
    Label, ListItem, SignalListItemFactory, gio,
    glib::{BoxedAnyObject, object::Cast},
    prelude::{BoxExt, ListItemExt},
};
use libadwaita::{ComboRow, prelude::ComboRowExt};
use std::rc::Rc;
use tracing::{debug, error};

pub struct CategoryRow {
    pub combo_row: ComboRow,
    desktop_file: Rc<DesktopFile>,
}

impl CategoryRow {
    pub fn new(desktop_file: &Rc<DesktopFile>) -> Rc<Self> {
        let title =
            t!("web_apps.web_app_view.optional.dialog.menu_group.category.title").to_string();
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
            .title(&title)
            .subtitle(t!(
                "web_apps.web_app_view.optional.dialog.menu_group.category.subtitle"
            ))
            .model(&list)
            .factory(&factory)
            .build();

        Rc::new(Self {
            combo_row,
            desktop_file: desktop_file.clone(),
        })
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

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn WebAppViewShared>) {
        self.set_from_desktop_file();

        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        self.combo_row.connect_selected_item_notify(move |_| {
            self_clone.on_select(&web_app_view_clone);
        });
    }

    fn on_select(&self, web_app_view: &Rc<dyn WebAppViewShared>) {
        let selected_item = self.combo_row.selected_item();
        let Some(selected_item) = selected_item else {
            return;
        };
        let Ok(category_item_boxed) = selected_item.downcast::<BoxedAnyObject>() else {
            web_app_view.error_message(
                "Failed to set category",
                Some(&anyhow!("Failed to downcast selected item in category_row")),
            );
            return;
        };
        let category = category_item_boxed.borrow::<Category>();

        self.desktop_file.set_category(&category);
    }

    fn set_from_desktop_file(&self) {
        let all_categories = Category::get_all();

        if let Some(current_category) = self.desktop_file.get_category()
            && let Some(index) = all_categories
                .iter()
                .position(|category| current_category == category.to_string())
            && let Ok(index) = index.try_into()
        {
            self.combo_row.set_selected(index);
        } else if let Some(index) = all_categories
            .iter()
            .position(|category| Category::Network == *category)
            && let Ok(index) = index.try_into()
        {
            self.combo_row.set_selected(index);
        }
    }

    fn on_reset(&self) {
        debug!("Resetting CategoryRow");

        self.set_from_desktop_file();
    }
}

impl FormField for CategoryRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn apply(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_select(&web_app_view);
    }

    fn reset(&self, _web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_reset();
    }

    fn is_valid(&self, _do_dirty_check: bool, _apply_to_field: bool) -> bool {
        true
    }

    fn is_dirty(&self) -> bool {
        false
    }
}
