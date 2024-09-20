use std::{env, io::Write, path::Path};

use dotenv::dotenv;
use mysql::{OptsBuilder, Pool, PooledConn};
use repository::job_application_repository::{get_job_applications, JobApplication};

mod repository;

fn main() {
    // Objects that should be owned by the main function
    dotenv().unwrap();
    let mut conn = get_conn().unwrap();
    let temp_dir = tempfile::TempDir::new().unwrap();

    let job_applications: Vec<JobApplication> = get_job_applications(&mut conn).unwrap();

    print_table(job_applications, temp_dir.path()).unwrap();
    println!("Press [ENTER] to exit");
    std::io::stdin().read_line(&mut String::new()).unwrap();
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
            match job_application.automated_response {
                true => "Yes",
                false => "No",
            },
            job_application.human_response,
            job_application.human_response_date.map_or("".to_string(),
                |d| format!("{:02}/{:02}/{}", d.month() as u8, d.day(), d.year())
            ),
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

/// Boilerplate for getting connection information from environment
fn get_conn() -> Result<PooledConn, mysql::Error> {
    let mut sql_opts_builder = OptsBuilder::new();
    if let Ok(host_name) = env::var("DB_HOST") {
        sql_opts_builder = sql_opts_builder.ip_or_hostname(Some(host_name));
    }
    if let Ok(port_str) = env::var("DB_PORT") {
        if let Ok(port_int) = port_str.parse::<u16>() {
            sql_opts_builder = sql_opts_builder.tcp_port(port_int);
        }
    }
    if let Ok(user) = env::var("DB_USER") {
        sql_opts_builder = sql_opts_builder.user(Some(user));
    }
    if let Ok(pass) = env::var("DB_PASSWORD") {
        sql_opts_builder = sql_opts_builder.pass(Some(pass));
    }
    if let Ok(db_name) = env::var("DB_DATABASE") {
        sql_opts_builder = sql_opts_builder.db_name(Some(db_name));
    }

    let pool = Pool::new(sql_opts_builder)?;
    pool.get_conn()
}
