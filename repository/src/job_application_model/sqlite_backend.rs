use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, ValueRef},
    Row, ToSql,
};
use time::ext::NumericalDuration;

use super::{HumanResponse, JobApplication};

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
