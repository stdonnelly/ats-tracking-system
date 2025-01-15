//! Job application repository with a MySQL database

use std::env;

use mysql::{OptsBuilder, Pool, PooledConn};

/// Define `struct JobApplication` and some implement conversions between that and MySQL objects
pub mod job_application_model;
/// Define CRUD actions for `struct JobApplication` into the MySQL database
pub mod job_application_repository;

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
