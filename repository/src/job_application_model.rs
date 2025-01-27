use std::fmt::{Debug, Display};

use time::{Date, Duration};

#[cfg(feature = "mysql")]
use mysql::prelude::FromRow;

/// Implementation using a mysql backend
#[cfg(feature = "mysql")]
mod mysql_backend;

/// Implementation with an sqlite backend
#[cfg(not(feature = "mysql"))]
mod sqlite_backend;

/// A row in the job application table
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "mysql", derive(FromRow))]
#[cfg_attr(feature = "mysql", mysql(table_name = "job_applications"))]
pub struct JobApplication {
    /// The table primary key
    pub id: i32,
    /// LinkedIn, Indeed, referral, etc
    pub source: String,
    /// The company that is hiring
    pub company: String,
    /// The job title
    pub job_title: String,
    /// When the user initially sent an application
    pub application_date: Date,
    /// The amount of time the user spent filling out the application
    pub time_investment: Option<Duration>,
    /// The response that was given, if the employer has responded
    pub human_response: HumanResponse,
    /// The date that the above response was given
    pub human_response_date: Option<Date>,
    /// A URL for the application website, if applicable (i.e. not easy apply)
    pub application_website: Option<String>,
    /// Notes on anything notable about the application process or company
    pub notes: Option<String>,
}

/// Enum to hold possible human responses
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HumanResponse {
    /// No response yet
    #[default]
    None,
    /// The company rejected the user's application
    Rejection,
    /// The company requested an interview
    InterviewRequest,
    /// Interviewed, then received a rejection
    InterviewedThenRejected,
    /// Interviewed, received a job offer
    JobOffer,
}

impl Display for HumanResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "No response yet",
            Self::Rejection => "Rejection",
            Self::InterviewRequest => "Interview request",
            Self::InterviewedThenRejected => "Interviewed, then rejected",
            Self::JobOffer => "Job offer",
        })
    }
}

impl TryFrom<&str> for HumanResponse {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
            "job offer" | "j" => Ok(HumanResponse::JobOffer),
            "interviewed then rejected" | "ir" => Ok(HumanResponse::InterviewedThenRejected),
            "interview request" | "i" => Ok(HumanResponse::InterviewRequest),
            "rejection" | "r" => Ok(HumanResponse::Rejection),
            "" | "n" => Ok(HumanResponse::None),
            _ => Err(()),
        }
    }
}

impl From<String> for HumanResponse {
    /// Tries to parse a `String` as a `HumanResponse`, if unrecognized, `HumanResponse::None` is returned
    fn from(value: String) -> Self {
        return TryFrom::<&str>::try_from(&value).unwrap_or_default();
    }
}

/// Field in a JobApplication to allow the creation of partial job applications
pub enum JobApplicationField {
    /// The table primary key
    Id(i32),
    /// LinkedIn, Indeed, referral, etc
    Source(String),
    /// The company that is hiring
    Company(String),
    /// The job title
    JobTitle(String),
    /// When the user initially sent an application
    ApplicationDate(Date),
    /// The amount of time the user spent filling out the application
    TimeInvestment(Option<Duration>),
    /// The response that was given, if the employer has responded
    HumanResponse(HumanResponse),
    /// The date that the above response was given
    HumanResponseDate(Option<Date>),
    /// A URL for the application website, if applicable (i.e. not easy apply)
    ApplicationWebsite(Option<String>),
    /// Notes on anything notable about the application process or company
    Notes(Option<String>),
}

impl JobApplicationField {
    pub(crate) fn name(&self) -> String {
        match self {
            JobApplicationField::Id(_) => "id",
            JobApplicationField::Source(_) => "source",
            JobApplicationField::Company(_) => "company",
            JobApplicationField::JobTitle(_) => "job_title",
            JobApplicationField::ApplicationDate(_) => "application_date",
            JobApplicationField::TimeInvestment(_) => "time_investment",
            JobApplicationField::HumanResponse(_) => "human_response",
            JobApplicationField::HumanResponseDate(_) => "human_response_date",
            JobApplicationField::ApplicationWebsite(_) => "application_website",
            JobApplicationField::Notes(_) => "notes",
        }
        .to_owned()
    }
}

/// Newtype to allow impl Into<Params>
pub struct PartialJobApplication(pub Vec<JobApplicationField>);
