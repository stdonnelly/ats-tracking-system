use std::{
    convert::Infallible,
    fmt::Display,
    io::{self, stdin, stdout, Write},
    path::Path,
};

use mysql::prelude::Queryable;
use time::{macros::format_description, Date, Duration};

use crate::repository::job_application_repository::{
    get_job_applications, get_pending_job_applications, insert_job_application, HumanResponse, JobApplication
};

use super::shell_option::{ReadType, ShellOption, UpdateType};

/// The main loop that runs the prompt
/// Will exit if there is an  
pub fn main_loop<C: Queryable>(conn: &mut C) -> Result<(), io::Error> {
    // Hold on to an stdin instance
    let stdin = stdin();
    // Temporary directory that is owned by this function
    // This will be automatically deleted when this function exits
    let temp_dir = tempfile::TempDir::new()?;
    // Input buffer to read line by line without locking stdin between queries
    let mut input = String::new();
    // Flag to tell the while loop when to stop
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
                ShellOption::Read(read_type) => read(conn, read_type, temp_dir.path()),
                ShellOption::Update(update_type, id) => update(conn, update_type, id),
                ShellOption::Delete(id) => delete(conn, id),
                ShellOption::Exit => unreachable!(),
            }
            .map_or_else(|e| println!("{e}"), |_| ()),
        };

        // Reprint the prompt
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

/// Prompt a user for all parts of a job application and insert the new element
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

fn read<C: Queryable>(
    conn: &mut C,
    read_type: ReadType,
    temp_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the job application(s), depending on read type
    let applications: Vec<JobApplication> = match read_type {
        ReadType::All => get_job_applications(conn)?,
        ReadType::Pending => get_pending_job_applications(conn)?,
        ReadType::Search(_query) => todo!(),
        ReadType::One(_id) => todo!(),
    };

    match applications.len() {
        0 => Err(Box::<dyn std::error::Error>::from(
            "No job application found",
        )),
        1 => {
            // This should never panic, we just verified there is exactly one job application
            let job_application = applications.get(0).unwrap();
            println!("One job application found:");
            println!(
                "ID: {}
Source: {}
Company: {}
Job title: {}
Application date: {:02}/{:02}/{}
Time Taken to complete application: {}
Was there an automated response after applying? {}
Response from a human: {}
Human response date: {}
Response time (days): {}
Application website: {}
Notes: {}",
                job_application.id,
                job_application.source.replace("\"", "\"\""),
                job_application.company.replace("\"", "\"\""),
                job_application.job_title.replace("\"", "\"\""),
                job_application.application_date.month() as u8,
                job_application.application_date.day(),
                job_application.application_date.year(),
                job_application
                    .time_investment
                    .map_or("".to_string(), |t| format!(
                        "{:02}:{:02}",
                        t.whole_minutes(),
                        t.whole_seconds() % 60
                    )),
                match job_application.automated_response {
                    true => "Yes",
                    false => "No",
                },
                job_application.human_response,
                job_application
                    .human_response_date
                    .map_or("".to_string(), |d| format!(
                        "{:02}/{:02}/{}",
                        d.month() as u8,
                        d.day(),
                        d.year()
                    )),
                job_application
                    .human_response_date
                    .map_or("".to_owned(), |resp_date: Date| {
                        let duration_between_dates = resp_date - job_application.application_date;
                        duration_between_dates.whole_days().to_string()
                    }),
                job_application
                    .application_website
                    .as_deref()
                    .map_or("".to_string(), |s| s.replace("\"", "\"\"")),
                job_application
                    .notes
                    .as_deref()
                    .map_or("".to_string(), |s| s.replace("\"", "\"\"")),
            );
            Ok(())
        }
        _ => print_table(applications, temp_dir),
    }
}

fn update<C: Queryable>(
    conn: &mut C,
    update_type: UpdateType,
    id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Deal with warnings for now
    let _ = conn;
    let _ = update_type;
    let _ = id;
    todo!();
}

fn delete<C: Queryable>(conn: &mut C, id: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Deal with the warnings for now
    let _ = conn;
    let _ = id;
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

/// Print the table, then show the results in the native spreadsheet application.
/// This is crude, but an easy way to display while other features are being worked on.
fn print_table(
    job_applications: Vec<JobApplication>,
    temp_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary file
    let mut file = tempfile::Builder::new()
        // Suffix must be CSV for opener to recognize it
        .suffix(".csv")
        // Keep even when `file` goes out of scope.
        // This relies on the destructor of `temp_dir` to clean files.
        // The program should clean this as soon as the spreadsheet system is closed, but using a spreadsheet system is a hack anyway.
        .keep(true)
        // Create in the temporary directory
        .tempfile_in(temp_dir)?;

    // Write to that file
    writeln!(&mut file, "ID,Source,Company,Job Title,Application Date,Time Taken,Auto Response,Human Response,Date,Website,Notes")?;
    for job_application in job_applications {
        writeln!(&mut file, "\"{}\",\"{}\",\"{}\",\"{}\",\"{:02}/{:02}/{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
            job_application.id,
            job_application.source.replace("\"","\"\""),
            job_application.company.replace("\"","\"\""),
            job_application.job_title.replace("\"","\"\""),
            job_application.application_date.month() as u8,
            job_application.application_date.day(),
            job_application.application_date.year(),
            job_application.time_investment.map_or("".to_string(),
                |t| format!("{:02}:{:02}", t.whole_minutes(), t.whole_seconds() % 60)
            ),
            match job_application.automated_response {
                true => "Yes",
                false => "No",
            },
            job_application.human_response,
            job_application.human_response_date.map_or("".to_string(),
                |d| format!("{:02}/{:02}/{}", d.month() as u8, d.day(), d.year())
            ),
            job_application.human_response_date
                .map_or("".to_owned(), |resp_date: Date| {
                    let duration_between_dates = resp_date - job_application.application_date;
                    duration_between_dates.whole_days().to_string()
                }),
            job_application.application_website.map_or("".to_string(), |s| s.replace("\"","\"\"")),
            job_application.notes.map_or("".to_string(), |s| s.replace("\"","\"\"")),
        )?;
    }
    // Not sure if flushing is necessary, but it doesn't hurt
    file.flush()?;

    // Open whatever is used to open CSV
    opener::open(file.path())?;

    Ok(())
}
