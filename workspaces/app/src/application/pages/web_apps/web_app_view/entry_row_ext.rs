use common::desktop_file::{self, DesktopFile};
use gtk::{
    InputPurpose,
    glib::{
        WeakRef,
        object::{ObjectExt, ObjectType},
    },
    prelude::{EditableExt, WidgetExt},
};
use libadwaita::{
    EntryRow,
    prelude::{EntryRowExt as _, PreferencesRowExt},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};
use tracing::error;

#[derive(Debug)]
pub enum EntryRowValidation {
    Empty(EntryRow),
    InvalidUrl(EntryRow),
    Valid(EntryRow),
}

struct EntryRowExtEntry {
    desktop_file: Rc<RefCell<DesktopFile>>,
    key: desktop_file::Key,
    default_title: String,
}

struct EntryRowExtEntryWeak {
    entry_row: WeakRef<EntryRow>,
    desktop_file: Weak<RefCell<DesktopFile>>,
    key: desktop_file::Key,
    default_title: String,
}

impl EntryRowExtEntryWeak {
    fn new(
        entry_row: &EntryRow,
        desktop_file: &Rc<RefCell<DesktopFile>>,
        key: desktop_file::Key,
        default_title: &str,
    ) -> Self {
        Self {
            entry_row: entry_row.downgrade(),
            desktop_file: Rc::downgrade(desktop_file),
            key,
            default_title: default_title.to_string(),
        }
    }

    fn upgrade(&self) -> Option<EntryRowExtEntry> {
        Some(EntryRowExtEntry {
            desktop_file: self.desktop_file.upgrade()?,
            key: self.key.clone(),
            default_title: self.default_title.clone(),
        })
    }
}

thread_local! {
    static DESKTOP_FILES: RefCell<HashMap<usize, EntryRowExtEntryWeak>> =
        RefCell::new(HashMap::new());
}

fn cleanup_desktop_files() {
    DESKTOP_FILES.with(|map| {
        map.borrow_mut().retain(|_, entry| {
            entry.entry_row.upgrade().is_some() && entry.desktop_file.upgrade().is_some()
        });
    });
}

fn set_entry_row_ext(
    entry_row: &EntryRow,
    desktop_file: &Rc<RefCell<DesktopFile>>,
    key: desktop_file::Key,
    default_title: &str,
) {
    cleanup_desktop_files();

    DESKTOP_FILES.with(|map| {
        map.borrow_mut().insert(
            entry_row.id(),
            EntryRowExtEntryWeak::new(entry_row, desktop_file, key, default_title),
        );
    });
}

fn get_entry_row_ext(entry_row: &EntryRow) -> Option<EntryRowExtEntry> {
    cleanup_desktop_files();

    DESKTOP_FILES.with(|map| {
        map.borrow()
            .get(&entry_row.id())
            .and_then(EntryRowExtEntryWeak::upgrade)
    })
}

pub trait EntryRowAppExt {
    fn id(&self) -> usize;
    fn set_entry_row_ext(
        &self,
        desktop_file: &Rc<RefCell<DesktopFile>>,
        key: desktop_file::Key,
        default_title: &str,
    );
    fn saved_text(&self) -> Option<String>;
    fn default_title(&self) -> Option<String>;
    fn validate(&self, do_dirty_check: bool) -> EntryRowValidation;
    fn is_dirty(&self) -> bool;
}

impl EntryRowAppExt for EntryRow {
    fn id(&self) -> usize {
        self.as_ptr() as usize
    }

    fn set_entry_row_ext(
        &self,
        desktop_file: &Rc<RefCell<DesktopFile>>,
        key: desktop_file::Key,
        default_title: &str,
    ) {
        set_entry_row_ext(self, desktop_file, key, default_title);
    }

    fn saved_text(&self) -> Option<String> {
        let Some(EntryRowExtEntry {
            desktop_file, key, ..
        }) = get_entry_row_ext(self)
        else {
            error!("saved_text Called on EntryRow before set_entry_row_ext is set is set");
            return None;
        };

        Some(desktop_file.borrow().get_by_key(&key).unwrap_or_default())
    }

    fn default_title(&self) -> Option<String> {
        let Some(EntryRowExtEntry { default_title, .. }) = get_entry_row_ext(self) else {
            error!("default_title Called on EntryRow before set_entry_row_ext is set");
            return None;
        };

        Some(default_title)
    }

    fn validate(&self, do_dirty_check: bool) -> EntryRowValidation {
        if let Some(default_title) = self.default_title() {
            let text = self.text();
            let current_text = text.trim();

            let validation = if current_text.is_empty() {
                EntryRowValidation::Empty(self.clone())
            } else if matches!(self.input_purpose(), InputPurpose::Url)
                && DesktopFile::validate_url(current_text).is_err()
            {
                EntryRowValidation::InvalidUrl(self.clone())
            } else {
                EntryRowValidation::Valid(self.clone())
            };

            match validation {
                EntryRowValidation::Empty(ref _entry_row) => {
                    self.add_css_class("error");
                    self.set_title(&t!("forms.validation.empty", field = &default_title));
                    self.set_show_apply_button(false);
                }

                EntryRowValidation::InvalidUrl(ref _entry_row) => {
                    self.add_css_class("error");
                    self.set_title(&t!("forms.validation.url", field = &default_title));
                    self.set_show_apply_button(false);
                }

                EntryRowValidation::Valid(ref _entry_row) => {
                    if do_dirty_check && self.is_dirty() {
                        self.add_css_class("error");
                        self.set_title(&t!("forms.validation.dirty", field = &default_title));
                    } else {
                        self.remove_css_class("error");
                        self.set_title(&default_title);
                    }
                    self.set_show_apply_button(true);
                }
            }

            validation
        } else {
            error!("validate Called on EntryRow before set_entry_row_ext is set");
            EntryRowValidation::Valid(self.clone())
        }
    }

    fn is_dirty(&self) -> bool {
        if let Some(saved_text) = self.saved_text() {
            self.text().trim() != saved_text
        } else {
            error!("is_dirty Called on EntryRow before set_entry_row_ext is set");
            false
        }
    }
}
