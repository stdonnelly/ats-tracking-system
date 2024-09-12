use std::env;

use dotenv::dotenv;
use mysql::{OptsBuilder, Pool, PooledConn};
use repository::job_application_repository::{get_job_applications, JobApplication};

mod repository;

fn main() {
    dotenv().ok();
    let mut conn = get_conn().unwrap();

    let job_applications: Vec<JobApplication> = get_job_applications(&mut conn).unwrap();

    for job_application in job_applications {
        println!("{:?}", job_application);
    }
}

/// Boilerplate for getting connection information from environment
fn get_conn() -> Result<PooledConn, mysql::Error> {
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
