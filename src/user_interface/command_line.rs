use std::{
    fmt::Display,
    io::{self, stdin, stdout, Write},
};

use mysql::prelude::Queryable;
use time::{Date, Duration};

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

    // Initialize the fields
    source = input("Source (job board, referral, etc): ").unwrap();
    company = input("Company: ").unwrap();
    job_title = input("Job Title: ").unwrap();
    // TODO: The rest

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

    insert_job_application(new_application);
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

/// Print a prompt and return the input, parsed as `T`.
/// Returns an Error if stdin.lines() returns an error, or if stdin.lines() ends (this should not happen because stdin should not have EOF).
fn input<T>(prompt: &str) -> Result<T, io::Error>
where
    T: for<'a> TryFrom<&'a str>,
    for<'a> <T as TryFrom<&'a str>>::Error: Display,
{
    print!("{prompt} ");
    std::io::stdout().flush().unwrap();
    for line in stdin().lines() {
        match T::try_from((line?).as_str()) {
            Ok(o) => return Ok(o),
            Err(e) => println!("Invalid input: {e}"),
        }
    }

    // This should be unreachable
    Err(io::Error::other("Reached EOF from stdin"))
}
