use std::{cell::RefCell, rc::Rc};

use super::slint_generated::{AppWindow, JobApplicationView};
use mysql::prelude::Queryable;
use repository::job_application_repository::get_job_application_by_id;
use slint::ComponentHandle;

// Public functions

/// Handle the callback for `use-job-application`
/// 
/// Sets the sidebar job application to the job application that corresponds to the given ID.
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

/// Handle the callback for `submit-job-application`
/// 
/// Creates or updates the job application on the sidebar into the database
pub fn handle_submit_job_application(ui: &AppWindow) {
    let ui_clone = ui.as_weak();

    ui.on_submit_job_application(move || {
        if let Some(ui) = ui_clone.upgrade() {
            let job_application_view = ui.get_selected_job_application();
            #[cfg(debug_assertions)]
            print_job_application_to_terminal(&job_application_view);
        } else {
            eprintln!("Error submitting job application: AppWindow no longer exists");
        }
    });
}

/// Handle the callback for `new-job-application`
/// 
/// Clears the selected job application and sets the application date to now
pub fn handle_new_job_application(ui: &AppWindow) {
    let ui_clone = ui.as_weak();

    ui.on_new_job_application(move || {
        if let Some(ui) = ui_clone.upgrade() {
            ui.set_selected_job_application(JobApplicationView {
                // Application date should be today
                application_date: time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                    .date()
                    .into(),
                // Default for everything else is find
                // Important defaults:
                // - id = 0: Necessary because this is what `handle_submit_job_application(...)` uses to mean create instead of update.
                // - human_response = None
                // - strings are ""
                ..JobApplicationView::default()
            });
        } else {
            eprintln!("Error clearing job application: AppWindow no longer exists");
        }
    });
}

// Helper functions

fn select_row<C: Queryable>(conn: &mut C, ui: AppWindow, application_id: i32) {
    match get_job_application_by_id(conn, application_id) {
        // Put job application into selected-job-application
        Ok(Some(ja)) => ui.set_selected_job_application(ja.into()),
        Ok(None) => eprintln!("No job application matches id {application_id}"),
        Err(error) => eprintln!("{error}"),
    };
}

#[cfg(debug_assertions)]
fn print_job_application_to_terminal(job_application_view: &JobApplicationView) {
    println!(
        "ID: {}
Source: {}
Company: {}
Job Title: {}
Application date: {:?}
Time investment: {}
Human response: {:?}
Human response date: {:?}
Days to respond: {}
Application website: {}
Notes: {}",
        job_application_view.id,                  //: int,
        job_application_view.source,              //: string,
        job_application_view.company,             //: string,
        job_application_view.job_title,           //: string,
        job_application_view.application_date,    //: Date,
        job_application_view.time_investment,     //: string,
        job_application_view.human_response,      //: HumanResponseView,
        job_application_view.human_response_date, //: Date,
        job_application_view.days_to_respond,     //: int,
        job_application_view.application_website, //: string,
        job_application_view.notes,               //: string,
    );
}
