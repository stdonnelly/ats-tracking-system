use rusqlite::{named_params, params_from_iter, Connection, OptionalExtension, Params, ToSql};
use time::{Date, Duration};

use crate::job_application_model::{
    HumanResponse, JobApplication, JobApplicationField, PartialJobApplication,
};

use super::JobApplicationRepository;

impl JobApplicationRepository for Connection {
    type Error = rusqlite::Error;

    fn get_job_applications(&mut self) -> Result<Vec<JobApplication>, Self::Error> {
        execute_query(
            self,
            "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes \
            FROM job_applications",
            ()
        )
    }

    fn get_pending_job_applications(&mut self) -> Result<Vec<JobApplication>, Self::Error> {
        execute_query(
            self,
            "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes \
            FROM job_applications \
            WHERE human_response = 'N'",
            ()
        )
    }

    fn get_job_application_by_id(
        &mut self,
        id: i32,
    ) -> Result<Option<JobApplication>, Self::Error> {
        let mut stmt = self.prepare_cached("SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes \
            FROM job_applications \
            WHERE id = ?"
        )?;

        // Execute the statement
        // At most one row can be returned when querying by primary key
        stmt.query_row((id,), |row| row.try_into()).optional()
    }

    fn search_job_applications(&mut self, query: &str) -> Result<Vec<JobApplication>, Self::Error> {
        // Add wildcards to query
        let query_with_wildcards = "%".to_owned() + &query.to_lowercase() + "%";

        execute_query(
            self,
            "SELECT id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes \
            FROM job_applications \
            WHERE LOWER(source) LIKE ?1 \
            OR LOWER(company) LIKE ?1 \
            OR LOWER(job_title) LIKE ?1",
        (query_with_wildcards,)
        )
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
        human_response: HumanResponse,
        human_response_date: Option<Date>,
    ) -> Result<(), Self::Error> {
        let mut stmt = self.prepare_cached(
            "UPDATE job_applications \
            SET human_response = :human_response, human_response_date = :human_response_date \
            WHERE id = :id",
        )?;

        stmt.execute(named_params! {
            ":id": id,
            ":human_response": human_response,
            ":human_response_date": human_response_date,
        })
        .map(|_| ())
    }

    fn update_job_application(&mut self, application: &JobApplication) -> Result<(), Self::Error> {
        let mut stmt = self.prepare_cached(
            "UPDATE job_applications \
            SET source = :source, \
            company = :company, \
            job_title = :job_title, \
            application_date = :application_date, \
            time_investment = :time_investment, \
            human_response = :human_response, \
            human_response_date = :human_response_date, \
            application_website = :application_website, \
            notes = :notes \
            WHERE id = :id",
        )?;

        stmt.execute(named_params! {
            ":id": application.id,
            ":source": application.source,
            ":company": application.company,
            ":job_title": application.job_title,
            ":application_date": application.application_date,
            ":time_investment": application.time_investment.map(Duration::whole_seconds),
            ":human_response": application.human_response,
            ":human_response_date": application.human_response_date,
            ":application_website": application.application_website,
            ":notes": application.notes,
        })
        .map(|_| ())
    }

    fn update_job_application_partial(
        &mut self,
        partial_application: PartialJobApplication,
    ) -> Result<(), Self::Error> {
        // Build the query parameters in a string
        // This is necessary because we only want to modify the given columns
        // This is not a SQLi vulnerability because we will only be using this for the names, which are defined statically in `JobApplicationField::name()`
        let mut query_builder = "UPDATE job_applications".to_owned();

        let mut id_index: Option<usize> = None;

        // Loop over all field names
        // Flag for if this is the first variable
        // We still need this, even though we are looping over an index, because id should not affect first
        let mut is_first = true;
        for (index, field) in partial_application.0.iter().enumerate() {
            if let JobApplicationField::Id(_) = field {
                // Id is special because we are using it in the WHERE clause instead of SET
                // Ensure there is only one id. If there are more the statement will fail
                if id_index.is_none() {
                    id_index = Some(index + 1);
                } else {
                    return Err(rusqlite::Error::ToSqlConversionFailure(Box::from(
                        "Unable to generate SQL statement because there are multiple id fields",
                    )));
                }
            } else if is_first {
                // The first non-id value is special because of where the SET and commas are
                query_builder += &format!(" SET {} = ?{}", field.name(), index + 1);
                is_first = false
            } else {
                // Normal placement
                query_builder += &format!(",\n{} = ?{}", field.name(), index + 1);
            }
        }

        // End with the WHERE clause
        query_builder += &format!(
            "\nWHERE id = ?{}",
            id_index.ok_or(rusqlite::Error::ToSqlConversionFailure(Box::from(
                "Unable to generate SQL statement because there is no id field"
            )))?
        );

        // Now that we have the statement, prepare it
        // We will not be caching this due to the variance in the number of ways to represent this query
        let mut stmt = self.prepare(&query_builder)?;

        // Finally, execute returning Result<()>
        stmt.execute(params_from_iter(Into::<Vec<Box<dyn ToSql>>>::into(
            partial_application,
        )))
        .map(|_| ())
    }

    fn delete_job_application(&mut self, id: i32) -> Result<(), Self::Error> {
        let mut stmt = self.prepare_cached("DELETE FROM job_applications WHERE id = ?")?;

        stmt.execute((id,)).map(|_| ())
    }
}

/// Internal method to make a query where multiple rows are returned easier
///
/// This function exists for sqlite but not mysql because the sqlite query process has much more boilerplate
fn execute_query<P: Params>(
    conn: &mut Connection,
    sql: &str,
    params: P,
) -> Result<Vec<JobApplication>, rusqlite::Error> {
    // Create a prepared statement object
    let mut stmt = conn.prepare_cached(sql)?;

    // Execute the statement, which will return an iterator to the mapped rows
    // For some reason, using `TryInto::try_into` gives an error message, but calling within a closure doesn't
    let row_iter = stmt.query_map(params, |row| row.try_into())?;

    // Collect into a Vec using a for loop because we can't effectively use collect the iterator using `Iterator::collect`.
    // This is because we want the row mapping result to be passed to the caller on error using the `?` operator.
    let mut row_vec: Vec<JobApplication> = Vec::new();
    for row in row_iter {
        row_vec.push(row?);
    }

    // Return the collected Vec
    Ok(row_vec)
}
