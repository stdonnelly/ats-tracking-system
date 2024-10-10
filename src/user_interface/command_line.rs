use std::{
    convert::Infallible,
    fmt::Display,
    io::{self, stdin, stdout, Write},
};

use mysql::prelude::Queryable;
use time::{macros::format_description, Date, Duration};

use crate::repository::job_application_repository::{
    insert_job_application, HumanResponse, JobApplication,
};

use super::shell_option::{ReadType, ShellOption, UpdateType};

/// The main loop that runs the prompt
/// Will exit if there is an  
pub fn main_loop<C: Queryable>(conn: &mut C) -> Result<(), io::Error> {
    let stdin = stdin();

    print!("ats tracking> ");
    stdout().flush().unwrap();
    for line in stdin.lines() {
        let input = line?;

        match ShellOption::try_from(input.as_str()) {
            // If we should exit, do that
            Ok(ShellOption::Exit) => break,
            // If there is an error, print the error
            Err(s) => println!("{}", s),
            // For any other commands
            Ok(command) => match command {
                ShellOption::Help => help(),
                ShellOption::Create => create(conn),
                ShellOption::Read(read_type) => read(conn, read_type),
                ShellOption::Update(update_type, id) => update(conn, update_type, id),
                ShellOption::Delete(id) => delete(conn, id),
                ShellOption::Exit => unreachable!(),
            },
        };

        print!("ats tracking> ");
        stdout().flush().unwrap();
    }

    Ok(())
}

fn help() {
    print!(
        "ATS Tracking System

Available commands:
  help | h
  exit | quit
  create
  read (all | pending | search <search_query> | one <id>)
  update (response | other) <id>
  delete <id>
"
    );
}

fn create<C: Queryable>(conn: &mut C) {
    // Declare the variables here to make sure I define all of them
    let source: String;
    let company: String;
    let job_title: String;
    let application_date: Date;
    let time_investment: Option<Duration>;
    let automated_response: bool;
    let human_response: HumanResponse;
    let human_response_date: Option<Date>;
    let application_website: Option<String>;
    let notes: Option<String>;

    // This will be used by multiple inputs
    let wrap_ok = |s: &str| Result::<_, Infallible>::Ok(s.to_owned());

    // Initialize the fields
    source = input("Source (job board, referral, etc): ", wrap_ok).unwrap();
    company = input("Company: ", wrap_ok).unwrap();
    job_title = input("Job Title: ", wrap_ok).unwrap();
    application_date = input("Application date (leave blank for today) (mm/dd/yy):", |s| {
        if !s.is_empty() {
            // If a date was given, try to parse it
            Date::parse(
                s,
                format_description!("[month repr:numerical]/[day]/[year repr:last_two]"),
            )
        } else {
            // The string being empty is fine, just use today
            Ok(time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                .date())
        }
    })
    .unwrap();
    time_investment = input("Time taken to complete application (leave blank for unknown) (mm:ss):", |s| {
        if !s.is_empty() {
            if let Some((minutes_str, seconds_str)) = s.split_once(':') {
                let minutes = match minutes_str.parse::<i32>() {
                    Ok(i) => i,
                    Err(e) => return Err(e.to_string()),
                };
                let seconds = match seconds_str.parse::<i32>() {
                    Ok(i) => i,
                    Err(e) => return Err(e.to_string()),
                };
                Ok(Some(Duration::seconds((minutes * 60 + seconds) as i64)))
            } else {
                Err("No colon found".to_owned())
            }
        } else {
            Ok(None)
        }
    }).unwrap();

    // Construct the new application.
    let new_application = JobApplication {
        id: 0,
        source,
        company,
        job_title,
        application_date,
        time_investment,
        automated_response,
        human_response,
        human_response_date,
        application_website,
        notes,
    };

    insert_job_application(conn, &new_application);
}

fn read<C: Queryable>(conn: &mut C, read_type: ReadType) {
    todo!();
}

fn update<C: Queryable>(conn: &mut C, update_type: UpdateType, id: i32) {
    todo!();
}

fn delete<C: Queryable>(conn: &mut C, id: i32) {
    todo!();
}

/// Prints a given prompt and returns the input, parsed by `parse` to `T`
/// Returns an Error if stdin.lines() returns an error, or if stdin.lines() ends (this should not happen because stdin should not have EOF).
/// If `parse` returns an error, the program will try again, displaying the error message given by `parse`
fn input<T, U, F>(prompt: &str, parse: F) -> Result<T, io::Error>
where
    U: Display,
    F: Fn(&str) -> Result<T, U>,
{
    print!("{prompt} ");
    std::io::stdout().flush().unwrap();
    for line in stdin().lines() {
        match parse((line?).trim()) {
            Ok(o) => return Ok(o),
            Err(e) => println!("Invalid input: {e}"),
        }
    }

    Err(io::Error::other("Reached EOF from stdin"))
}
