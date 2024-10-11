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
    // Hold on to an stdin instance
    let stdin = stdin();
    // Input line by line
    let mut input = String::new();
    let mut keep_looping = true;

    print!("ats tracking> ");
    stdout().flush().unwrap();
    while keep_looping {
        // We can't use for line in stdin.lines() because that locks stdin while looping
        input.clear();
        stdin.read_line(&mut input)?;

        match ShellOption::try_from(input.trim()) {
            // If we should exit, do that
            Ok(ShellOption::Exit) => keep_looping = false,
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
            }
            .map_or_else(|e| println!("{e}"), |_| ()),
        };

        print!("ats tracking> ");
        stdout().flush().unwrap();
    }

    Ok(())
}

fn help() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

fn create<C: Queryable>(conn: &mut C) -> Result<(), Box<dyn std::error::Error>> {
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
    // Multiple inputs parse date
    let parse_date = |s: &str| {
        if !s.is_empty() {
            // If a date was given, try to parse it
            Date::parse(
                s,
                format_description!("[month repr:numerical]/[day]/[year]"),
            )
        } else {
            // The string being empty is fine, just use today
            Ok(time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
                .date())
        }
    };

    // Initialize the fields
    source = input("Source (job board, referral, etc):", wrap_ok)?;
    company = input("Company:", wrap_ok)?;
    job_title = input("Job Title:", wrap_ok)?;
    application_date = input(
        "Application date (leave blank for today) (mm/dd/yy):",
        parse_date,
    )?;
    time_investment = input(
        "Time taken to complete application (leave blank for unknown) (mm:ss):",
        |s| {
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
        },
    )?;
    automated_response = input("Was there an automated email after applying? [y/n]:", |s| {
        if s.starts_with(&['y', 'Y']) {
            Ok(true)
        } else if s.starts_with(&['n', 'N']) {
            Ok(false)
        } else {
            Err("Enter 'y' for yes or 'n' for no")
        }
    })?;
    human_response = input(
        "Response sent by a human later\nEnter r for rejection, i for interview request, or leave blank for none:",
        |s| {
            if s.starts_with(&['r', 'R']) {
                Ok(HumanResponse::Rejection)
            } else if s.starts_with(&['i', 'I']) {
                Ok(HumanResponse::InterviewRequest)
            } else if s.is_empty() {
                Ok(HumanResponse::None)
            } else {
                Err("Unknown response")
            }
        },
    )?;
    // Only prompt if human response is not null
    if let HumanResponse::None = human_response {
        human_response_date = None
    } else {
        human_response_date = Some(input(
            "Response date (leave blank for today) (mm/dd/yy):",
            parse_date,
        )?);
    }
    application_website = Some(input(
        "Application website (if applied using the company website):",
        wrap_ok,
    )?)
    .filter(|s| !s.is_empty());
    notes = Some(input("Notes:", wrap_ok)?).filter(|s| !s.is_empty());

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

    // println!("Job application: {new_application:?}");
    insert_job_application(conn, &new_application)?;
    Ok(())
}

fn read<C: Queryable>(conn: &mut C, read_type: ReadType) -> Result<(), Box<dyn std::error::Error>> {
    todo!();
}

fn update<C: Queryable>(
    conn: &mut C,
    update_type: UpdateType,
    id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!();
}

fn delete<C: Queryable>(conn: &mut C, id: i32) -> Result<(), Box<dyn std::error::Error>> {
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
        print!("{prompt} ");
        std::io::stdout().flush().unwrap();
    }

    Err(io::Error::other("Reached EOF from stdin"))
}
