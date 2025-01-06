//! Encapsulate slint generated objects to avoid slint `Date` conflict with `time::Date`
//!
//! This also implements `From<>` for some objects

use repository::job_application_model::{HumanResponse, JobApplication};

// Slint macro to include all generated code for the Slint UI
// This automatically applies `pub use` to all necessary objects as well.
slint::include_modules!();

impl From<time::Date> for Date {
    fn from(value: time::Date) -> Self {
        Self {
            day: value.day() as i32,
            month: value.month() as i32,
            year: value.year(),
        }
    }
}

impl From<JobApplication> for JobApplicationView {
    fn from(value: JobApplication) -> Self {
        Self {
            id: value.id,
            source: value.source.into(),
            company: value.company.into(),
            job_title: value.job_title.into(),
            application_date: value.application_date.into(),
            time_investment: value
                .time_investment
                .map(|t| format!("{:02}:{:02}", t.whole_minutes(), t.whole_seconds() % 60).into())
                .unwrap_or_default(),
            human_response: value.human_response.into(),
            human_response_date: value.human_response_date.map(Into::into).unwrap_or_default(),
            days_to_respond: value
                .human_response_date
                .map(|resp_date: time::Date| {
                    let duration_between_dates = resp_date - value.application_date;
                    duration_between_dates.whole_days() as i32
                })
                .unwrap_or_default(),
            application_website: value.application_website.as_deref().unwrap_or_default().into(),
            notes: value.notes.as_deref().unwrap_or_default().into(),
        }
    }
}

impl From<HumanResponse> for HumanResponseView {
    fn from(value: HumanResponse) -> Self {
        match value {
            HumanResponse::None => Self::None,
            HumanResponse::Rejection => Self::Rejection,
            HumanResponse::InterviewRequest => Self::InterviewRequest,
        }
    }
}
