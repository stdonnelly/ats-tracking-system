// Some workaround for windows that was in the project template
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, rc::Rc};

use dotenv::dotenv;
use mysql::prelude::Queryable;
use repository::{
    job_application_model::JobApplication,
    job_application_repository::{get_job_application_by_id, get_job_applications},
};
use slint::{ModelRc, SharedString, StandardListViewItem, VecModel};
use time::Date;

slint::include_modules!();

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
    {
        let conn_clone = Rc::clone(&conn);
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
    };

    // Finally, run the UI
    ui.run()?;
    Ok(())
}

fn select_row<C: Queryable>(conn: &mut C, ui: AppWindow, application_id: i32) {
    match get_job_application_by_id(conn, application_id) {
        Ok(Some(ja)) => {
            // Put job application into selected-job-application
            ui.set_selected_job_application(JobApplicationView {
                id: ja.id,
                source: ja.source.into(),
                company: ja.company.into(),
                job_title: ja.job_title.into(),
                application_date: format!(
                    "{:02}/{:02}/{}",
                    ja.application_date.month() as u8,
                    ja.application_date.day(),
                    ja.application_date.year()
                )
                .into(),
                time_investment: ja.time_investment.map_or(SharedString::new(), |t| {
                    format!("{:02}:{:02}", t.whole_minutes(), t.whole_seconds() % 60).into()
                }),
                human_response: ja.human_response.to_string().into(),
                human_response_date: ja.human_response_date.map_or(SharedString::new(), |d| {
                    format!("{:02}/{:02}/{}", d.month() as u8, d.day(), d.year()).into()
                }),
                days_to_respond: ja.human_response_date.map_or(
                    SharedString::new(),
                    |resp_date: Date| {
                        let duration_between_dates = resp_date - ja.application_date;
                        duration_between_dates.whole_days().to_string().into()
                    },
                ),
                application_website: ja.application_website.as_deref().unwrap_or_default().into(),
                notes: ja.notes.as_deref().unwrap_or_default().into(),
            })
        }
        Ok(None) => eprintln!("No job application matches id {application_id}"),
        Err(error) => eprintln!("{error}"),
    };
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
            .map_or("".to_owned(), |resp_date: Date| {
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
