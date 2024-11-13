use std::env;

use dotenv::dotenv;
use mysql::{OptsBuilder, Pool, PooledConn};

mod command_line;
mod shell_option;

fn main() {
    // Objects that should be owned by the main function
    dotenv().unwrap();
    let mut conn = get_conn().unwrap();

    command_line::main_loop(&mut conn).unwrap();
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
