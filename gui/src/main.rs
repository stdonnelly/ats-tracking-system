// Some workaround for windows that was in the project template
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, rc::Rc};

use controller::{
    handle_date_diff, handle_new_job_application, handle_submit_job_application,
    handle_use_job_application,
};
use dotenv::dotenv;
use repository::{
    job_application_model::JobApplication, job_application_repository::get_job_applications,
};
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};

mod model;
use model::AppWindow;
mod controller;

fn main() -> Result<(), Box<dyn Error>> {
    // Objects that should be owned by the main function
    dotenv()?;
    let conn = Rc::new(RefCell::new(repository::get_conn()?));
    let ui = AppWindow::new()?;

    // Set up initial state
    let all_applications =
        get_job_applications(RefCell::borrow_mut(&conn).as_mut()).unwrap_or_default();
    let table_rows: VecModel<ModelRc<StandardListViewItem>> = VecModel::default();
    for application in all_applications {
        table_rows.push(job_application_into_row(&application));
    }
    ui.set_table_rows(ModelRc::new(table_rows));

    // Set up callbacks
    handle_use_job_application(&conn, &ui);
    handle_submit_job_application(&conn, &ui);
    handle_new_job_application(&ui);
    handle_date_diff(&ui);

    // Finally, run the UI
    ui.run()?;
    Ok(())
}

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
