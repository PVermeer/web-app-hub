mod icon;
mod icon_fetcher;

use crate::application::App;
use anyhow::{Context, Result, bail};
use common::desktop_file::DesktopFile;
use gtk::{
    self, Align, Button, FileDialog, FileFilter, FlowBox, Image, Label, Orientation, SelectionMode,
    gdk_pixbuf::{Pixbuf, PixbufFormat},
    gio::prelude::FileExt,
    glib::GString,
    prelude::{BoxExt, ButtonExt, FlowBoxChildExt, ListBoxRowExt, WidgetExt},
};
use icon::Icon;
use icon_fetcher::IconFetcher;
use libadwaita::{
    AlertDialog, ButtonContent, ButtonRow, DialogPresentationMode, PreferencesGroup,
    PreferencesPage, PreferencesRow, ResponseAppearance, Spinner, StatusPage,
    gio::Cancellable,
    glib,
    prelude::{AdwDialogExt, AlertDialogExt, PreferencesGroupExt, PreferencesPageExt},
};
use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::HashMap,
    fs,
    rc::Rc,
    time::{Duration, SystemTime},
};
use tracing::{debug, error};

pub struct IconPicker {
    init: RefCell<bool>,
    fetched_icons_ts: RefCell<SystemTime>,
    prefs_page: PreferencesPage,
    app: Rc<App>,
    desktop_file: Rc<RefCell<DesktopFile>>,
    icons: RefCell<HashMap<String, Rc<Icon>>>,
    icons_ordered: RefCell<Vec<(String, Rc<Icon>)>>,
    removed_icon: RefCell<Option<Rc<Icon>>>,
    pref_row_icons: PreferencesRow,
    pref_row_icons_fail: PreferencesRow,
    pref_row_icons_flow_box: RefCell<Option<FlowBox>>,
    pref_group_icons_reset_button: Button,
    pref_group_icons_add_button_row: ButtonRow,
    content_box: gtk::Box,
    spinner: Spinner,
}
impl IconPicker {
    pub const DIALOG_SAVE: &str = "save";
    pub const DIALOG_CANCEL: &str = "cancel";
    pub const ONLINE_FETCH_THROTTLE: Duration = Duration::from_secs(20);
    pub const CURRENT_ICON_SOURCE: &str = "current";
    pub const ICON_MAX_SIZE: i32 = 64;

    pub fn new(app: &Rc<App>, desktop_file: &Rc<RefCell<DesktopFile>>) -> Rc<Self> {
        let icons = RefCell::new(HashMap::new());
        let icons_ordered = RefCell::new(Vec::new());
        let content_box = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .vexpand(true)
            .build();
        let spinner = Self::build_spinner();
        let prefs_page = PreferencesPage::new();
        let pref_row_icons = Self::build_pref_row_icons();
        let pref_row_icons_fail = Self::build_pref_row_icons_fail();
        let (pref_group_icons, pref_group_icons_reset_button) = Self::build_pref_group_icons();
        let pref_group_icons_add_button_row = Self::build_pref_row_add_icon();

        prefs_page.add(&pref_group_icons);
        pref_group_icons.add(&pref_row_icons);
        pref_group_icons.add(&pref_row_icons_fail);
        pref_group_icons.add(&pref_group_icons_add_button_row);

        content_box.append(&spinner);
        content_box.append(&prefs_page);

        let fetched_icons_ts = RefCell::new(
            SystemTime::now()
                .checked_sub(Self::ONLINE_FETCH_THROTTLE + Duration::from_secs(5))
                .unwrap_or(SystemTime::now()),
        );

        Rc::new(Self {
            init: RefCell::new(false),
            fetched_icons_ts,
            prefs_page,
            app: app.clone(),
            desktop_file: desktop_file.clone(),
            icons,
            icons_ordered,
            removed_icon: RefCell::new(None),
            pref_row_icons,
            pref_row_icons_fail,
            pref_row_icons_flow_box: RefCell::new(None),
            pref_group_icons_reset_button,
            pref_group_icons_add_button_row,
            content_box,
            spinner,
        })
    }

    pub fn init(self: &Rc<Self>) {
        let mut is_init = self.init.borrow_mut();
        self.load_icons(false);

        if *is_init {
            return;
        }

        let self_clone = self.clone();
        self.pref_group_icons_reset_button
            .connect_clicked(move |_| {
                self_clone.load_icons(true);
            });

        let self_clone = self.clone();
        self.pref_group_icons_add_button_row
            .connect_activated(move |_| {
                self_clone.load_icon_file_picker();
            });

        *is_init = true;
    }

    pub fn show_dialog<Success, Fail>(
        self: &Rc<Self>,
        success_cb: Option<Success>,
        fail_cb: Option<Fail>,
    ) -> AlertDialog
    where
        Success: Fn() + 'static,
        Fail: Fn() + 'static,
    {
        self.init();

        let dialog = AlertDialog::builder()
            .heading(t!("web_apps.web_app_view.icon.dialog.title"))
            .presentation_mode(DialogPresentationMode::Auto)
            .content_height(690)
            .content_width(500)
            .follows_content_size(false)
            .extra_child(&self.content_box)
            .build();
        dialog.add_response(
            Self::DIALOG_CANCEL,
            &t!("web_apps.web_app_view.icon.dialog.cancel"),
        );
        dialog.add_response(
            Self::DIALOG_SAVE,
            &t!("web_apps.web_app_view.icon.dialog.save"),
        );
        dialog.set_response_appearance(Self::DIALOG_SAVE, ResponseAppearance::Suggested);
        dialog.set_default_response(Some(Self::DIALOG_CANCEL));
        dialog.set_close_response(Self::DIALOG_CANCEL);

        let self_clone = self.clone();
        dialog.connect_response(
            Some(Self::DIALOG_SAVE),
            move |_, _| match (|| -> Result<()> {
                let icon = self_clone.get_selected_icon()?;
                self_clone.save(&icon)?;
                Ok(())
            })() {
                Ok(()) => {
                    if let Some(success_cb) = &success_cb {
                        success_cb();
                    }
                }
                Err(error) => {
                    error!("Error saving icon: {error:?}");
                    if let Some(fail_cb) = &fail_cb {
                        fail_cb();
                    }
                }
            },
        );

        dialog.connect_closed(|dialog| {
            dialog.set_extra_child(None::<&gtk::Widget>);
        });

        dialog.present(Some(&self.app.window.adw_window));
        dialog
    }

    pub async fn save_first_icon_found(self: &Rc<Self>) -> Result<()> {
        self.set_online_icons(false).await?;
        self.set_icons_ordered();
        let icons_ordered_borrow = self.icons_ordered.borrow();

        let Some((_source, icon)) = icons_ordered_borrow.first() else {
            bail!("No icons found")
        };

        self.save(icon)?;
        Ok(())
    }

    fn get_selected_icon(self: &Rc<Self>) -> Result<Rc<Icon>> {
        let source = self
            .clone()
            .pref_row_icons_flow_box
            .borrow()
            .clone()
            .context("Flow box does not exist")?
            .selected_children()
            .first()
            .context("Flowbox does not have a selected item")?
            .first_child()
            .context("Could not get container of selected flowbox item")?
            .widget_name()
            .to_string();

        let icon = self
            .icons
            .borrow()
            .get(&source)
            .context("Cannot find icon in HashMap???")?
            .clone();
        Ok(icon)
    }

    fn set_icons_loading(&self) {
        self.prefs_page.set_visible(false);
        self.spinner.set_visible(true);
        self.pref_row_icons.set_visible(false);
        self.pref_row_icons_fail.set_visible(true);
    }

    fn set_no_icons(&self) {
        self.prefs_page.set_visible(true);
        self.spinner.set_visible(false);
        self.pref_row_icons.set_visible(false);
        self.pref_row_icons_fail.set_visible(true);
    }

    fn set_show_icons(&self) {
        self.prefs_page.set_visible(true);
        self.spinner.set_visible(false);
        self.pref_row_icons.set_visible(true);
        self.pref_row_icons_fail.set_visible(false);
    }

    fn select_icon(&self, source: &str) {
        let flow_box_borrow = self.pref_row_icons_flow_box.borrow();
        if let Some(flow_box) = flow_box_borrow.as_ref() {
            let mut index = 0;
            while let Some(flow_box_child) = flow_box.child_at_index(index) {
                if let Some(widget) = flow_box_child.child()
                    && source == widget.widget_name()
                {
                    flow_box.select_child(&flow_box_child);
                    break;
                }

                index += 1;
            }
        }
    }

    fn load_icons(self: &Rc<Self>, force: bool) {
        let self_clone = self.clone();

        glib::spawn_future_local(async move {
            self_clone.set_icons_loading();

            if let Err(error) = self_clone.set_online_icons(force).await {
                error!("{error:?}");
            }
            if let Err(error) = self_clone.set_local_icon() {
                error!("{error:?}");
            }
            self_clone.set_icons_ordered();
            self_clone.reload_icon_flowbox();
        });
    }

    fn reload_icon_flowbox(self: &Rc<Self>) {
        let self_clone = self.clone();
        let flow_box = Self::build_pref_row_icons_flow_box();
        let pref_row_icons = &self_clone.pref_row_icons;
        pref_row_icons.set_child(Some(&flow_box));

        let icons_ordered_borrow = self_clone.icons_ordered.borrow();
        let mut first_icon_item = None;
        let mut current_icon_item = None;

        for icon_item in icons_ordered_borrow.iter() {
            let (source, icon) = icon_item;
            if first_icon_item.is_none() {
                first_icon_item = Some(icon_item);
            }
            if source == Self::CURRENT_ICON_SOURCE {
                current_icon_item = Some(icon_item);
            }

            let frame = gtk::Box::new(Orientation::Vertical, 0);
            frame.set_widget_name(source);
            let picture = Image::from_pixbuf(Some(&icon.pixbuf));
            let pixel_size = if icon.pixbuf.height() > Self::ICON_MAX_SIZE {
                Self::ICON_MAX_SIZE
            } else {
                icon.pixbuf.height()
            };
            picture.set_pixel_size(pixel_size);
            picture.set_size_request(Self::ICON_MAX_SIZE, Self::ICON_MAX_SIZE);
            frame.append(&picture);

            let size_text = format!("{} x {}", icon.pixbuf.width(), icon.pixbuf.height());
            let label = Label::builder().label(&size_text).build();
            frame.append(&label);

            flow_box.insert(&frame, -1);
        }

        *self_clone.pref_row_icons_flow_box.borrow_mut() = Some(flow_box);

        if let Some((source, _icon)) = current_icon_item {
            self.select_icon(source);
        } else if let Some((source, _icon)) = first_icon_item {
            self.select_icon(source);
        }

        if icons_ordered_borrow.is_empty() {
            self.set_no_icons();
        } else {
            self.set_show_icons();
        }
    }

    async fn set_online_icons(self: &Rc<Self>, force: bool) -> Result<()> {
        if !force && self.should_throttle() {
            return Ok(());
        }

        debug!("Fetching online icons");

        let Some(url) = self.desktop_file.borrow().get_url() else {
            bail!("No url on desktop file")
        };
        let Ok(mut icon_fetcher) = IconFetcher::new(&self.app, &url) else {
            bail!("Invalid url")
        };
        let Ok(icons) = icon_fetcher.get_online_icons().await else {
            bail!("Failed to get online icons")
        };

        let mut self_icons_borrow = self.icons.borrow_mut();
        *self_icons_borrow = icons
            .into_iter()
            .map(|icon| (icon.source.clone(), icon))
            .collect();

        if self_icons_borrow.is_empty() {
            bail!("No icons found for: {url}")
        }

        Ok(())
    }

    fn set_local_icon(self: &Rc<Self>) -> Result<()> {
        let Some(current_icon_path) = self.desktop_file.borrow().get_icon_path() else {
            bail!("No icon saved")
        };
        let current_icon =
            Rc::new(Icon::from_path(&current_icon_path).context("Could not load current image")?);

        let mut icons_borrow = self.icons.borrow_mut();
        let mut removed_icon_borrow = self.removed_icon.borrow_mut();

        if let Some(removed_icon) = removed_icon_borrow.take() {
            let removed_local_icon = icons_borrow.remove(Self::CURRENT_ICON_SOURCE);
            if let Some(removed_local_icon) = removed_local_icon {
                debug!(
                    source = removed_local_icon.source,
                    "Removed local icon from map"
                );
            }

            let is_new_icon = icons_borrow
                .insert(removed_icon.source.clone(), removed_icon.clone())
                .is_some();
            if is_new_icon {
                debug!(
                    source = removed_icon.source,
                    "Restored icon from local find"
                );
            }
        }

        let matched_icon = icons_borrow
            .iter()
            .find(|(_source, icon)| **icon == current_icon)
            .map(|(source, icon)| (source.clone(), icon.clone()));

        if let Some((matched_source, matched_icon)) = matched_icon {
            icons_borrow.remove(&matched_source);
            *removed_icon_borrow = Some(matched_icon);

            debug!(source = matched_source, "Replaced icon with local find");
        }
        icons_borrow.insert(Self::CURRENT_ICON_SOURCE.into(), current_icon);

        Ok(())
    }

    fn set_icons_ordered(&self) {
        let mut self_icons_ordered_borrow = self.icons_ordered.borrow_mut();

        *self_icons_ordered_borrow = self
            .icons
            .borrow()
            .iter()
            .map(|(source, icon)| (source.clone(), icon.clone()))
            .collect();
        self_icons_ordered_borrow.sort_by_key(|(_source, icon)| Reverse(icon.pixbuf.byte_length()));
    }

    fn should_throttle(self: &Rc<Self>) -> bool {
        let now = SystemTime::now();
        let previous_fetch_ts = *self.fetched_icons_ts.borrow();
        let Ok(previous_fetch_duration) = now.duration_since(previous_fetch_ts) else {
            error!("Could not calculate duration for throttle");
            return false;
        };

        if previous_fetch_duration < Self::ONLINE_FETCH_THROTTLE {
            return true;
        }

        *self.fetched_icons_ts.borrow_mut() = SystemTime::now();

        false
    }

    fn load_icon_file_picker(self: &Rc<Self>) {
        debug!("Opening file picker");

        let file_filter = FileFilter::new();
        file_filter.set_name(Some(&t!("web_apps.web_app_view.icon.file_dialog.filter")));
        let mimetypes: Vec<GString> = Pixbuf::formats()
            .iter()
            .flat_map(PixbufFormat::mime_types)
            .collect();
        for mimetype in &mimetypes {
            file_filter.add_mime_type(mimetype);
        }

        let file_dialog = FileDialog::builder()
            .title(t!("web_apps.web_app_view.icon.file_dialog.title"))
            .default_filter(&file_filter)
            .build();

        let self_clone = self.clone();
        let app_clone = self.app.clone();

        file_dialog.open(
            Some(&app_clone.window.adw_window),
            None::<&Cancellable>,
            move |file| {
                let Ok(file) = file else {
                    error!("Failed to get file");
                    return;
                };
                let Some(path) = file.path() else {
                    error!("Failed to get path");
                    return;
                };
                let filename = file.parse_name().to_string();

                debug!("Loading image: '{filename}'");

                let icon = match Icon::from_path(&path) {
                    Ok(icon) => icon,
                    Err(error) => {
                        error!("Failed to load image: '{error:?}'");
                        return;
                    }
                };

                self_clone
                    .icons
                    .borrow_mut()
                    .insert(filename.clone(), Rc::new(icon));

                self_clone.set_icons_ordered();
                self_clone.reload_icon_flowbox();
                self_clone.select_icon(&filename);
            },
        );
    }

    fn save(self: &Rc<Self>, icon: &Rc<Icon>) -> Result<()> {
        let mut desktop_file_borrow = self.desktop_file.borrow_mut();
        if let Some(old_icon_path) = desktop_file_borrow.get_icon_path()
            && old_icon_path.is_file()
        {
            fs::remove_file(old_icon_path).context("Failed to remove old icon")?;
        }

        let app_id = desktop_file_borrow
            .get_id()
            .context("No file id on DesktopFile")?;

        let icon_dir = &self.app.dirs.app_data_icons;
        let file_name = sanitize_filename::sanitize(format!("{app_id}.png"));
        let save_path = icon_dir.join(&file_name);

        debug!(
            "Saving icon '{}' to fs: {}",
            &file_name,
            save_path.display()
        );

        icon.pixbuf
            .savev(save_path.clone(), "png", &[])
            .context("Failed to save icon to fs")?;

        desktop_file_borrow.set_icon_path(&save_path);
        drop(desktop_file_borrow);

        Ok(())
    }

    fn build_spinner() -> Spinner {
        Spinner::builder()
            .height_request(48)
            .width_request(96)
            .halign(Align::Center)
            .valign(Align::Center)
            .hexpand(true)
            .vexpand(true)
            .build()
    }

    fn build_pref_group_icons() -> (PreferencesGroup, Button) {
        let content = ButtonContent::builder()
            .label(t!("web_apps.web_app_view.icon.button.reset"))
            .icon_name("folder-download-symbolic")
            .build();
        let button = Button::builder()
            .css_classes(["flat"])
            .child(&content)
            .build();

        let pref_group = PreferencesGroup::builder()
            .title(t!("web_apps.web_app_view.icon.title"))
            .header_suffix(&button)
            .build();

        (pref_group, button)
    }

    fn build_pref_row_add_icon() -> ButtonRow {
        ButtonRow::builder()
            .title(t!("web_apps.web_app_view.icon.button.add_icon"))
            .start_icon_name("list-add-symbolic")
            .build()
    }

    fn build_pref_row_icons_flow_box() -> FlowBox {
        FlowBox::builder()
            .height_request(96)
            .column_spacing(20)
            .row_spacing(10)
            .homogeneous(false)
            .selection_mode(SelectionMode::Single)
            .build()
    }

    fn build_pref_row_icons() -> PreferencesRow {
        PreferencesRow::builder().build()
    }

    fn build_pref_row_icons_fail() -> PreferencesRow {
        let status_page = StatusPage::builder()
            .title(t!("web_apps.web_app_view.icon.no_icons.title"))
            .description(t!("web_apps.web_app_view.icon.no_icons.description"))
            .css_classes(["compact"])
            .build();

        PreferencesRow::builder().child(&status_page).build()
    }
}
