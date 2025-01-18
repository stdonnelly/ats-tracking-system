//! Integration tests for the SQLite implementation

use rusqlite::{named_params, Connection};
use time::{ext::NumericalDuration as _, Date, Month};

use crate::{
    job_application_model::{HumanResponse, JobApplication},
    job_application_repository::JobApplicationRepository,
};

/// Test [JobApplicationRepository::get_job_applications] where the table has no rows
#[test]
fn test_get_job_applications_none() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    let job_applications = conn.get_job_applications()?;

    assert_eq!(
        job_applications,
        vec![],
        "Expected empty Vec when table is empty, got {job_applications:?}"
    );

    Ok(())
}

/// Test [JobApplicationRepository::get_job_applications] where the table has exactly one row
#[test]
fn test_get_job_applications_one() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    let id = 1;
    let source = "Test source";
    let company = "Test company";
    let job_title = "Test job title";
    let application_date = "2000-01-01";
    let time_investment = 83; // 1:23
    let human_response = HumanResponse::Rejection;
    let human_response_date = "2000-01-02";
    let application_website = "http://example.com";
    let notes = "Test notes\nWith newline";

    // Insert one job application with known fields
    conn.execute(
        "INSERT INTO job_applications (id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes) \
        VALUES (:id, :source, :company, :job_title, :application_date, :time_investment, :human_response, :human_response_date, :application_website, :notes)",
        named_params! {
            ":id": id,
            ":source": source,
            ":company": company,
            ":job_title": job_title,
            ":application_date": application_date,
            ":time_investment": time_investment,
            ":human_response": human_response,
            ":human_response_date": human_response_date,
            ":application_website": application_website,
            ":notes": notes,
        }
    )?;

    assert_eq!(
        conn.get_job_applications()?,
        vec![JobApplication {
            id,
            source: source.to_string(),
            company: company.to_string(),
            job_title: job_title.to_string(),
            application_date: Date::from_calendar_date(2000, Month::January, 1).unwrap(),
            time_investment: Some(time_investment.seconds()),
            human_response,
            human_response_date: Some(Date::from_calendar_date(2000, Month::January, 2).unwrap()),
            application_website: Some(application_website.to_string()),
            notes: Some(notes.to_string()),
        }],
        "Incorrect job application vec returned"
    );

    Ok(())
}

/// Test [JobApplicationRepository::get_job_applications] where the table has multiple rows
#[test]
fn test_get_job_applications_many() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    let source = "Test source";
    let company = "Test company";
    let job_title = "Test job title";
    let application_date = "2000-01-01";
    let time_investment = 83; // 1:23
    let human_response = HumanResponse::Rejection;
    let human_response_date = "2000-01-02";
    let application_website = "http://example.com";
    let notes = "Test notes\nWith newline";

    // Insert three job applications with known fields
    // This will insert 3 copies of the same values, but that's unlikely to miss any bugs
    conn.execute(
        "INSERT INTO job_applications (id, source, company, job_title, application_date, time_investment, human_response, human_response_date, application_website, notes) \
        VALUES (1, :source, :company, :job_title, :application_date, :time_investment, :human_response, :human_response_date, :application_website, :notes), \
        (2, :source, :company, :job_title, :application_date, :time_investment, :human_response, :human_response_date, :application_website, :notes), \
        (3, :source, :company, :job_title, :application_date, :time_investment, :human_response, :human_response_date, :application_website, :notes)",
        named_params! {
            ":source": source,
            ":company": company,
            ":job_title": job_title,
            ":application_date": application_date,
            ":time_investment": time_investment,
            ":human_response": human_response,
            ":human_response_date": human_response_date,
            ":application_website": application_website,
            ":notes": notes,
        }
    )?;

    let mut returned_vec = conn.get_job_applications()?;

    // Ensure the vector is sorted by id (ascending) because we are not testing order
    returned_vec.sort_unstable_by_key(|o| o.id);

    assert_eq!(
        returned_vec,
        vec![
            JobApplication {
                id: 1,
                source: source.to_string(),
                company: company.to_string(),
                job_title: job_title.to_string(),
                application_date: Date::from_calendar_date(2000, Month::January, 1).unwrap(),
                time_investment: Some(time_investment.seconds()),
                human_response,
                human_response_date: Some(
                    Date::from_calendar_date(2000, Month::January, 2).unwrap()
                ),
                application_website: Some(application_website.to_string()),
                notes: Some(notes.to_string()),
            },
            JobApplication {
                id: 2,
                source: source.to_string(),
                company: company.to_string(),
                job_title: job_title.to_string(),
                application_date: Date::from_calendar_date(2000, Month::January, 1).unwrap(),
                time_investment: Some(time_investment.seconds()),
                human_response,
                human_response_date: Some(
                    Date::from_calendar_date(2000, Month::January, 2).unwrap()
                ),
                application_website: Some(application_website.to_string()),
                notes: Some(notes.to_string()),
            },
            JobApplication {
                id: 3,
                source: source.to_string(),
                company: company.to_string(),
                job_title: job_title.to_string(),
                application_date: Date::from_calendar_date(2000, Month::January, 1).unwrap(),
                time_investment: Some(time_investment.seconds()),
                human_response,
                human_response_date: Some(
                    Date::from_calendar_date(2000, Month::January, 2).unwrap()
                ),
                application_website: Some(application_website.to_string()),
                notes: Some(notes.to_string()),
            },
        ]
    );

    Ok(())
}

/// Test [JobApplicationRepository::get_pending_job_applications]
///
/// 3 job applications are inserted, but only the one where `human_response == None` should be returned.
#[test]
fn test_get_pending_job_applications() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    // Base job application
    // We will only be checking id and human_response
    let job_application = JobApplication {
        id: 0,
        source: "Test source".to_string(),
        company: "Test company".to_string(),
        job_title: "Test job title".to_string(),
        application_date: Date::from_calendar_date(2000, Month::January, 1).unwrap(),
        time_investment: None,
        human_response: HumanResponse::None,
        human_response_date: None,
        application_website: None,
        notes: None,
    };

    conn.execute(
        "INSERT INTO job_applications (id, source, company, job_title, application_date, human_response) \
        VALUES (1, :source, :company, :job_title, :application_date, :human_response1), \
        (2, :source, :company, :job_title, :application_date, :human_response2), \
        (3, :source, :company, :job_title, :application_date, :human_response3)",
        named_params! {
            ":source": job_application.source,
            ":company": job_application.company,
            ":job_title": job_application.job_title,
            ":application_date": job_application.application_date,
            ":human_response1": HumanResponse::None,
            ":human_response2": HumanResponse::Rejection,
            ":human_response3": HumanResponse::InterviewRequest
        }
    )?;

    assert_eq!(
        conn.get_pending_job_applications()?,
        vec![JobApplication {
            id: 1,
            human_response: HumanResponse::None,
            ..job_application
        }],
    );

    Ok(())
}

#[test]
fn test_get_job_application_by_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

#[test]
fn test_search_job_applications() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

#[test]
fn test_insert_job_application() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

#[test]
fn test_update_human_response() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

#[test]
fn test_update_job_application() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

#[test]
fn test_update_job_application_partial() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

#[test]
fn test_delete_job_application() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;
    Ok(())
}

fn get_memory_connection() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open_in_memory()?;

    conn.execute(include_str!("../resources/sqlite_table_definition.sql"), ())?;

    Ok(conn)
}
