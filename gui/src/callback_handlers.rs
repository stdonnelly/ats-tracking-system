use std::{cell::RefCell, rc::Rc};

use crate::slint_generated;

use super::slint_generated::{AppWindow, JobApplicationView};
use mysql::prelude::Queryable;
use repository::{
    job_application_model::JobApplication,
    job_application_repository::{get_job_application_by_id, insert_job_application},
};
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
pub fn handle_submit_job_application<C, Q>(conn: &Rc<RefCell<C>>, ui: &AppWindow)
where
    C: Queryable + AsMut<Q> + 'static,
    Q: Queryable,
{
    let conn_clone = Rc::clone(conn);
    let ui_clone = ui.as_weak();

    ui.on_submit_job_application(move || {
        if let Some(ui) = ui_clone.upgrade() {
            let job_application_view = ui.get_selected_job_application();
            submit_job_application(
                RefCell::borrow_mut(&conn_clone).as_mut(),
                &ui,
                job_application_view,
            )
            // Print any errors, but otherwise discard them.
            // We may want to actually do something with these errors later, though
            .map_or_else(|e| eprintln!("{e}"), |_| ());
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

/// Handle the callback for `date-diff`
///
/// Returns the difference between two dates in days (to - from)
pub fn handle_date_diff(ui: &AppWindow) {
    ui.on_date_diff(
        |from: slint_generated::Date, to: slint_generated::Date| -> i32 {
            // Ignore invocations where one or both dates are 0/0/0
            if to == slint_generated::Date::default() || from == slint_generated::Date::default() {
                return 0;
            }

            // Only try if both can be converted
            match (time::Date::try_from(from), time::Date::try_from(to)) {
                (Ok(from_date), Ok(to_date)) => {
                    let duration = to_date - from_date;
                    duration.whole_days() as i32
                }
                // Both error arms will just return 0.
                // It would probably be best to display some error in the future
                (Err(error), _) => {
                    eprintln!("Error parsing the 'from' date in difference: {error}");
                    0
                }
                (_, Err(error)) => {
                    eprintln!("Error parsing the 'to' date in difference: {error}");
                    0
                }
            }
        },
    );
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
        job_application_view.application_website, //: string,
        job_application_view.notes,               //: string,
    );
}

fn submit_job_application<C: Queryable>(
    conn: &mut C,
    ui: &AppWindow,
    job_application_view: JobApplicationView,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        println!("Submitting job application:");
        print_job_application_to_terminal(&job_application_view);
    }

    let job_application: JobApplication = job_application_view.try_into()?;

    // Insert or update the job application
    if job_application.id == 0 {
        // Insert
        let new_job_application = insert_job_application(conn, &job_application)?;

        // Add the new job application to the job application table and sidebar
        todo!();
    } else {
        // Update full job application
        // Needs a new function in the job application model
        todo!();

        // Find the job application in the table and update it
        todo!();
    }

    Ok(())
}
