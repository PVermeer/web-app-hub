use super::form::FormField;
use crate::application::App;
use anyhow::anyhow;
use common::{browsers::Browser, desktop_file::DesktopFile};
use gtk::{
    Image, Label, ListItem, SignalListItemFactory, Widget, gio,
    glib::{BoxedAnyObject, object::Cast},
    prelude::{BoxExt, ListItemExt, WidgetExt},
};
use libadwaita::{
    ComboRow,
    prelude::{ActionRowExt, ComboRowExt},
};
use std::rc::Rc;
use tracing::{debug, error};

struct ComboRowStateObject {
    is_selectable: bool,
    item: BoxedAnyObject,
}

pub struct BrowserRow {
    pub combo_row: ComboRow,
    app: Rc<App>,
    desktop_file: Rc<DesktopFile>,
    list: gio::ListStore,
    no_browser_boxed: BoxedAnyObject,
    validate_icon: Image,
}

impl BrowserRow {
    pub fn new(desktop_file: &Rc<DesktopFile>, app: &Rc<App>) -> Rc<Self> {
        let installed_browsers = app.browser_configs.get_installed_browsers();
        let no_browser = Rc::new(app.browser_configs.build_no_browser());

        // Some weird factory setup where the list calls factory methods...
        // First create all data structures, then set data from ListStore.
        // Why is this so unnecessary complicated? ¯\_(ツ)_/¯
        let list = gio::ListStore::new::<BoxedAnyObject>();
        for browser in installed_browsers {
            let boxed_state_object = ComboRowStateObject {
                is_selectable: true,
                item: BoxedAnyObject::new(browser.clone()),
            };
            let boxed = BoxedAnyObject::new(boxed_state_object);
            list.append(&boxed);
        }
        let no_browser_boxed = BoxedAnyObject::new(ComboRowStateObject {
            is_selectable: false,
            item: BoxedAnyObject::new(no_browser.clone()),
        });
        list.append(&no_browser_boxed);

        let factory = SignalListItemFactory::new();
        factory.connect_bind(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<ListItem>() else {
                error!(?list_item, "Failed to downcast list item");
                return;
            };
            let Some(item_state_boxed) = list_item
                .item()
                .and_then(|item| item.downcast::<BoxedAnyObject>().ok())
            else {
                error!(?list_item, "Failed to downcast boxed list item");
                return;
            };

            let state_object = item_state_boxed.borrow::<ComboRowStateObject>();
            let browser = state_object.item.borrow::<Rc<Browser>>();
            let box_container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let label = Label::new(Some(&browser.get_name_with_installation()));
            let icon = browser.get_icon();

            box_container.append(&icon);
            box_container.append(&label);

            if !browser.is_installed() {
                icon.add_css_class("error");
                box_container.add_css_class("dimmed");
                list_item.set_activatable(false);
                list_item.set_selectable(false);
            }

            if browser.is_no_browser() {
                label.set_label(&t!("web_apps.web_app_view.browser.no_browser"));
            }

            list_item.set_child(Some(&box_container));
        });

        let combo_row = ComboRow::builder()
            .title(t!("web_apps.web_app_view.browser.title"))
            .subtitle(t!("web_apps.web_app_view.browser.subtitle"))
            .model(&list)
            .factory(&factory)
            .build();

        let validate_icon = Image::from_icon_name("dialog-warning-symbolic");
        combo_row.add_suffix(&validate_icon);

        Rc::new(Self {
            app: app.clone(),
            combo_row,
            desktop_file: desktop_file.clone(),
            list: list.clone(),
            no_browser_boxed: no_browser_boxed.clone(),
            validate_icon,
        })
    }

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        self.set_from_desktop_file(web_app_view);

        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        self.combo_row.connect_selected_item_notify(move |_| {
            self_clone.on_select(&web_app_view_clone);
        });
    }

    fn on_select(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let selected_item = self.combo_row.selected_item();
        let Some(selected_item) = selected_item else {
            return;
        };
        let Ok(browser_item_boxed) = selected_item.downcast::<BoxedAnyObject>() else {
            web_app_view.error_message(
                "Failed to save browser",
                Some(&anyhow!("Failed to downcast selected item in browser_row")),
            );
            return;
        };
        let item_state = browser_item_boxed.borrow::<ComboRowStateObject>();
        let browser = item_state.item.borrow::<Rc<Browser>>();

        self.desktop_file.set_browser(&browser);
    }

    fn set_from_desktop_file(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        let installed_browsers = self.app.browser_configs.get_installed_browsers();

        if let Some(browser_index) = self
            .desktop_file
            .get_browser()
            .and_then(|browser| browser.get_index())
            && let Ok(index) = browser_index.try_into()
        {
            self.combo_row.set_selected(index);
        } else if web_app_view.is_new()
            && let Some(browser) = installed_browsers.first()
        {
            // ComboRow has already selected the first item on load, so sync this if new.
            self.desktop_file.set_browser(browser);
        } else {
            // Last should be no browser
            self.combo_row
                .set_selected(self.list.find(&self.no_browser_boxed).unwrap_or_default());
        }

        self.validate_icon.set_visible(false);
        self.validate_icon.set_css_classes(&["error"]);
    }

    fn on_reset(&self, web_app_view: &Rc<dyn super::WebAppViewShared>) {
        debug!("Resetting browser_row");

        self.set_from_desktop_file(web_app_view);
    }
}

impl FormField for BrowserRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_init(&web_app_view);
    }

    fn reset(&self, web_app_view: Rc<dyn super::WebAppViewShared>) {
        self.on_reset(&web_app_view);
    }

    fn is_valid(&self, _do_dirty_check: bool, _apply_to_field: bool) -> bool {
        let Some(selected_item) = self.combo_row.selected_item() else {
            return false;
        };
        let Ok(item_boxed) = selected_item.downcast::<BoxedAnyObject>() else {
            error!("Failed to downcast selected item in browser_row for apply check");
            return false;
        };
        let item_state = item_boxed.borrow::<ComboRowStateObject>();

        if item_state.is_selectable {
            self.validate_icon.set_tooltip_text(None);
            self.validate_icon.set_visible(false);
        } else {
            self.validate_icon
                .set_tooltip_text(Some(&t!("forms.select_option")));
            self.validate_icon.set_visible(true);
        }

        true
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn form_widget(&self) -> Widget {
        self.combo_row.clone().upcast()
    }
}
