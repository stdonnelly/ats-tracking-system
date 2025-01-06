use std::{cell::RefCell, rc::Rc};

use super::slint_generated::AppWindow;
use mysql::prelude::Queryable;
use repository::job_application_repository::get_job_application_by_id;
use slint::ComponentHandle;

pub fn handle_use_job_application<C, Q>(conn: &Rc<RefCell<C>>, ui: &AppWindow)
where
    C: Queryable + AsMut<Q> + 'static,
    Q: Queryable,
{
    let conn_clone = Rc::clone(conn);
    let ui_clone = ui.as_weak();

    ui.on_use_job_application(move |application_id| {
        if let Some(ui) = ui_clone.upgrade() {
            select_row(
                RefCell::borrow_mut(&conn_clone).as_mut(),
                ui,
                application_id,
            );
        } else {
            eprintln!("Error setting selected job application: AppWindow no longer exists");
        };
    });
}

fn select_row<C: Queryable>(conn: &mut C, ui: AppWindow, application_id: i32) {
    match get_job_application_by_id(conn, application_id) {
        // Put job application into selected-job-application
        Ok(Some(ja)) => ui.set_selected_job_application(ja.into()),
        Ok(None) => eprintln!("No job application matches id {application_id}"),
        Err(error) => eprintln!("{error}"),
    };
}
