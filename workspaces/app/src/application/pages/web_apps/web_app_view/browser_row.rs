use super::{WebAppView, form::FormField};
use crate::application::App;
use anyhow::anyhow;
use common::{browsers::Browser, desktop_file::DesktopFile};
use gtk::{
    Image, Label, ListItem, SignalListItemFactory, gio,
    glib::{BoxedAnyObject, JoinHandle, object::Cast},
    prelude::{BoxExt, ListItemExt, WidgetExt},
};
use libadwaita::{
    ComboRow,
    prelude::{ActionRowExt, ComboRowExt},
};
use std::{cell::RefCell, rc::Rc};
use tracing::{error, warn};

struct ComboRowStateObject {
    is_selectable: bool,
    item: BoxedAnyObject,
}

pub struct BrowserRow {
    pub combo_row: ComboRow,
    desktop_file: Rc<RefCell<DesktopFile>>,
    validate_icon: Image,
}

impl BrowserRow {
    pub fn new(desktop_file: &Rc<RefCell<DesktopFile>>, app: &Rc<App>, is_new: bool) -> Rc<Self> {
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

        if let Some(browser_index) = desktop_file
            .borrow()
            .get_browser()
            .and_then(|browser| browser.get_index())
            && let Ok(index) = browser_index.try_into()
        {
            combo_row.set_selected(index);
        } else if is_new && let Some(browser) = installed_browsers.first() {
            // ComboRow has already selected the first item on load, so sync this if new.
            desktop_file.borrow_mut().set_browser(browser);
        } else {
            // Last should be no browser
            combo_row.set_selected(list.find(&no_browser_boxed).unwrap_or_default());
        }

        let validate_icon = Image::from_icon_name("dialog-warning-symbolic");
        validate_icon.set_visible(false);
        validate_icon.set_css_classes(&["error"]);
        combo_row.add_suffix(&validate_icon);

        Rc::new(Self {
            combo_row,
            desktop_file: desktop_file.clone(),
            validate_icon,
        })
    }

    pub fn init(self: &Rc<Self>, web_app_view: &Rc<WebAppView>) {
        let self_clone = self.clone();
        let web_app_view_clone = web_app_view.clone();

        self.combo_row.connect_selected_item_notify(move |_| {
            self_clone.on_select(&web_app_view_clone);
        });
    }

    fn on_select(&self, web_app_view: &Rc<WebAppView>) {
        let selected_item = self.combo_row.selected_item();
        let Some(selected_item) = selected_item else {
            return;
        };
        let Ok(browser_item_boxed) = selected_item.downcast::<BoxedAnyObject>() else {
            web_app_view.on_error(
                "Failed to save browser",
                Some(&anyhow!("Failed to downcast selected item in browser_row")),
            );
            return;
        };
        let item_state = browser_item_boxed.borrow::<ComboRowStateObject>();
        let browser = item_state.item.borrow::<Rc<Browser>>();

        self.desktop_file.borrow_mut().set_browser(&browser);

        web_app_view.isolate_row.on_isolation_change(web_app_view);

        if self.is_valid(false)
            && let Some(next_field) = self.combo_row.next_sibling()
        {
            next_field.grab_focus();
        }
    }
}

impl FormField for BrowserRow {
    fn apply(self: Rc<Self>, _web_app_view: &Rc<WebAppView>) -> Option<JoinHandle<()>> {
        None
    }

    fn is_valid(&self, _do_dirty_check: bool) -> bool {
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

    fn reset(&self, _do_dirty_check: bool) {
        warn!("RESET NOT YET IMPLEMENTED FOR BROWSER ROW");
    }
}
