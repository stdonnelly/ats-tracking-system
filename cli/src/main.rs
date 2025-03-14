//! Entry point for the CLI version of ats-tracking

use dotenv::dotenv;

mod command_line;
mod shell_option;

fn main() {
    // Objects that should be owned by the main function
    // We don't really care is a .env file is successfully found because we can just use actual environment variables
    _ = dotenv();
    let mut conn = repository::get_conn().unwrap();

    command_line::main_loop(&mut conn).unwrap();
}
