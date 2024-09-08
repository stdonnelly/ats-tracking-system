# ats-tracking-system

The applicant tracking system tracking system

## Overview

The name is a slight misnomer, as this tracks job applications and employers, not the ATSs they use.
This application is designed to track each job application.
This was created because, at the time of writing, landing a software development job requires numerous applications.
Because most employers receive numerous applications, many elect to use applicant tracking systems \[ATS\]s
Many applications do not even receive a response, making this a useful tool for determining where to send follow-ups, assuming the applicant is willing to follow up.

## Basic architecture

- Programming language: Rust.
  While it is not the most appropriate language for this project, I decided to use Rust for the majority of the code.
  This is mainly for me to become more familiar with the language.
- Database: MySQL or MariaDB.
  MariaDB is simple to install on Linux and set up.
  All data this project will likely involve easily fits into ordinary SQL tables.
  At the time of writing this, the only data that needs to be stored is whatever is related to each job application and maybe login information if this becomes a webapp.
- Frontend: During initial development and testing, there will be a command-line interface.
  After most of the application is developed, some kind of GUI will probably be made.
  This may be either GTK, Qt, or some web frontend.
