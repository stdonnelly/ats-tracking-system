use std::fmt::{Debug, Display};

use mysql::{params, prelude::Queryable};
use time::{Date, Duration};

/// A row in the job application table
#[derive(Debug, Clone)]
pub struct JobApplication {
    pub id: i32,
    pub source: String,
    pub company: String,
    pub job_title: String,
    pub application_date: Date,
    pub time_investment: Option<Duration>,
    pub automated_response: bool,
    pub human_response: HumanResponse,
    pub human_response_date: Option<Date>,
    pub application_website: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum HumanResponse {
    None,
    Rejection,
    InterviewRequest,
}

impl Display for HumanResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "No response yet",
            Self::Rejection => "Rejection",
            Self::InterviewRequest => "Interview Request",
        })
    }
}

impl TryFrom<&str> for HumanResponse {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "interview request" => Ok(HumanResponse::InterviewRequest),
            "rejection" => Ok(HumanResponse::Rejection),
            "" => Ok(HumanResponse::None),
            _ => Err(())
        }
    }
}

impl From<HumanResponse> for Option<&str> {
    fn from(value: HumanResponse) -> Self {
        match value {
            HumanResponse::None => None,
            HumanResponse::Rejection => Some("Rejection"),
            HumanResponse::InterviewRequest => Some("Interview Request"),
        }
    }
}

/// Get all job applications
pub fn get_job_applications<C: Queryable>(
    conn: &mut C,
) -> Result<Vec<JobApplication>, mysql::Error> {
    conn.query_map(
        "SELECT id, source, company, job_title, application_date, time_investment, automated_response, human_response, human_response_date, application_website, notes
        FROM job_applications",
        map_row
    )
}

/// Get all job applications where `human_response == None`
pub fn get_pending_job_applications<C: Queryable>(
    conn: &mut C,
) -> Result<Vec<JobApplication>, mysql::Error> {
    conn.query_map(
        "SELECT id, source, company, job_title, application_date, time_investment, automated_response, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE human_response IS NULL",
        map_row
    )
}

/// Insert a new job application, returning the new application with generated `id` and `application_date`.
///
/// `id` and `application_date` are automatically generated by the next available id and the current date, respectively.
pub fn insert_job_application<C: Queryable>(
    conn: &mut C,
    application: &JobApplication,
) -> Result<JobApplication, mysql::Error> {
    let new_id = conn.exec_first::<i32,_,_>(
        "INSERT INTO job_applications (source, company, job_title, application_date, time_investment, automated_response, human_response, human_response_date, application_website, notes)
        VALUES (:source, :company, :job_title, :application_date, :time_investment, :automated_response, :human_response, :human_response_date, :application_website, :notes)
        RETURNING id",
        params! {
            "source" => &application.source,
            "company" => &application.company,
            "job_title" => &application.job_title,
            "application_date" => &application.application_date,
            "time_investment" => &application.time_investment,
            "automated_response" => &application.automated_response,
            "human_response" => Option::<&str>::from((&application.human_response).to_owned()),
            "human_response_date" => &application.human_response_date,
            "application_website" => &application.application_website,
            "notes" => &application.notes,
        }
    )?;

    Ok(JobApplication {
        id: new_id.unwrap_or_default(),
        ..application.clone()
    })
}

/// Update the human response of a job application
/// 
/// `human_response_date` is optional. If `None`, the date is generated by mysql `NOW()` function
pub fn update_human_response<C: Queryable>(
    conn: &mut C,
    id: i32,
    human_response: HumanResponse,
    human_response_date: Option<Date>,
) -> Result<(), mysql::Error> {
    conn.exec_drop(
        "UPDATE job_applications
        SET human_response = :human_response, human_response_date = COALESCE(:human_response_date, NOW())
        WHERE id = :id",
        params! {
            "id" => id,
            "human_response" => Option::<&str>::from(human_response),
            "human_response_date" => human_response_date
        }
    )?;

    Ok(())
}

/// Update a job application, returning the updated application.
///
/// `id` is used to determine what application to overwrite.
/// If there is no application with that id, nothing will be changed in the database and an error will be returned
pub fn update_job_application<C: Queryable>(
    conn: &mut C,
    application: JobApplication,
) -> Result<JobApplication, Box<dyn std::error::Error>> {
    todo!()
}

fn map_row(
    (
        id,
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
    ): (
        i32,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<Date>,
        Option<Duration>,
        Option<String>,
        Option<String>,
        Option<Date>,
        Option<String>,
        Option<String>,
    ),
) -> JobApplication {
    JobApplication {
        id,
        source: source.unwrap_or("".to_string()),
        company: company.unwrap_or("".to_string()),
        job_title: job_title.unwrap_or("".to_string()),
        application_date: application_date.unwrap_or(Date::from_ordinal_date(2000, 1).unwrap()),
        time_investment,
        automated_response: {
            if let Some(e) = automated_response {
                e.as_str() == "Y"
            } else {
                false
            }
        },
        human_response: {
            if let Some(human_response_unwrapped) = human_response {
                HumanResponse::try_from(human_response_unwrapped.as_str())
                    .unwrap_or(HumanResponse::None)
            } else {
                HumanResponse::None
            }
        },
        human_response_date,
        application_website,
        notes,
    }
}
