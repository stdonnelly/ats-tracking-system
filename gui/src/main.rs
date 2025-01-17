//! Entry point for the GUI version of ats-tracking
//!
//! This crate uses Slint for a GUI.

// Some workaround for windows that was in the project template
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, ops::DerefMut, rc::Rc};

use controller::{
    handle_date_diff, handle_new_job_application, handle_submit_job_application,
    handle_use_job_application, init_ui,
};
use dotenv::dotenv;
use slint::ComponentHandle as _;

mod model;
use model::AppWindow;
mod controller;

fn main() -> Result<(), Box<dyn Error>> {
    // Objects that should be owned by the main function
    dotenv()?;
    let conn = Rc::new(RefCell::new(repository::get_conn()?));
    let ui = AppWindow::new()?;

    // Set initial state
    init_ui(RefCell::borrow_mut(&conn).deref_mut(), &ui);

    // Set up callbacks
    handle_use_job_application(&conn, &ui);
    handle_submit_job_application(&conn, &ui);
    handle_new_job_application(&ui);
    handle_date_diff(&ui);

    // Finally, run the UI
    ui.run()?;
    Ok(())
}
