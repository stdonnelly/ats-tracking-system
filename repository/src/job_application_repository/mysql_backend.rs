use mysql::{params, prelude::Queryable};

use crate::job_application_model::JobApplicationField;

use super::*;

impl<C> JobApplicationRepository for C
where
    C: Queryable,
{
    type Error = mysql::Error;

    fn get_job_applications(&mut self) -> Result<Vec<JobApplication>, mysql::Error> {
        self.query(
        "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes
        FROM job_applications"
    )
    }

    fn get_job_application_by_id(
        &mut self,
        id: i32,
    ) -> Result<Option<JobApplication>, mysql::Error> {
        self.exec_first(
        "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE id = ?",
        (id,),
    )
    }

    fn search_job_applications(
        &mut self,
        query: &str,
    ) -> Result<Vec<JobApplication>, mysql::Error> {
        // Add wildcards to the beginning and end of the query
        let query_with_wildcards = "%".to_owned() + &query.to_lowercase() + "%";
        self.exec(
        "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE LOWER(source) LIKE :query
        OR LOWER(company) LIKE :query
        OR LOWER(job_title) LIKE :query",
        params! {"query" => query_with_wildcards}
    )
    }

    fn search_by_human_response(
        &mut self,
        human_response: HumanResponse,
    ) -> Result<Vec<JobApplication>, mysql::Error> {
        self.exec(
        "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE human_response = :human_response",
        params! {"human_response" => &human_response}
    )
    }

    fn search_by_query_and_human_response(
        &mut self,
        query: &str,
        human_response: HumanResponse,
    ) -> Result<Vec<JobApplication>, mysql::Error> {
        // Add wildcards to the beginning and end of the query
        let query_with_wildcards = "%".to_owned() + &query.to_lowercase() + "%";
        self.exec(
        "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes
        FROM job_applications
        WHERE (
            LOWER(source) LIKE :query
            OR LOWER(company) LIKE :query
            OR LOWER(job_title) LIKE :query
        ) AND human_response = :human_response",
        params! {
            "query" => query_with_wildcards,
            "human_response" => &human_response
        }
    )
    }

    fn insert_job_application(
        &mut self,
        application: &JobApplication,
    ) -> Result<JobApplication, mysql::Error> {
        self.exec_first(
        "INSERT INTO job_applications (source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes)
        VALUES (:source, :company, :job_title, :application_date, :time_investment, :human_response, :human_response_date, :application_website, :notes)
        RETURNING id",
        application
    )
    .map(|new_id| JobApplication {id: new_id.unwrap_or_default(), ..application.clone()})
    }

    fn update_human_response(
        &mut self,
        id: i32,
        human_response: HumanResponse,
        human_response_date: Option<Date>,
    ) -> Result<(), mysql::Error> {
        self.exec_drop(
            "UPDATE job_applications
        SET human_response = :human_response, human_response_date = :human_response_date
        WHERE id = :id",
            params! {
                "id" => id,
                "human_response" => &human_response,
                "human_response_date" => human_response_date
            },
        )
    }

    fn update_job_application(&mut self, application: &JobApplication) -> Result<(), mysql::Error> {
        self.exec_drop(
            "UPDATE job_applications
        SET source = :source,
        company = :company,
        job_title = :job_title,
        application_date = :application_date,
        time_investment = :time_investment,
        human_response = :human_response,
        human_response_date = :human_response_date,
        application_website = :application_website,
        notes = :notes
        WHERE id = :id",
            application,
        )
    }

    fn update_job_application_partial(
        &mut self,
        partial_application: PartialJobApplication,
    ) -> Result<(), mysql::Error> {
        let mut query_builder = "UPDATE job_applications".to_owned();

        // Loop over all field names
        // Flag for if this is the first variable
        let mut is_first = true;
        for field in partial_application.0.iter() {
            if let JobApplicationField::Id(_) = field {
                // NO-OP: Id is special because we are using it in the WHERE clause instead of SET
            } else if is_first {
                // The first non-id value is special because of where the SET and commas are
                query_builder += &format!(" SET {0} = :{0}", field.name());
                is_first = false
            } else {
                // Normal placement
                query_builder += &format!(",\n{0} = :{0}", field.name());
            }
        }

        // Assert there is at least one change
        if is_first {
            // Use `std::io::Error` to return an arbitrary `mysql::Error`
            return Err(std::io::Error::other(
                "Unable to generate SQL statement because there are no changes",
            )
            .into());
        }

        // End with the WHERE clause
        query_builder += "\nWHERE id = :id";
        // RETURNING id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes";

        self.exec_drop(query_builder, partial_application)
    }

    fn delete_job_application(&mut self, id: i32) -> Result<(), mysql::Error> {
        self.exec_drop(
            "DELETE FROM job_applications WHERE id = :id",
            params! {"id" => id},
        )
    }
}
