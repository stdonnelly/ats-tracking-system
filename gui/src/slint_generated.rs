//! Encapsulate slint generated objects to avoid slint `Date` conflict with `time::Date`
//!
//! This also implements `From<>` for some objects

use repository::job_application_model::{HumanResponse, JobApplication};
use time::{error::ComponentRange, ext::NumericalDuration as _, Month};

// rust-analyzer sometimes doesn't like the `include!` macro. Use `cargo check` for a more accurate check.
// Slint macro to include all generated code for the Slint UI
// This automatically applies `pub use` to all necessary objects as well.
slint::include_modules!();

impl TryFrom<JobApplicationView> for JobApplication {
    type Error = ComponentRange;

    fn try_from(value: JobApplicationView) -> Result<Self, Self::Error> {
        Ok(Self {
            // These are simple
            id: value.id,
            source: value.source.into(),
            company: value.company.into(),
            job_title: value.job_title.into(),
            application_date: value.application_date.try_into()?,
            time_investment: Some(value.time_investment)
                .filter(|i| *i != 0)
                .map(|i| (i as i64).seconds()),
            human_response: value.human_response.into(),
            human_response_date: match value.human_response {
                HumanResponseView::None => None,
                _ => Some(value.human_response_date.try_into()?),
            },
            // We want both of these as options.
            // An empty string will be converted to `Option::None`
            application_website: Some(value.application_website)
                .filter(|s| !s.is_empty())
                .map(Into::into),
            notes: Some(value.notes).filter(|s| !s.is_empty()).map(Into::into),
        })
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
                .map(|t| t.whole_seconds() as i32)
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

impl From<HumanResponse> for HumanResponseView {
    fn from(value: HumanResponse) -> Self {
        match value {
            HumanResponse::None => Self::None,
            HumanResponse::Rejection => Self::Rejection,
            HumanResponse::InterviewRequest => Self::InterviewRequest,
        }
    }
}

impl From<HumanResponseView> for HumanResponse {
    fn from(value: HumanResponseView) -> Self {
        match value {
            HumanResponseView::None => Self::None,
            HumanResponseView::Rejection => Self::Rejection,
            HumanResponseView::InterviewRequest => Self::InterviewRequest,
        }
    }
}
