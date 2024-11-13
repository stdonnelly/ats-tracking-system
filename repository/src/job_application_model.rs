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
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use time::{ext::NumericalDuration, Month};

    use super::*;

    /// Test Into<Params> for JobApplication when all fields are non-null
    #[test]
    fn test_into_params() {
        // Example job application
        let example_job_application = JobApplication {
            id: 12,
            source: "foo source".to_owned(),
            company: "foo company".to_owned(),
            job_title: "foo job".to_owned(),
            application_date: Date::from_calendar_date(2001, Month::February, 2).unwrap(),
            time_investment: Some(90.seconds()),
            human_response: HumanResponse::Rejection,
            human_response_date: Some(Date::from_calendar_date(2001, Month::February, 3).unwrap()),
            application_website: Some("foo website".to_owned()),
            notes: Some("foo notes".to_owned()),
        };

        // Convert using impl Into<Params> for &JobApplication
        let actual_params: Params = (&example_job_application).into();
        let expected_map = HashMap::from([
            // Into<Params> never applies id because MySQL auto increment handles that
            // (b"id".to_vec(), Value::Int(12)),
            (b"source".to_vec(), Value::Bytes(b"foo source".to_vec())),
            (b"company".to_vec(), Value::Bytes(b"foo company".to_vec())),
            (b"job_title".to_vec(), Value::Bytes(b"foo job".to_vec())),
            (
                b"application_date".to_vec(),
                Value::Date(2001, 2, 2, 0, 0, 0, 0),
            ),
            (
                b"time_investment".to_vec(),
                Value::Time(false, 0, 0, 1, 30, 0),
            ),
            (b"human_response".to_vec(), Value::Bytes(b"R".to_vec())),
            (
                b"human_response_date".to_vec(),
                Value::Date(2001, 2, 3, 0, 0, 0, 0),
            ),
            (
                b"application_website".to_vec(),
                Value::Bytes(b"foo website".to_vec()),
            ),
            (b"notes".to_vec(), Value::Bytes(b"foo notes".to_vec())),
        ]);

        // Ensure the params are named
        if let Params::Named(actual_map) = actual_params {
            // Assert there are 10 parameters so
            assert_eq!(
                actual_map, expected_map,
                "Actual and expected params differ"
            );
        } else {
            panic!(
                "params should be positional, actual params is {:?}",
                actual_params
            );
        }
    }

    /// Test Into<Params> for JobApplication when all nullable fields are nullable
    #[test]
    fn test_into_params_null() {
        // Example job application
        let example_job_application = JobApplication {
            id: 12,
            source: "foo source".to_owned(),
            company: "foo company".to_owned(),
            job_title: "foo job".to_owned(),
            application_date: Date::from_calendar_date(2001, Month::February, 2).unwrap(),
            time_investment: None,
            human_response: HumanResponse::None,
            human_response_date: None,
            application_website: None,
            notes: None,
        };

        // Convert using impl Into<Params> for &JobApplication
        let actual_params: Params = (&example_job_application).into();
        let expected_map = HashMap::from([
            // Into<Params> never applies id because MySQL auto increment handles that
            // (b"id".to_vec(), Value::Int(12)),
            (b"source".to_vec(), Value::Bytes(b"foo source".to_vec())),
            (b"company".to_vec(), Value::Bytes(b"foo company".to_vec())),
            (b"job_title".to_vec(), Value::Bytes(b"foo job".to_vec())),
            (
                b"application_date".to_vec(),
                Value::Date(2001, 2, 2, 0, 0, 0, 0),
            ),
            (b"time_investment".to_vec(), Value::NULL),
            (b"human_response".to_vec(), Value::Bytes(b"N".to_vec())),
            (b"human_response_date".to_vec(), Value::NULL),
            (b"application_website".to_vec(), Value::NULL),
            (b"notes".to_vec(), Value::NULL),
        ]);

        // Ensure the params are named
        if let Params::Named(actual_map) = actual_params {
            // Assert there are 10 parameters so
            assert_eq!(
                actual_map, expected_map,
                "Actual and expected params differ"
            );
        } else {
            panic!(
                "params should be positional, actual params is {:?}",
                actual_params
            );
        }
    }

    /// Test ToValue for HumanResponse
    #[test]
    fn test_to_value_human_response() {
        assert_eq!(
            HumanResponse::None.to_value(),
            Value::Bytes(b"N".to_vec()),
            "None -> N"
        );
        assert_eq!(
            HumanResponse::Rejection.to_value(),
            Value::Bytes(b"R".to_vec()),
            "Rejection -> R"
        );
        assert_eq!(
            HumanResponse::InterviewRequest.to_value(),
            Value::Bytes(b"I".to_vec()),
            "InterviewRequest -> I"
        );
    }

    // FromRow can't really be tested because `Row` fields are all private.

    // Test all possible values to HumanResponse
    #[test]
    fn test_from_value_human_response() {
        // Normal cases
        assert_eq!(
            HumanResponse::from_value(Value::Bytes(b"N".to_vec())),
            HumanResponse::None,
            "N -> None"
        );
        assert_eq!(
            HumanResponse::from_value(Value::Bytes(b"R".to_vec())),
            HumanResponse::Rejection,
            "R -> Rejection"
        );
        assert_eq!(
            HumanResponse::from_value(Value::Bytes(b"I".to_vec())),
            HumanResponse::InterviewRequest,
            "I -> InterviewRequest"
        );

        // Error case: using "" instead of NULL because intermediate String cannot be constructed from NULL
        assert_eq!(
            HumanResponse::from_value(Value::Bytes(b"".to_vec())),
            HumanResponse::None,
            "Unknown value -> None"
        );
    }

    /// Test Into<Params> for PartialJobApplication
    #[test]
    fn test_into_params_partial() {
        // This should be the same job application as test_into_params, but as a PartialJobApplication
        let example_job_application: PartialJobApplication = PartialJobApplication(vec![
            JobApplicationField::Source("foo source".to_owned()),
            JobApplicationField::Company("foo company".to_owned()),
            JobApplicationField::JobTitle("foo job".to_owned()),
            JobApplicationField::ApplicationDate(
                Date::from_calendar_date(2001, Month::February, 2).unwrap(),
            ),
            JobApplicationField::TimeInvestment(Some(90.seconds())),
            JobApplicationField::HumanResponse(HumanResponse::Rejection),
            JobApplicationField::HumanResponseDate(Some(
                Date::from_calendar_date(2001, Month::February, 3).unwrap(),
            )),
            JobApplicationField::ApplicationWebsite(Some("foo website".to_owned())),
            JobApplicationField::Notes(Some("foo notes".to_owned())),
        ]);

        // This is all the same as test_into_params
        // Convert using impl Into<Params> for PartialJobApplication
        let actual_params: Params = example_job_application.into();
        let expected_map = HashMap::from([
            (b"source".to_vec(), Value::Bytes(b"foo source".to_vec())),
            (b"company".to_vec(), Value::Bytes(b"foo company".to_vec())),
            (b"job_title".to_vec(), Value::Bytes(b"foo job".to_vec())),
            (
                b"application_date".to_vec(),
                Value::Date(2001, 2, 2, 0, 0, 0, 0),
            ),
            (
                b"time_investment".to_vec(),
                Value::Time(false, 0, 0, 1, 30, 0),
            ),
            (b"human_response".to_vec(), Value::Bytes(b"R".to_vec())),
            (
                b"human_response_date".to_vec(),
                Value::Date(2001, 2, 3, 0, 0, 0, 0),
            ),
            (
                b"application_website".to_vec(),
                Value::Bytes(b"foo website".to_vec()),
            ),
            (b"notes".to_vec(), Value::Bytes(b"foo notes".to_vec())),
        ]);

        // Ensure the params are named
        if let Params::Named(actual_map) = actual_params {
            // Assert there are 10 parameters so
            assert_eq!(
                actual_map, expected_map,
                "Actual and expected params differ"
            );
        } else {
            panic!(
                "params should be positional, actual params is {:?}",
                actual_params
            );
        }
    }
}
