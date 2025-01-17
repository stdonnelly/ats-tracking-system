use rusqlite::Connection;

use super::JobApplicationRepository;

impl JobApplicationRepository for Connection {
    type Error = rusqlite::Error;

    fn get_job_applications(
        &mut self,
    ) -> Result<Vec<crate::job_application_model::JobApplication>, Self::Error> {
        todo!()
    }

    fn get_pending_job_applications(
        &mut self,
    ) -> Result<Vec<crate::job_application_model::JobApplication>, Self::Error> {
        todo!()
    }

    fn get_job_application_by_id(
        &mut self,
        id: i32,
    ) -> Result<Option<crate::job_application_model::JobApplication>, Self::Error> {
        todo!()
    }

    fn search_job_applications(
        &mut self,
        query: &str,
    ) -> Result<Vec<crate::job_application_model::JobApplication>, Self::Error> {
        todo!()
    }

    fn insert_job_application(
        &mut self,
        application: &crate::job_application_model::JobApplication,
    ) -> Result<crate::job_application_model::JobApplication, Self::Error> {
        todo!()
    }

    fn update_human_response(
        &mut self,
        id: i32,
        human_response: crate::job_application_model::HumanResponse,
        human_response_date: Option<time::Date>,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn update_job_application(
        &mut self,
        application: &crate::job_application_model::JobApplication,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn update_job_application_partial(
        &mut self,
        partial_application: crate::job_application_model::PartialJobApplication,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn delete_job_application(&mut self, id: i32) -> Result<(), Self::Error> {
        todo!()
    }
}
