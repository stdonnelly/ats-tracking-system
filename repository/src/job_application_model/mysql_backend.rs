use crate::job_application_model::{HumanResponse, JobApplication, JobApplicationField, PartialJobApplication};

use mysql::{
    params,
    prelude::{FromValue, ToValue},
    Params, Value,
};

use std::collections::HashMap;

impl Into<Params> for &JobApplication {
    fn into(self) -> Params {
        params! {
            "id" => &self.id,
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
