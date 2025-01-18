use rusqlite::{named_params, Connection};
use time::Duration;

use crate::job_application_model::{JobApplication, PartialJobApplication};

use super::JobApplicationRepository;

impl JobApplicationRepository for Connection {
    type Error = rusqlite::Error;

    fn get_job_applications(&mut self) -> Result<Vec<JobApplication>, Self::Error> {
        // Create a prepared statement object
        let mut stmt = self.prepare_cached(
            "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes \
            FROM job_applications"
        )?;

        // Execute the statement, which will return an iterator to the mapped rows
        let row_iter = stmt.query_map((), |row| row.try_into())?;

        // Collect into a Vec using a for loop because we can't effectively use collect the iterator using `Iterator::collect`.
        // This is because we want the row mapping result to be passed to the caller on error using the `?` operator.
        let mut row_vec: Vec<JobApplication> = Vec::new();
        for row in row_iter {
            row_vec.push(row?);
        }

        // Return the collected Vec
        Ok(row_vec)
    }

    fn get_pending_job_applications(&mut self) -> Result<Vec<JobApplication>, Self::Error> {
        todo!()
    }

    fn get_job_application_by_id(
        &mut self,
        id: i32,
    ) -> Result<Option<JobApplication>, Self::Error> {
        todo!()
    }

    fn search_job_applications(&mut self, query: &str) -> Result<Vec<JobApplication>, Self::Error> {
        todo!()
    }

    fn insert_job_application(
        &mut self,
        application: &JobApplication,
    ) -> Result<JobApplication, Self::Error> {
        self.prepare_cached(
            "INSERT INTO job_applications (source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes) \
                VALUES (:source, :company, :job_title, :application_date, :time_investment, :human_response, :human_response_date, :application_website, :notes)")?
            // If the preparation succeeded, insert the row
            .insert(
                named_params! {
                    ":source": application.source,
                    ":company": application.company,
                    ":job_title": application.job_title,
                    ":application_date": application.application_date,
                    ":time_investment": application.time_investment.map(Duration::whole_seconds),
                    ":human_response": application.human_response,
                    ":human_response_date": application.human_response_date,
                    ":application_website": application.application_website,
                    ":notes": application.notes,
                }
            )
            // If that succeeded, return the new job application
            .map(|id| JobApplication {id: id as i32, ..application.clone()})
    }

    fn update_human_response(
        &mut self,
        id: i32,
        human_response: crate::job_application_model::HumanResponse,
        human_response_date: Option<time::Date>,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn update_job_application(&mut self, application: &JobApplication) -> Result<(), Self::Error> {
        todo!()
    }

    fn update_job_application_partial(
        &mut self,
        partial_application: PartialJobApplication,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn delete_job_application(&mut self, id: i32) -> Result<(), Self::Error> {
        todo!()
    }
}
