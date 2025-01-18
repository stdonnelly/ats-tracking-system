//! Job application repository with a MySQL database

/// Define `struct JobApplication` and some implement conversions between that and MySQL objects
pub mod job_application_model;
/// Define CRUD actions for `struct JobApplication` into the MySQL database
pub mod job_application_repository;

pub use backend_connection::get_conn;

#[cfg(feature = "mysql")]
mod backend_connection {
    use std::env;

    use mysql::{OptsBuilder, Pool, PooledConn};

    /// Get a connection object to be used by the rest of this crate
    ///
    /// This exists so that the main function can own the connection object instead of creating a new one for every call
    pub fn get_conn() -> Result<PooledConn, mysql::Error> {
        let mut sql_opts_builder = OptsBuilder::new();
        if let Ok(host_name) = env::var("DB_HOST") {
            sql_opts_builder = sql_opts_builder.ip_or_hostname(Some(host_name));
        }
        if let Ok(port_str) = env::var("DB_PORT") {
            if let Ok(port_int) = port_str.parse::<u16>() {
                sql_opts_builder = sql_opts_builder.tcp_port(port_int);
            }
        }
        if let Ok(user) = env::var("DB_USER") {
            sql_opts_builder = sql_opts_builder.user(Some(user));
        }
        if let Ok(pass) = env::var("DB_PASSWORD") {
            sql_opts_builder = sql_opts_builder.pass(Some(pass));
        }
        if let Ok(db_name) = env::var("DB_DATABASE") {
            sql_opts_builder = sql_opts_builder.db_name(Some(db_name));
        }

        let pool = Pool::new(sql_opts_builder)?;
        pool.get_conn()
    }
}

#[cfg(not(feature = "mysql"))]
mod backend_connection {
    use std::path::{Path, PathBuf};

    use rusqlite::Connection;

    /// Get a connection object to be used by the rest of this crate
    ///
    /// This exists so that the main function can own the connection object instead of creating a new one for every call
    pub fn get_conn() -> Result<Connection, rusqlite::Error> {
        // The ats-tracking.db3 file should be placed in the user's home directory

        // The reason this is deprecated is fixed in Rust 1.85 and the deprecation notice will be removed soon.
        let home = std::env::home_dir().unwrap_or_else(|| {
            // If home_dir() fails, use current working directory
            eprintln!(
                "Unable to find home directory. Using current directory for ats-tracking.db3"
            );
            PathBuf::from(".")
        });

        get_or_make_db(home.join("ats-tracking.db3"))
    }

    /// Get connection for a path and ensure the job_applications table exists
    fn get_or_make_db<P: AsRef<Path>>(path: P) -> Result<Connection, rusqlite::Error> {
        // Get connection
        let conn = Connection::open(path)?;

        // Ensure the table exists
        conn.execute(include_str!("resources/sqlite_table_definition.sql"), ())?;

        // Return conn
        Ok(conn)
    }

    #[cfg(test)]
    mod tests {
        use tempfile::TempDir;

        use super::*;

        #[test]
        fn test_new_db() -> Result<(), Box<dyn std::error::Error>> {
            let expected_tbl_name = "job_applications";
            // Generates a 1 if the table exists, or a 0 if the table doesn't exist
            let expected_tbl_sql =
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type = 'table' AND tbl_name = ?1)";

            // Create a new directory for the database
            let path = TempDir::new()?;
            let db_path = path.path().join("test_db.db3");

            let conn = get_or_make_db(db_path)?;

            assert_eq!(
                1,
                conn.query_row(expected_tbl_sql, (expected_tbl_name,), |row| row
                    .get::<usize, i32>(0)
                    .map(Into::<i32>::into))?
            );

            Ok(())
        }

        #[test]
        fn test_existing_db() -> Result<(), Box<dyn std::error::Error>> {
            // Create db path
            let path = TempDir::new()?;
            let db_path = path.path().join("test_db.db3");

            let job_application_id: i64;
            let test_data = (
                "test source".to_string(),
                "test company".to_string(),
                "test job title".to_string(),
                "test application date".to_string(),
                3,
                "R".to_string(),
                "test human response date".to_string(),
                "test application website".to_string(),
                "test notes".to_string(),
            );

            // Create new database table and insert a row
            {
                let conn = get_or_make_db(&db_path)?;

                job_application_id = conn.prepare("INSERT INTO job_applications (source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes) \
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")?
                    .insert(test_data.clone()).expect("INSERT FAIL");
            }

            // The earlier conn should now be dropped, so we can connect to the file with a new connection
            let conn = get_or_make_db(&db_path)?;

            // Now that we have a new connection, the earlier data should still exist
            assert_eq!(
                test_data,
                conn.prepare("SELECT source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes FROM job_applications WHERE id = ?")?
                    .query_row((job_application_id,), |row| {
                        Ok((
                            row.get::<usize, _>(0)?,
                            row.get::<usize, _>(1)?,
                            row.get::<usize, _>(2)?,
                            row.get::<usize, _>(3)?,
                            row.get::<usize, _>(4)?,
                            row.get::<usize, _>(5)?,
                            row.get::<usize, _>(6)?,
                            row.get::<usize, _>(7)?,
                            row.get::<usize, _>(8)?
                        ))
                    }).map_err(|e| match e {
                        rusqlite::Error::QueryReturnedNoRows => "New database connection failed to use existing database.".to_string(),
                        _ => format!("{e}")
                    })?
            );

            Ok(())
        }
    }
}
