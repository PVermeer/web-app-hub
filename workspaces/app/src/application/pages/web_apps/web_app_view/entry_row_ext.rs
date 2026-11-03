use common::desktop_file::{self, DesktopFile};
use gtk::{
    EventControllerFocus, InputPurpose,
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
    fmt::Display,
    rc::{Rc, Weak},
};
use tracing::{debug, error};

#[derive(Debug)]
pub enum EntryRowValidation {
    Empty(EntryRow),
    InvalidUrl(EntryRow),
    Dirty(EntryRow),
    Valid(EntryRow),
}
impl Display for EntryRowValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty(entry_row) => write!(f, "{}: empty", entry_row.title()),
            Self::InvalidUrl(entry_row) => write!(f, "{}: invalid url", entry_row.title()),
            Self::Dirty(entry_row) => write!(f, "{}: not applied", entry_row.title()),
            Self::Valid(entry_row) => write!(f, "{}: valid", entry_row.title()),
        }
    }
}

struct EntryRowExtEntry {
    entry_row: EntryRow,
    desktop_file: Rc<DesktopFile>,
    key: desktop_file::Key,
    default_title: String,
    is_applied_once: bool,
}

struct EntryRowExtEntryWeak {
    entry_row: WeakRef<EntryRow>,
    desktop_file: Weak<DesktopFile>,
    key: desktop_file::Key,
    default_title: String,
    is_applied_once: bool,
}

impl EntryRowExtEntryWeak {
    fn new(
        entry_row: &EntryRow,
        desktop_file: &Rc<DesktopFile>,
        key: &desktop_file::Key,
        default_title: &str,
        is_applied_once: bool,
    ) -> Self {
        Self {
            entry_row: entry_row.downgrade(),
            desktop_file: Rc::downgrade(desktop_file),
            key: key.clone(),
            default_title: default_title.to_string(),
            is_applied_once,
        }
    }

    fn upgrade(&self) -> Option<EntryRowExtEntry> {
        Some(EntryRowExtEntry {
            entry_row: self.entry_row.upgrade()?,
            desktop_file: self.desktop_file.upgrade()?,
            key: self.key.clone(),
            default_title: self.default_title.clone(),
            is_applied_once: self.is_applied_once,
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
    desktop_file: &Rc<DesktopFile>,
    key: &desktop_file::Key,
    default_title: &str,
    is_applied_once: bool,
) {
    cleanup_desktop_files();

    DESKTOP_FILES.with(|map| {
        map.borrow_mut().insert(
            entry_row.id(),
            EntryRowExtEntryWeak::new(entry_row, desktop_file, key, default_title, is_applied_once),
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

fn update_entry_row_ext(entry_row_ext: &EntryRowExtEntry) {
    cleanup_desktop_files();

    let EntryRowExtEntry {
        entry_row,
        desktop_file,
        key,
        default_title,
        is_applied_once,
    } = entry_row_ext;

    set_entry_row_ext(
        entry_row,
        desktop_file,
        key,
        default_title,
        *is_applied_once,
    );
}

/// Add validation and the apply button to ``EntryRow``.
pub trait EntryRowAppExt {
    fn id(&self) -> usize;
    fn set_entry_row_ext(
        &self,
        desktop_file: &Rc<DesktopFile>,
        key: &desktop_file::Key,
        default_title: &str,
    );
    fn on_changes(&self);
    fn on_apply(&self);
    fn saved_text(&self) -> Option<String>;
    fn default_title(&self) -> Option<String>;
    fn validate(&self, do_dirty_check: bool, apply_to_field: bool) -> EntryRowValidation;
    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool;
    fn is_dirty(&self) -> bool;
    fn unfocus(&self);
}

impl EntryRowAppExt for EntryRow {
    fn id(&self) -> usize {
        self.as_ptr() as usize
    }

    fn set_entry_row_ext(
        &self,
        desktop_file: &Rc<DesktopFile>,
        key: &desktop_file::Key,
        default_title: &str,
    ) {
        set_entry_row_ext(self, desktop_file, key, default_title, false);
        self.set_show_apply_button(true);
        let entry_row = self.clone();
        let focus_controller = EventControllerFocus::new();

        self.connect_apply(EntryRowAppExt::on_apply);
        self.connect_changed(EntryRowAppExt::on_changes);
        self.connect_entry_activated(|entry_row| entry_row.emit_by_name("apply", &[]));
        focus_controller.connect_leave(move |_| entry_row.emit_by_name("apply", &[]));

        self.add_controller(focus_controller);
    }

    fn on_changes(&self) {
        if get_entry_row_ext(self).is_some_and(|entry_row_ext| !entry_row_ext.is_applied_once) {
            return;
        }
        self.validate(false, true);
    }

    fn on_apply(&self) {
        let Some(mut entry_row_ext) = get_entry_row_ext(self) else {
            return;
        };

        entry_row_ext.is_applied_once = true;
        update_entry_row_ext(&entry_row_ext);

        // This is a bit sketchy, would think this would recurse
        // with the focus_controller but gtk seems to handle it.
        if self.is_valid(false, false) {
            self.unfocus();
        }
    }

    fn saved_text(&self) -> Option<String> {
        let Some(EntryRowExtEntry {
            desktop_file, key, ..
        }) = get_entry_row_ext(self)
        else {
            error!("saved_text Called on EntryRow before set_entry_row_ext is set is set");
            return None;
        };

        Some(desktop_file.get_by_key(&key).unwrap_or_default())
    }

    fn default_title(&self) -> Option<String> {
        let Some(EntryRowExtEntry { default_title, .. }) = get_entry_row_ext(self) else {
            error!("default_title Called on EntryRow before set_entry_row_ext is set");
            return None;
        };

        Some(default_title)
    }

    fn validate(&self, do_dirty_check: bool, apply_to_field: bool) -> EntryRowValidation {
        let Some(default_title) = self.default_title() else {
            error!("Validate ran on EntryRow before set_entry_row_ext is set");
            return EntryRowValidation::Valid(self.clone());
        };

        let text = self.text();
        let current_text = text.trim();

        let validation = if current_text.is_empty() {
            EntryRowValidation::Empty(self.clone())
        } else if matches!(self.input_purpose(), InputPurpose::Url)
            && DesktopFile::validate_url(current_text).is_err()
        {
            EntryRowValidation::InvalidUrl(self.clone())
        } else if do_dirty_check && self.is_dirty() {
            EntryRowValidation::Dirty(self.clone())
        } else {
            EntryRowValidation::Valid(self.clone())
        };

        if apply_to_field {
            match validation {
                EntryRowValidation::Empty(ref _entry_row) => {
                    self.set_show_apply_button(false);
                    self.add_css_class("error");
                    self.set_title(&t!("forms.validation.empty", field = &default_title));
                }

                EntryRowValidation::InvalidUrl(ref _entry_row) => {
                    self.set_show_apply_button(false);

                    self.add_css_class("error");
                    self.set_title(&t!("forms.validation.url", field = &default_title));
                }

                EntryRowValidation::Dirty(ref _entry_row) => {
                    self.set_show_apply_button(true);
                    self.add_css_class("error");
                    self.set_title(&t!("forms.validation.dirty", field = &default_title));
                }

                EntryRowValidation::Valid(ref _entry_row) => {
                    self.set_show_apply_button(self.is_dirty());
                    self.remove_css_class("error");
                    self.set_title(&default_title);
                }
            }
        }

        // This is a bit weird. Setting this button does not behave like one would think.
        // Need to enable it back (in the same frame???) so it's ready for the next user input.
        // My guess is that it internally resets to the current value.
        self.set_show_apply_button(true);

        debug!(%validation);
        validation
    }

    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool {
        matches!(
            self.validate(do_dirty_check, apply_to_field),
            EntryRowValidation::Valid(_)
        )
    }

    fn is_dirty(&self) -> bool {
        if let Some(saved_text) = self.saved_text() {
            self.text().trim() != saved_text
        } else {
            error!("is_dirty Called on EntryRow before set_entry_row_ext is set");
            false
        }
    }

    fn unfocus(&self) {
        if let Some(parent) = self.parent() {
            parent.grab_focus();
        }
    }
}
