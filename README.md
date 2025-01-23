# ats-tracking-system

The applicant tracking system tracking system

## Overview

The name is a slight misnomer, as this tracks job applications and employers, not the ATSs they use.
This application is designed to track each job application.
This was created because, at the time of writing, landing a software development job requires numerous applications.
Because most employers receive numerous applications, many elect to use applicant tracking systems \[ATS\]s
Many applications do not even receive a response, making this a useful tool for determining where to send follow-ups, assuming the applicant is willing to follow up.

## Building

The Rust compiler is required. See [Install Rust](https://www.rust-lang.org/tools/install).

For the default options, with optimizations: Run `cargo build --release` in this repository's root directory. In case you aren't used to the Rust compiler, this will take a few minutes.

This will create two executables in the `target/release/` folder: `ats-tracking` and `ats-tracking-cli`.
These are the executables the GUI and CLI versions respectively.
Both are standalone executables that can be moved used anywhere, but they will both create and use a file named `ats-tracking.db3` in the user's home directory (`$HOME` on macOS/Linux/Unix-like, or `%USERPROFILE%` on Windows[^1]).

### Options

#### MySQL

Using the argument `--features repository/mysql` when compiling will cause the application to use MySQL instead of SQLite for the database.
This means that the `ats-tracking.db3` file will not be used, but a connection to a MySQL server will be required.
The connection must be to a database that contains the `job_applications` table, and a user that has DELETE, INSERT, SELECT, and UPDATE permissions to that table.
The schema for that table is in [db_config.sql](setup_scripts/db_config.sql).

The following environment variables are used by the MySQL version of the ATS tracking system.
They can either be defined as environment variables, or in a file named ".env" in the same folder as the executable (or a parent folder).

- DB_DATABASE: The name of the database to be used by ats-tracking
- DB_USER: The username for ats-tracking to use for database access
- DB_PASSWORD: The password for ats-tracking to use for database access
- DB_HOST: Optional - The hostname or IP address of the database. Defaults to `127.0.0.1`
- DB_PORT: Optional - The port of the database. Defaults to `3306`

#### Optimization

The `--release` argument is not necessary. It just makes the compiled application run a bit faster. See [Profiles - The Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html) for more information.

#### Only compiling one executable

It is possible to only build one of the executables by building with `--bin ats-tracking` or `--bin ats-tracking-cli` to select which binary to compile.

## Basic architecture

### Programming language

Rust

While it is not the most appropriate language for this project, I decided to use Rust for the majority of the code.
This is mainly for me to become more familiar with the language.

### Database

#### SQLite

This is the new default as of version 0.3.0.
MySQL is still available using the feature `repository/mysql`.
This has been added because this application does not need an actual SQL server, which add complexity in deployment.
 As an added benefit, SQLite is easier to test because I can create in-memory SQLite databases.

#### MySQL or MariaDB

MariaDB is simple to install on Linux and set up.
All data this project will likely involve easily fits into ordinary SQL tables.
At the time of writing this, the only data that needs to be stored is whatever is related t each job application and maybe login information if this becomes a webapp.

### Frontend

#### CLI

This was the first interface that was made. It can still be used with the executable`ats-tracking-cli`.

#### GUI

There is now (as of version 0.2.0) a graphical interface, with the executable`ats-tracking`.
This uses [Slint](https://slint.dev/).
Currently, there are some features present in the CLI version, but not the GUI version.
Some of these features are irrelevant to the GUI (i.e. a partial update is unnecessary because the app can just pre-fill the form).

## License

This software is licensed under the MIT license (see [LICENSE](LICENSE)) but uses the Slint library under the GNU GPLv3 license for the gui crate.
Any derivatives of this project that include Slint need to use one of the [Slint licenses](https://github.com/slint-ui/slint/blob/master/LICENSE.md).

## Footnotes

[^1]: Because the function that finds the home directory is not implemented correctly for Rust <= 1.84, the `ats-tracking.db3` may be placed in the wrong directory on Windows, or the application may fail to run. macOS, Linux, and other Unix-like operating systems are unaffected. See the deprecation notice on [home_dir](https://doc.rust-lang.org/1.84.0/std/env/fn.home_dir.html#deprecation)
