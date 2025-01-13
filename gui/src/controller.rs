use std::{cell::RefCell, iter::once, rc::Rc};

use crate::model::{self, get_today_as_slint_date, AppWindow, JobApplicationView};
use mysql::prelude::Queryable;
use repository::{
    job_application_model::JobApplication,
    job_application_repository::{
        get_job_application_by_id, get_job_applications, insert_job_application,
        update_job_application,
    },
};
use slint::{ComponentHandle, Model, ModelRc, StandardListViewItem, ToSharedString, VecModel};

// Public functions

/// Initialize the data in the ui
///
/// Populates table and resets sidebar.
pub fn init_ui<C: Queryable>(conn: &mut C, ui: &AppWindow) {
    // Get job applications
    let all_applications: Vec<JobApplication> =
        get_job_applications(conn).unwrap_or_else(|error| {
            // If get_job_applications produces an error, print that there was an error, then just use an empty Vec
            eprintln!("Error getting initial state of job applications table: {error}");
            Vec::default()
        });

    // Initialize vec model to map to
    let table_rows: VecModel<ModelRc<StandardListViewItem>> = VecModel::default();

    // Map the applications to `table_rows`
    for application in all_applications {
        table_rows.push(job_application_into_row(&application));
    }

    // Set the table rows
    ui.set_table_rows(ModelRc::new(table_rows));

    // Finally, reset the sidebar
    reset_selected_row(ui);
}

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
            reset_selected_row(&ui);
        } else {
            eprintln!("Error clearing job application: AppWindow no longer exists");
        }
    });
}

/// Handle the callback for `date-diff`
///
/// Returns the difference between two dates in days (to - from)
pub fn handle_date_diff(ui: &AppWindow) {
    ui.on_date_diff(|from: model::Date, to: model::Date| -> i32 {
        // Ignore invocations where one or both dates are 0/0/0
        if to == model::Date::default() || from == model::Date::default() {
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
    });
}

// Helper functions

/// Put a job application into a row that can be used by the Slint StandardTableView
///
/// The design is similar to cli::command_line::print_job_application_to_terminal(&JobApplication)
fn job_application_into_row(ja: &JobApplication) -> ModelRc<StandardListViewItem> {
    [
        // Using From::from to make sure StandardListViewItem is inferred
        StandardListViewItem::from(ja.id.to_string().as_str()),
        // All the rest can infer StandardListViewItem, but must be converted into &str
        // For whatever reason, StandardListViewItem implements From<&str> but not From<String>
        ja.source.as_str().into(),
        ja.company.as_str().into(),
        ja.job_title.as_str().into(),
        format!(
            "{:02}/{:02}/{}",
            ja.application_date.month() as u8,
            ja.application_date.day(),
            ja.application_date.year(),
        )
        .as_str()
        .into(),
        ja.time_investment
            .map_or("".to_string(), |t| {
                format!("{:02}:{:02}", t.whole_minutes(), t.whole_seconds() % 60)
            })
            .as_str()
            .into(),
        ja.human_response.to_string().as_str().into(),
        ja.human_response_date
            .map_or("".to_string(), |d| {
                format!("{:02}/{:02}/{}", d.month() as u8, d.day(), d.year())
            })
            .as_str()
            .into(),
        ja.human_response_date
            .map_or("".to_owned(), |resp_date: time::Date| {
                let duration_between_dates = resp_date - ja.application_date;
                duration_between_dates.whole_days().to_string()
            })
            .as_str()
            .into(),
        ja.application_website.as_deref().unwrap_or_default().into(),
        ja.notes.as_deref().unwrap_or_default().into(),
    ]
    .into()
}

/// Set the sidebar job application to the job application denoted by `application_id`
fn select_row<C: Queryable>(conn: &mut C, ui: AppWindow, application_id: i32) {
    match get_job_application_by_id(conn, application_id) {
        // Put job application into selected-job-application
        Ok(Some(ja)) => ui.set_selected_job_application(ja.into()),
        Ok(None) => eprintln!("No job application matches id {application_id}"),
        Err(error) => eprintln!("{error}"),
    };
}

fn reset_selected_row(ui: &AppWindow) {
    ui.set_selected_job_application(JobApplicationView {
        // Application date should be today
        application_date: get_today_as_slint_date(),
        // This will also set
        human_response_date: get_today_as_slint_date(),
        // Default for everything else is find
        // Important defaults:
        // - id = 0: Necessary because this is what `handle_submit_job_application(...)` uses to mean create instead of update.
        // - human_response = None
        // - strings are ""
        ..JobApplicationView::default()
    });
    ui.invoke_re_bind_selected();
}

/// If this is a debug build, print the job application to stdout
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

/// Save the given job application by either inserting or updating the database
///
/// If `job_application_view.id == 0`, an insert will be performed.
/// Otherwise, an update to the value at `job_application_view.id` will be performed.
/// When finished, the ui table will be updated
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

    // Try to convert the job application
    // The variable is mutable because we will pull from the database after the insert
    let mut job_application: JobApplication = job_application_view.try_into()?;

    let is_new = job_application.id == 0;

    if is_new {
        // Insert
        job_application = insert_job_application(conn, &job_application)?;

        // Since this is an insert, we should insert a row into the table instead of trying to edit an existing entry
        let table_rows: ModelRc<ModelRc<StandardListViewItem>> = ui.get_table_rows();

        // Try to just use the array in place
        if let Some(table_rows_vec) = table_rows
            .as_any()
            .downcast_ref::<VecModel<ModelRc<StandardListViewItem>>>()
        {
            table_rows_vec.push(job_application_into_row(&job_application));
        } else {
            // If that isn't possible, we need to recreate the model
            #[cfg(debug_assertions)]
            println!("Unable to use the table_rows model as-is. Recollecting.");

            ui.set_table_rows(ModelRc::new(
                table_rows
                    .iter()
                    .chain(once(job_application_into_row(&job_application)))
                    .collect::<VecModel<ModelRc<StandardListViewItem>>>(),
            ));
        }
    } else {
        // Update
        update_job_application(conn, &job_application)?;

        // Since this is an update, we should just update the row that contains the updated data
        let id = job_application.id;

        let table_rows: ModelRc<ModelRc<StandardListViewItem>> = ui.get_table_rows();

        // There may be a better way than this, but I can't find one
        // Loop over rows in the table. Whenever the id matches the id we are looking for, replace that row.
        for i in 0..table_rows.row_count() {
            if let Some(table_row) = table_rows.row_data(i) {
                if table_row.row_data(0) == Some(id.to_shared_string().into()) {
                    table_rows.set_row_data(i, job_application_into_row(&job_application));
                    break;
                }
            }
        }
    }

    reset_selected_row(ui);

    Ok(())
}
