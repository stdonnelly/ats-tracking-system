use std::{
    convert::Infallible,
    fmt::Display,
    io::{self, stdin, stdout, Write},
    path::Path,
};

use mysql::prelude::Queryable;
use time::{macros::format_description, Date, Duration};

use crate::repository::{
    job_application_model::{
        HumanResponse, JobApplication, JobApplicationField, PartialJobApplication,
    },
    job_application_repository::{
        delete_job_application, get_job_application_by_id, get_job_applications,
        get_pending_job_applications, insert_job_application, search_job_applications,
        update_human_response, update_job_application,
    },
};

use super::shell_option::{ReadType, ShellOption, UpdateType};

macro_rules! input_optional {
    ($partial_application:ident, $prompt:literal, $parser:ident, $field_variant:tt) => {
        input::<Option<_>, _, _>(
            concat!($prompt, "\nLeave blank to leave unchanged:"),
            $parser,
        )?
        .map(|o| {
            $partial_application
                .0
                .push(JobApplicationField::$field_variant(o))
        });
    };
    // Not sure if it's possible to just use the same template for both, since it's the same
    ($partial_application:ident, $prompt:literal, $parser:expr, $field_variant:tt) => {
        input::<Option<_>, _, _>(
            concat!($prompt, "\nLeave blank to leave unchanged:"),
            $parser,
        )?
        .map(|o| {
            $partial_application
                .0
                .push(JobApplicationField::$field_variant(o))
        });
    };
}

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

    while keep_looping {
        print!("ats tracking> ");
        stdout().flush().unwrap();
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
    }

    Ok(())
}

fn help() -> Result<(), Box<dyn std::error::Error>> {
    print!(
        "ATS Tracking System

Available commands:
  help | h
  exit | quit
  create | new
  read [all] | pending | <id> | search <search_query>
  search <search_query>
    ^shorthand for read search <search_query>
  (update | edit) (response | other) <id>
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
    let human_response: HumanResponse;
    let human_response_date: Option<Date>;
    let application_website: Option<String>;
    let notes: Option<String>;

    // This will be used by multiple inputs
    let wrap_ok = |s: &str| Result::<_, Infallible>::Ok(s.to_owned());

    // Initialize the fields
    source = input("Source (job board, referral, etc):", wrap_ok)?;
    company = input("Company:", wrap_ok)?;
    job_title = input("Job Title:", wrap_ok)?;
    application_date = input(
        "Application date (leave blank for today) (mm/dd/yyyy):",
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
    human_response = input(
        "Response sent by a human\nEnter r for rejection, i for interview request, or leave blank for none:",
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
            "Response date (leave blank for today) (mm/dd/yyyy):",
            parse_date,
        )?);
    }
    application_website = Some(input(
        "Application website (if applied using the company website):",
        wrap_ok,
    )?)
    .filter(|s| !s.is_empty());
    let notes_first_line = Some(input("Notes:", wrap_ok)?).filter(|s| !s.is_empty());

    // Support multiline notes
    notes = match notes_first_line {
        Some(note) if note.starts_with('`') => {
            let mut notes = String::new();
            let mut note_line = note[1..].to_owned();
            // If first notes line starts with a backtick, check until the next backtick
            loop {
                if let Some(first_backtick) = note_line.find('`') {
                    // If this line contains a bactick
                    notes += &note_line[..first_backtick];
                    break;
                } else {
                    // Append this line
                    notes += &note_line;
                    notes += "\n";
                    // And keep looking
                    note_line = input("\\`bquote>", wrap_ok)?;
                }
            }
            Some(notes)
        }
        _ => notes_first_line,
    };

    // Construct the new application.
    let new_application = JobApplication {
        id: 0,
        source,
        company,
        job_title,
        application_date,
        time_investment,
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
        ReadType::Search(query) => search_job_applications(conn, &query)?,
        ReadType::One(id) => get_job_application_by_id(conn, id)?.map_or(Vec::new(), |a| vec![a]),
    };

    match applications.len() {
        0 => Err(Box::<dyn std::error::Error>::from(
            "No job application found",
        )),
        1 => {
            // This should never panic, we just verified there is exactly one job application
            print_job_application_to_terminal(applications.get(0).unwrap());
            Ok(())
        }
        _ => print_table(applications, temp_dir),
    }
}

fn print_job_application_to_terminal(ja: &JobApplication) {
    println!("One job application found:");
    println!(
        "ID: {}
Source: {}
Company: {}
Job title: {}
Application date: {:02}/{:02}/{}
Time Taken to complete application: {}
Response from a human: {}
Human response date: {}
Response time (days): {}
Application website: {}
Notes: {}",
        ja.id,
        ja.source.replace("\"", "\"\""),
        ja.company.replace("\"", "\"\""),
        ja.job_title.replace("\"", "\"\""),
        ja.application_date.month() as u8,
        ja.application_date.day(),
        ja.application_date.year(),
        ja.time_investment.map_or("".to_string(), |t| format!(
            "{:02}:{:02}",
            t.whole_minutes(),
            t.whole_seconds() % 60
        )),
        ja.human_response,
        ja.human_response_date.map_or("".to_string(), |d| format!(
            "{:02}/{:02}/{}",
            d.month() as u8,
            d.day(),
            d.year()
        )),
        ja.human_response_date
            .map_or("".to_owned(), |resp_date: Date| {
                let duration_between_dates = resp_date - ja.application_date;
                duration_between_dates.whole_days().to_string()
            }),
        ja.application_website
            .as_deref()
            .map_or("".to_string(), |s| s.replace("\"", "\"\"")),
        ja.notes
            .as_deref()
            .map_or("".to_string(), |s| s.replace("\"", "\"\"")),
    );
}

/// Determine the update type and call the appropriate function
fn update<C: Queryable>(
    conn: &mut C,
    update_type: UpdateType,
    id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Deal with warnings for now
    match update_type {
        UpdateType::HumanResponse => update_human_response_command(conn, id),
        UpdateType::Other => update_other_command(conn, id),
    }
}

/// Ask the user for the human response and update it
fn update_human_response_command<C: Queryable>(
    conn: &mut C,
    id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let human_response: HumanResponse;
    let human_response_date: Option<Date>;

    // Get the parameters
    human_response = input(
        "Response sent by a human\nEnter r for rejection, i for interview request, or leave blank for none:",
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
            "Response date (leave blank for today) (mm/dd/yyyy):",
            parse_date,
        )?);
    }

    update_human_response(conn, id, human_response, human_response_date)
        // Box the error, if any
        .map_err(Box::<dyn std::error::Error>::from)
}

/// Ask the user what to update and update it
fn update_other_command<C: Queryable>(
    conn: &mut C,
    id: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // This will be used by multiple inputs
    let wrap_ok = |s: &str| {
        Result::<_, Infallible>::Ok(if s.is_empty() {
            None
        } else {
            Some(s.to_owned())
        })
    };

    // Initialize the fields
    let mut partial_application = PartialJobApplication(Vec::new());
    input_optional!(
        partial_application,
        "Source (job board, referral, etc)",
        wrap_ok,
        Source
    );
    input_optional!(partial_application, "Company", wrap_ok, Company);
    input_optional!(partial_application, "Job Title", wrap_ok, JobTitle);
    input_optional!(
        partial_application,
        "Application date (mm/dd/yyyy)",
        parse_date_optional,
        ApplicationDate
    );
    input_optional!(
        partial_application,
        "Time taken to complete application (enter 'remove' to remove) (mm:ss)",
        |s: &str| {
            match s {
                // If empty, just ignore this
                s if s.is_empty() => Ok(None),
                // If this is the word "remove", Some(None) will result in the element TimeInvestment(None), which will make the entry NULL
                "remove" => Ok(Some(None)),
                // For anything else, parse the time
                s => {
                    if let Some((minutes_str, seconds_str)) = s.split_once(':') {
                        let minutes = match minutes_str.parse::<i32>() {
                            Ok(i) => i,
                            Err(e) => return Err(e.to_string()),
                        };
                        let seconds = match seconds_str.parse::<i32>() {
                            Ok(i) => i,
                            Err(e) => return Err(e.to_string()),
                        };
                        Ok(Some(Some(Duration::seconds(
                            (minutes * 60 + seconds) as i64,
                        ))))
                    } else {
                        Err("No colon found".to_owned())
                    }
                }
            }
        },
        TimeInvestment
    );
    input_optional!(
        partial_application,
        "Response sent by a human\nEnter r for rejection, i for interview request, or 'remove' for none",
        |s| {
            if s == "remove" {
                Ok(Some(HumanResponse::None))
            } else if s.starts_with(&['r', 'R']) {
                Ok(Some(HumanResponse::Rejection))
            } else if s.starts_with(&['i', 'I']) {
                Ok(Some(HumanResponse::InterviewRequest))
            } else if s.is_empty() {
                Ok(None)
            } else {
                Err("Unknown response")
            }
        },
        HumanResponse
        );
    // Only prompt if human response is not null
    input_optional!(
        partial_application,
        "Response date (enter 'remove' to remove) (mm/dd/yyyy)",
        |s: &str| {
            if s == "remove" {
                // If the input is "remove", remove it by adding HumanResponseDate(None) to the vector
                Ok(Some(None))
            } else {
                // If the result is Ok, check if it's blank
                // If blank, just return Ok(None) so the item does not get added to the partial application record (i.e. Ok(None)->Ok(None))
                // If successful parse, return Ok(Some(*parsed*))
                parse_date_optional(s).map(|o| o.map(Option::from))
            }
        },
        HumanResponseDate
    );
    input_optional!(
        partial_application,
        "Application website (if applied using the company website) (enter 'remove' to remove)",
        |s: &str| {
            if s == "remove" {
                Ok(Some(None))
            } else {
                // If the result is Ok, check if it's blank
                // If blank, just return Ok(None) so the item does not get added to the partial application record (i.e. Ok(None)->Ok(None))
                // If successful parse, return Ok(Some(*parsed*))
                wrap_ok(s).map(|o| o.map(Option::from))
            }
        },
        ApplicationWebsite
    );

    // Not using the macro for this one because it can be multiline and nothing else should behave anything like this
    let first_notes_line = input("Notes\nLeave blank to leave unchanged: ", |s: &str| {
        if s == "remove" {
            Ok(Some(None))
        } else {
            // If the result is Ok, check if it's blank
            // If blank, just return Ok(None) so the item does not get added to the partial application record (i.e. Ok(None)->Ok(None))
            // If successful parse, return Ok(Some(*parsed*))
            wrap_ok(s).map(|o| o.map(Option::from))
        }
    })?;

    // Handle multi line notes
    match first_notes_line {
        // Match if the first line exists and starts with a backtick
        Some(Some(note)) if note.starts_with('`') => {
            let mut notes = String::new();
            let mut note_line = note[1..].to_owned();
            // If first notes line starts with a backtick, check until the next backtick
            loop {
                if let Some(first_backtick) = note_line.find('`') {
                    // If this line contains a bactick
                    notes += &note_line[..first_backtick];
                    break;
                } else {
                    // Append this line
                    notes += &note_line;
                    notes += "\n";
                    // And keep looking
                    // unwrap_or("") because this `wrap_ok` returns `None` if empty, unlike the version of `wrap_ok` in `create()`
                    note_line = input("\\`bquote>", wrap_ok)?.unwrap_or("".to_owned());
                }
            }
            partial_application
                .0
                .push(JobApplicationField::Notes(Some(notes)));
        }
        // Otherwise, just push the line by itself
        // This includes Some(None), which should cause removal
        Some(o) => partial_application.0.push(JobApplicationField::Notes(o)),
        // If None is returned, do nothing
        None => (),
    }

    // Make sure at least one change was made
    if partial_application.0.is_empty() {
        Err(Box::<dyn std::error::Error>::from("No changes made"))
    } else {
        // Add the ID of the job application to modify
        partial_application.0.push(JobApplicationField::Id(id));
        // For confirmation, print the returned job application
        update_job_application(conn, partial_application)?;
        // print_job_application_to_terminal(&new_job_application);

        Ok(())
    }
}

fn delete<C: Queryable>(conn: &mut C, id: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the job application we are trying to delete actually exists
    if let Some(job_application) = get_job_application_by_id(conn, id)? {
        // Print the job application so the user knows exactly what they are deleting
        print_job_application_to_terminal(&job_application);

        // Confirm delete
        if input(
            "Are you sure you want to delete this job application? [y/N]:",
            |s| Result::<bool, Infallible>::Ok(s.starts_with(&['y', 'Y'])), // Only do it if y, Y, or something that starts with y
        )? {
            delete_job_application(conn, id).map_err(Box::<dyn std::error::Error>::from)?;
            println!("Successfully deleted job application {id}");
        } else {
            println!("Aborting delete");
        }
        Ok(())
    } else {
        Err(Box::<dyn std::error::Error>::from(
            "No job application found",
        ))
    }
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
        // For some reason Rust forces me to create this intermediate variable line_string
        let line_string = line?;
        let trimmed = line_string.trim();
        if trimmed == "abort" {
            return Err(io::Error::other("Operation aborted"));
        }
        match parse(trimmed) {
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
    writeln!(&mut file, "ID,Source,Company,Job Title,Application Date,Time Taken,Human Response,Date,Days to Respond,Website,Notes")?;
    for job_application in job_applications {
        writeln!(&mut file, "\"{}\",\"{}\",\"{}\",\"{}\",\"{:02}/{:02}/{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
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

/// Parse a date string into a date
///
/// Used for input()
fn parse_date(s: &str) -> Result<Date, time::error::Parse> {
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
}

/// Parse a string into an optional date
///
/// If the string is "", return Ok(None), otherwise try to parse the string and return Ok(Some(*parsed*))
fn parse_date_optional(s: &str) -> Result<Option<Date>, time::error::Parse> {
    if !s.is_empty() {
        // If a date was given, try to parse it
        Ok(Some(Date::parse(
            s,
            format_description!("[month repr:numerical]/[day]/[year]"),
        )?))
    } else {
        // The string being empty is fine, just use today
        Ok(None)
    }
}
