//! Encapsulate slint generated objects to avoid slint `Date` conflict with `time::Date`
//!
//! This also implements `From<>` for some objects

use repository::job_application_model::{HumanResponse, JobApplication};
use time::{error::ComponentRange, Month};

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

impl TryFrom<Date> for time::Date {
    type Error = ComponentRange;

    fn try_from(value: Date) -> Result<Self, Self::Error> {
        // Safely convert month and day to u8
        // If outside the range of u8, this will just set them to the max, which will cause a ComponentRange error
        let month_u8 = if 0 <= value.month && value.month <= u8::MAX as i32 {
            value.month as u8
        } else {
            u8::MAX
        };
        let day_u8 = if 0 <= value.day && value.day <= u8::MAX as i32 {
            value.day as u8
        } else {
            u8::MAX
        };

        Month::try_from(month_u8)
            .and_then(|month| time::Date::from_calendar_date(value.year, month, day_u8))
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
            human_response_date: value
                .human_response_date
                .map(Into::into)
                .unwrap_or_default(),
            application_website: value
                .application_website
                .as_deref()
                .unwrap_or_default()
                .into(),
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
