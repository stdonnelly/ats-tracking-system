use std::{error::Error, fmt::Display};

use time::Date;

use super::job_application_model::{HumanResponse, JobApplication, PartialJobApplication};

/// Implementation using a mysql backend
#[cfg(feature = "mysql")]
mod mysql_backend;

/// Implementation with an sqlite backend
#[cfg(not(feature = "mysql"))]
mod sqlite_backend;

/// Abstract representation of some database connection
///
/// This exists to uncouple dependents from the database implementation
pub trait JobApplicationRepository {
    /// The error that may be returned from CRUD operations
    type Error: Error + Display + 'static;

    /// Get all job applications
    fn get_job_applications(&mut self) -> Result<Vec<JobApplication>, Self::Error>;

    /// Get all job applications where `human_response == None`
    fn get_pending_job_applications(&mut self) -> Result<Vec<JobApplication>, Self::Error> {
        self.search_by_human_response(HumanResponse::None)
    }

    /// Get the job application matching the specified `id`
    fn get_job_application_by_id(&mut self, id: i32)
        -> Result<Option<JobApplication>, Self::Error>;

    /// Get all job application where source, company, or job_title contains `query`. Case insensitive.
    fn search_job_applications(&mut self, query: &str) -> Result<Vec<JobApplication>, Self::Error>;

    /// Get all job applications with a certain human response
    fn search_by_human_response(
        &mut self,
        human_response: HumanResponse,
    ) -> Result<Vec<JobApplication>, Self::Error>;

    /// Get all job applications that matches a given human response AND a given search query
    fn search_by_query_and_human_response(
        &mut self,
        query: &str,
        human_response: HumanResponse,
    ) -> Result<Vec<JobApplication>, Self::Error>;

    /// Insert a new job application, returning the new application with generated `id` and `application_date`.
    ///
    /// `id` and `application_date` are automatically generated by the next available id and the current date, respectively.
    fn insert_job_application(
        &mut self,
        application: &JobApplication,
    ) -> Result<JobApplication, Self::Error>;

    /// Update the human response of a job application
    ///
    /// `human_response_date` is optional. If `None`, the date is generated as today.
    fn update_human_response(
        &mut self,
        id: i32,
        human_response: HumanResponse,
        human_response_date: Option<Date>,
    ) -> Result<(), Self::Error>;

    /// Update non-id fields of a job application
    ///
    /// Finds a job application using the id of `application` and replaces all other fields with the data contained in `application`.
    /// In the event there is no job application with a matching id, the database will remain unaffected and `Ok(())` will be returned.
    fn update_job_application(&mut self, application: &JobApplication) -> Result<(), Self::Error>;

    /// Update a job application, returning the updated application.
    ///
    /// `partial_application` must contain one [JobApplicationField::Id] element or a runtime error will occur.
    /// In the event there is no job application with a matching id, the database will remain unaffected and `Ok(())` will be returned.
    fn update_job_application_partial(
        &mut self,
        partial_application: PartialJobApplication,
    ) -> Result<(), Self::Error>;

    /// Delete the job application with the specified `id`
    fn delete_job_application(&mut self, id: i32) -> Result<(), Self::Error>;
}
