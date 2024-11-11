use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use mysql::{
    params,
    prelude::{FromRow, FromValue, ToValue},
    Params, Value,
};
use time::{Date, Duration};

/// A row in the job application table
#[derive(Debug, Clone, FromRow)]
#[mysql(table_name = "job_applications")]
pub struct JobApplication {
    pub id: i32,
    pub source: String,
    pub company: String,
    pub job_title: String,
    pub application_date: Date,
    pub time_investment: Option<Duration>,
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
            "human_response" => &self.human_response,
            "human_response_date" => &self.human_response_date,
            "application_website" => &self.application_website,
            "notes" => &self.notes,
        }
    }
}

/// Enum to hold possible human responses
#[derive(Debug, Clone, Copy, Default)]
pub enum HumanResponse {
    #[default]
    None,
    Rejection,
    InterviewRequest,
}

impl Display for HumanResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "No response yet",
            Self::Rejection => "Rejection",
            Self::InterviewRequest => "Interview request",
        })
    }
}

impl TryFrom<&str> for HumanResponse {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim().to_lowercase().as_str() {
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

impl ToValue for HumanResponse {
    fn to_value(&self) -> Value {
        match self {
            HumanResponse::None => "N",
            HumanResponse::Rejection => "R",
            HumanResponse::InterviewRequest => "I",
        }
        .to_value()
    }
}

impl FromValue for HumanResponse {
    // All we need to do is specify an intermediate.
    // The default implementation automatically converts `Value` -> `String` -> `HumanResponse`
    type Intermediate = String;
}

/// Field in a JobApplication to allow the creation of partial job applications
pub enum JobApplicationField {
    Id(i32),
    Source(String),
    Company(String),
    JobTitle(String),
    ApplicationDate(Date),
    TimeInvestment(Option<Duration>),
    HumanResponse(HumanResponse),
    HumanResponseDate(Option<Date>),
    ApplicationWebsite(Option<String>),
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

impl ToValue for JobApplicationField {
    fn to_value(&self) -> Value {
        match self {
            JobApplicationField::Id(o) => o.to_value(),
            JobApplicationField::Source(o) => o.to_value(),
            JobApplicationField::Company(o) => o.to_value(),
            JobApplicationField::JobTitle(o) => o.to_value(),
            JobApplicationField::ApplicationDate(o) => o.to_value(),
            JobApplicationField::TimeInvestment(o) => o.to_value(),
            JobApplicationField::HumanResponse(o) => o.to_value(),
            JobApplicationField::HumanResponseDate(o) => o.to_value(),
            JobApplicationField::ApplicationWebsite(o) => o.to_value(),
            JobApplicationField::Notes(o) => o.to_value(),
        }
    }
}

/// Newtype to allow impl Into<Params>
pub struct PartialJobApplication(pub Vec<JobApplicationField>);

impl Into<Params> for PartialJobApplication {
    fn into(self) -> Params {
        let mut params_map: HashMap<Vec<u8>, Value> = HashMap::with_capacity(self.0.len());
        for field in self.0 {
            params_map.insert(field.name().as_bytes().to_vec(), field.to_value());
        }
        Params::Named(params_map)
    }
}
