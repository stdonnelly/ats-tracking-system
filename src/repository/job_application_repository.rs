use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use mysql::{params, prelude::Queryable, Params, Value};
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

impl Into<Params> for &JobApplication {
    fn into(self) -> Params {
        params! {
            "source" => &self.source,
            "company" => &self.company,
            "job_title" => &self.job_title,
            "application_date" => &self.application_date,
            "time_investment" => &self.time_investment,
            "automated_response" => &self.automated_response,
            "human_response" => &self.human_response,
            "human_response_date" => &self.human_response_date,
            "application_website" => &self.application_website,
            "notes" => &self.notes,
        }
    }
}

/// Enum to hold possible human responses
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
            _ => Err(()),
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

impl Into<Value> for HumanResponse {
    fn into(self) -> Value {
        Value::from(Option::<&str>::from((self).to_owned()))
    }
}

/// Field in a JobApplication to allow the creation of partial job applications
pub enum JobApplicationField {
    Id(i32),
    Source(String),
    Company(String),
    JobTitle(String),
    ApplicationDate(Date),
    TimeInvestment(Option<Duration>),
    AutomatedResponse(bool),
    HumanResponse(HumanResponse),
    HumanResponseDate(Option<Date>),
    ApplicationWebsite(Option<String>),
    Notes(Option<String>),
}

impl JobApplicationField {
    fn name(&self) -> String {
        match self {
            JobApplicationField::Id(_) => "id",
            JobApplicationField::Source(_) => "source",
            JobApplicationField::Company(_) => "company",
            JobApplicationField::JobTitle(_) => "job_title",
            JobApplicationField::ApplicationDate(_) => "application_date",
            JobApplicationField::TimeInvestment(_) => "time_investment",
            JobApplicationField::AutomatedResponse(_) => "automated_response",
            JobApplicationField::HumanResponse(_) => "human_response",
            JobApplicationField::HumanResponseDate(_) => "human_response_date",
            JobApplicationField::ApplicationWebsite(_) => "application_website",
            JobApplicationField::Notes(_) => "notes",
        }
        .to_owned()
    }
}

impl Into<Value> for JobApplicationField {
    fn into(self) -> Value {
        match self {
            JobApplicationField::Id(o) => Into::<Value>::into(o),
            JobApplicationField::Source(o) => Into::<Value>::into(o),
            JobApplicationField::Company(o) => Into::<Value>::into(o),
            JobApplicationField::JobTitle(o) => Into::<Value>::into(o),
            JobApplicationField::ApplicationDate(o) => Into::<Value>::into(o),
            JobApplicationField::TimeInvestment(o) => Into::<Value>::into(o),
            JobApplicationField::AutomatedResponse(o) => Into::<Value>::into(o),
            JobApplicationField::HumanResponse(o) => Into::<Value>::into(o),
            JobApplicationField::HumanResponseDate(o) => Into::<Value>::into(o),
            JobApplicationField::ApplicationWebsite(o) => Into::<Value>::into(o),
            JobApplicationField::Notes(o) => Into::<Value>::into(o),
        }
    }
}

/// Newtype to allow impl Into<Params>
pub struct PartialJobApplication(Vec<JobApplicationField>);

impl Into<Params> for PartialJobApplication {
    fn into(self) -> Params {
        let mut params_map: HashMap<Vec<u8>, Value> = HashMap::with_capacity(self.0.len());
        for field in self.0 {
            params_map.insert(field.name().as_bytes().to_vec(), Into::<Value>::into(field));
        }
        Params::Named(params_map)
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

pub fn get_job_application_by_id<C: Queryable>(
    conn: &mut C,
    id: i32,
) -> Result<Option<JobApplication>, mysql::Error> {
    conn.exec_first(
        "SELECT id, source, company, job_title, application_date, time_investment, automated_response, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE id = ?",
        (id,),
    )
    .map(|o| {o.map(map_row)})
}

pub fn search_job_applications<C: Queryable>(
    conn: &mut C,
    query: &str,
) -> Result<Vec<JobApplication>, mysql::Error> {
    // Add wildcards to the beginning and end of the query
    let query_with_wildcards = "%".to_owned() + &query.to_lowercase() + "%";
    conn.exec_map(
        "SELECT id, source, company, job_title, application_date, time_investment, automated_response, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE LOWER(source) LIKE :query
        OR LOWER(company) LIKE :query
        OR LOWER(job_title) LIKE :query",
        params! {"query" => query_with_wildcards},
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
        application
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
    partial_application: PartialJobApplication,
) -> Result<JobApplication, Box<dyn std::error::Error>> {
    let mut query_builder = "UPDATE job_applications SET ".to_owned();

    // Loop over all field names
    // Flag for if this is the first variable
    let mut is_first = true;
    for field in partial_application.0.iter() {
        if let JobApplicationField::Id(_) = field {
            // NO-OP: Id is special because we are using it in the WHERE clause instead of SET
        } else if is_first {
            // The first non-id value is special because of where the SET and commas are
            query_builder += &format!("SET {0} = :{0}", field.name());
            is_first = false
        } else {
            // Normal placement
            query_builder += &format!(",\n{0} = :{0}", field.name());
        }
    }

    // End with the WHERE clause
    query_builder += "\nWHERE id = :id
    RETURNING id, source, company, job_title, application_date, time_investment, automated_response, human_response, human_response_date, application_website, notes";

    conn.exec_first(query_builder, partial_application)?
        .map(map_row)
        .ok_or(Box::<dyn std::error::Error>::from(
            "No job application found",
        ))
}

/// Delete a job application from the database
///
/// Not sure if I actually want this function
pub fn delete_job_application<C: Queryable>(conn: &mut C, id: i32) -> Result<(), mysql::Error> {
    let _ = conn;
    let _ = id;
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
