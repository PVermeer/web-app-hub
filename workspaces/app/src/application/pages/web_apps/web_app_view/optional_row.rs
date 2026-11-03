mod category_row;
mod description_row;

use super::{WebAppViewShared, form::FormField};
use crate::application::App;
use category_row::CategoryRow;
use common::desktop_file::DesktopFile;
use description_row::DescriptionRow;
use gtk::Image;
use libadwaita::{
    ActionRow, PreferencesDialog, PreferencesGroup, PreferencesPage, Toast,
    prelude::{
        ActionRowExt, AdwDialogExt, PreferencesDialogExt, PreferencesGroupExt, PreferencesPageExt,
    },
};
use std::rc::Rc;

pub struct OptionalRow {
    pub action_row: ActionRow,
    app: Rc<App>,
    pref_page: PreferencesPage,
    form_fields: Vec<Rc<dyn FormField>>,
}

impl OptionalRow {
    pub fn new(app: &Rc<App>, desktop_file: &Rc<DesktopFile>) -> Rc<Self> {
        let action_row = ActionRow::builder()
            .title(t!("web_apps.web_app_view.optional.title"))
            .subtitle(t!("web_apps.web_app_view.optional.subtitle"))
            .activatable(true)
            .build();
        let suffix = Image::from_icon_name("go-next-symbolic");
        action_row.add_suffix(&suffix);

        let pref_page = PreferencesPage::new();
        let optional_pref_group = PreferencesGroup::builder()
            .title(t!("web_apps.web_app_view.optional.dialog.menu_group.title"))
            .description(t!(
                "web_apps.web_app_view.optional.dialog.menu_group.subtitle"
            ))
            .build();

        let description_row = DescriptionRow::new(desktop_file);
        let category_row = CategoryRow::new(desktop_file);

        let form_fields: Vec<Rc<dyn FormField>> =
            vec![description_row.clone(), category_row.clone()];

        optional_pref_group.add(&description_row.entry_row);
        optional_pref_group.add(&category_row.combo_row);

        pref_page.add(&optional_pref_group);

        Rc::new(Self {
            action_row,
            app: app.clone(),
            pref_page,
            form_fields,
        })
    }

    fn on_init(self: &Rc<Self>, web_app_view: &Rc<dyn WebAppViewShared>) {
        let self_clone = self.clone();
        let web_app_view = web_app_view.clone();

        self.action_row.connect_activated(move |_| {
            self_clone.show_dialog(&web_app_view);
        });
    }

    pub fn show_dialog(
        self: &Rc<Self>,
        web_app_view: &Rc<dyn WebAppViewShared>,
    ) -> PreferencesDialog {
        self.reset(web_app_view.clone());

        let dialog = PreferencesDialog::builder()
            .title(t!("web_apps.web_app_view.optional.dialog.title"))
            .height_request(350)
            .can_close(false)
            .build();
        dialog.add(&self.pref_page);

        let self_clone = self.clone();

        dialog.connect_close_attempt(move |dialog| {
            // Dialog can be a bit emotional stressed and focus to much
            dialog.set_focus(None::<&gtk::Widget>);

            if self_clone.is_valid(true, true) {
                dialog.force_close();
                return;
            }

            let apply_toast = Toast::builder()
                .title(t!("forms.validation_error"))
                .button_label(t!("forms.force_close"))
                .build();

            let dialog_clone = dialog.clone();
            apply_toast.connect_button_clicked(move |_toast| dialog_clone.force_close());
            dialog.add_toast(apply_toast);
        });

        dialog.present(Some(&self.app.window.adw_window));
        dialog
    }
}

impl FormField for OptionalRow {
    fn init(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        self.on_init(&web_app_view);

        for field in &self.form_fields {
            field.clone().init(web_app_view.clone());
        }
    }

    fn apply(self: Rc<Self>, web_app_view: Rc<dyn WebAppViewShared>) {
        for field in &self.form_fields {
            field.clone().apply(web_app_view.clone());
        }
    }

    fn reset(&self, web_app_view: Rc<dyn WebAppViewShared>) {
        for field in &self.form_fields {
            field.reset(web_app_view.clone());
        }
    }

    fn is_valid(&self, do_dirty_check: bool, apply_to_field: bool) -> bool {
        self.form_fields
            .iter()
            .all(|field| field.is_valid(do_dirty_check, apply_to_field))
    }

    fn is_dirty(&self) -> bool {
        self.form_fields.iter().any(|field| field.is_dirty())
    }
}
