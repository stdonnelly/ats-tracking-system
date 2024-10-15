use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use mysql::{params, Params, Value};
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
    pub(crate) fn name(&self) -> String {
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
pub struct PartialJobApplication(pub Vec<JobApplicationField>);

impl Into<Params> for PartialJobApplication {
    fn into(self) -> Params {
        let mut params_map: HashMap<Vec<u8>, Value> = HashMap::with_capacity(self.0.len());
        for field in self.0 {
            params_map.insert(field.name().as_bytes().to_vec(), Into::<Value>::into(field));
        }
        Params::Named(params_map)
    }
}
