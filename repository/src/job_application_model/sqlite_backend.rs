use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, ValueRef},
    Row, ToSql,
};
use time::{ext::NumericalDuration, Duration};

use super::{HumanResponse, JobApplication, JobApplicationField, PartialJobApplication};

impl TryFrom<&Row<'_>> for JobApplication {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(JobApplication {
            id: value.get("id")?,
            source: value.get("source")?,
            company: value.get("company")?,
            job_title: value.get("job_title")?,
            application_date: value.get("application_date")?,
            time_investment: value
                .get::<&str, Option<i64>>("time_investment")?
                .map(NumericalDuration::seconds),
            human_response: value.get("human_response")?,
            human_response_date: value.get("human_response_date")?,
            application_website: value.get("application_website")?,
            notes: value.get("notes")?,
        })
    }
}

impl ToSql for HumanResponse {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(match self {
            HumanResponse::None => "N",
            HumanResponse::Rejection => "R",
            HumanResponse::InterviewRequest => "I",
            HumanResponse::InterviewedThenRejected => "IR",
            HumanResponse::JobOffer => "J",
        }
        .into())
    }
}

impl FromSql for HumanResponse {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        // Get value as string
        value.as_str().and_then(|s| {
            // If the value can be mapped to `HumanResponse`, use that
            s.try_into().map_err(|_| {
                // If a failure occurred, give an error with a descriptive error message
                FromSqlError::Other(
                    format!("Unable to parse value '{s}' into a human response").into(),
                )
            })
        })
    }
}

impl From<JobApplicationField> for Box<dyn ToSql> {
    fn from(value: JobApplicationField) -> Self {
        match value {
            JobApplicationField::Id(value) => Box::new(value),
            JobApplicationField::Source(value)
            | JobApplicationField::Company(value)
            | JobApplicationField::JobTitle(value) => Box::new(value),
            JobApplicationField::ApplicationDate(value) => Box::new(value),
            JobApplicationField::TimeInvestment(value) => {
                Box::new(value.map(Duration::whole_seconds))
            }
            JobApplicationField::HumanResponse(value) => Box::new(value),
            JobApplicationField::HumanResponseDate(value) => Box::new(value),
            JobApplicationField::ApplicationWebsite(value) | JobApplicationField::Notes(value) => {
                Box::new(value)
            }
        }
    }
}

impl From<PartialJobApplication> for Vec<Box<dyn ToSql>> {
    fn from(value: PartialJobApplication) -> Self {
        value.0.into_iter().map(Into::into).collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure [ToSql] is implemented correctly for all variants of [HumanResponse]
    #[test]
    fn test_to_sql_human_response() {
        assert_eq!(
            ToSqlOutput::from("N"),
            HumanResponse::None.to_sql().unwrap(),
            "None -> N"
        );
        assert_eq!(
            ToSqlOutput::from("R"),
            HumanResponse::Rejection.to_sql().unwrap(),
            "Rejection -> R"
        );
        assert_eq!(
            ToSqlOutput::from("I"),
            HumanResponse::InterviewRequest.to_sql().unwrap(),
            "InterviewRequest -> I"
        );
        assert_eq!(
            ToSqlOutput::from("IR"),
            HumanResponse::InterviewedThenRejected.to_sql().unwrap(),
            "InterviewedThenRejected -> IR"
        );
        assert_eq!(
            ToSqlOutput::from("J"),
            HumanResponse::JobOffer.to_sql().unwrap(),
            "JobOffer -> J"
        );
    }

    /// Ensure [FromSql] is implemented correctly for [HumanResponse]
    #[test]
    fn test_from_sql_human_response() {
        // Normal cases
        assert_eq!(
            HumanResponse::None,
            HumanResponse::column_result("N".into()).unwrap(),
            "N -> None"
        );
        assert_eq!(
            HumanResponse::Rejection,
            HumanResponse::column_result("R".into()).unwrap(),
            "R -> Rejection"
        );
        assert_eq!(
            HumanResponse::InterviewRequest,
            HumanResponse::column_result("I".into()).unwrap(),
            "I -> InterviewRequest"
        );
        assert_eq!(
            HumanResponse::InterviewedThenRejected,
            HumanResponse::column_result("IR".into()).unwrap(),
            "IR -> InterviewedThenRejected"
        );
        assert_eq!(
            HumanResponse::JobOffer,
            HumanResponse::column_result("J".into()).unwrap(),
            "J -> JobOffer"
        );

        // Error cases
        assert_eq!(
            HumanResponse::None,
            HumanResponse::column_result("".into()).unwrap(),
            "Empty string should produce HumanResponse::None"
        );
        assert_eq!(
            FromSqlError::Other("Unable to parse value 'FOO' into a human response".into())
                .to_string(),
            HumanResponse::column_result("FOO".into())
                .unwrap_err()
                .to_string(),
            "Invalid human response should produce an error"
        );
        assert_eq!(
            FromSqlError::InvalidType,
            HumanResponse::column_result(ValueRef::Null).unwrap_err(),
            "NULL should produce an InvalidType error"
        );
    }
}
