//! Integration tests for the SQLite implementation

use rusqlite::{named_params, Connection};
use time::{ext::NumericalDuration as _, Date, Month};

use crate::{
    job_application_model::{HumanResponse, JobApplication},
    job_application_repository::JobApplicationRepository,
};

// I attempted to make the tests only test one function, but manually operating on the DB got annoying.
// Starting with [test_search_job_applications], I decided to just use all necessary functions to make writing the tests easier.

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

/// Test [JobApplicationRepository::get_job_application_by_id]
///
/// Tests that existent job applications cause the function to return `Ok(Some(...))`
/// and that non-existent job applications return `Ok(None}`.
#[test]
fn test_get_job_application_by_id() -> Result<(), Box<dyn std::error::Error>> {
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

    assert_eq!(
        conn.get_job_application_by_id(2)?,
        Some(JobApplication {
            id: 2,
            source: source.to_string(),
            company: company.to_string(),
            job_title: job_title.to_string(),
            application_date: Date::from_calendar_date(2000, Month::January, 1).unwrap(),
            time_investment: Some(time_investment.seconds()),
            human_response,
            human_response_date: Some(Date::from_calendar_date(2000, Month::January, 2).unwrap()),
            application_website: Some(application_website.to_string()),
            notes: Some(notes.to_string()),
        }),
        "Job application ID 2 should find a job application"
    );

    assert_eq!(
        conn.get_job_application_by_id(4)?,
        None,
        "Job application ID 4 should not find anything"
    );

    Ok(())
}

/// Test [JobApplicationRepository::search_job_applications]
///
/// Should find all where the source, company, or job_title contains the search string.
/// Additionally, it should be case insensitive.
#[test]
fn test_search_job_applications() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    // Search string and a case-inverted version
    let search_string = "SeArCh StRiNg";
    let search_string_invert_case = "sEaRcH sTrInG";

    // Base job application without search string
    let job_application_base = JobApplication {
        id: 1,
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

    // Id for the base job application, which should not match
    let JobApplication { id: id_base, .. } = conn.insert_job_application(&job_application_base)?;

    // Id for job application where source matches
    let JobApplication { id: id_source, .. } = conn.insert_job_application(&JobApplication {
        source: "aaa".to_string() + search_string + "bbb",
        ..job_application_base.clone()
    })?;

    // Id for job application where company matches
    let JobApplication { id: id_company, .. } = conn.insert_job_application(&JobApplication {
        company: "aaa".to_string() + search_string,
        ..job_application_base.clone()
    })?;

    // Id for job application where job_title matches
    let JobApplication {
        id: id_job_title, ..
    } = conn.insert_job_application(&JobApplication {
        job_title: search_string.to_string(),
        ..job_application_base
    })?;

    let job_applications = conn.search_job_applications(&search_string)?;

    // Assert all job applications that should match do match
    for (name, id) in [
        ("source", id_source),
        ("company", id_company),
        ("job_title", id_job_title),
    ] {
        assert!(
            job_applications
                .iter()
                .any(|job_application| job_application.id == id),
            "{name} should produce a match"
        );
    }

    // And the base does not match
    assert!(
        !job_applications
            .iter()
            .any(|job_application| job_application.id == id_base),
        "The base job application should produce a match"
    );

    // Do the same for when the query would not match if it was case sensitive
    let job_applications_inverted_query = conn.search_job_applications(&search_string_invert_case)?;

    for (name, id) in [
        ("source", id_source),
        ("company", id_company),
        ("job_title", id_job_title),
    ] {
        assert!(
            job_applications_inverted_query
                .iter()
                .any(|job_application| job_application.id == id),
            "{name} should produce a match, even though the case does not match"
        );
    }

    assert!(
        !job_applications_inverted_query
            .iter()
            .any(|job_application| job_application.id == id_base),
        "The base job application should produce a match"
    );

    Ok(())
}

/// Test [JobApplicationRepository::insert_job_application]
#[test]
fn test_insert_job_application() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

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

    let JobApplication { id: id_1, .. } = conn.insert_job_application(&job_application)?;

    assert_eq!(id_1, 1, "Job applications should insert starting with id 1");

    assert_eq!(
        conn.get_job_application_by_id(id_1)?,
        Some(JobApplication {
            id: id_1,
            ..job_application.clone()
        }),
        "The job application should be able to be retrieved after being inserted"
    );

    let job_application_2 = JobApplication {
        time_investment: Some(90.seconds()),
        human_response: HumanResponse::Rejection,
        human_response_date: Some(Date::from_calendar_date(2000, Month::February, 2).unwrap()),
        application_website: Some("http://example.com".to_string()),
        notes: Some("Example\nNotes".to_string()),
        ..job_application
    };

    let JobApplication { id: id_2, .. } = conn.insert_job_application(&job_application_2)?;

    assert_eq!(id_2, 2, "The second job application should have id 2");

    assert_eq!(
        conn.get_job_application_by_id(id_2)?,
        Some(JobApplication {
            id: id_2,
            ..job_application_2
        }),
        "The second job application should be able to be retrieved after being inserted"
    );

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

/// Not a test. Just a helper function to generate empty memory connections.
fn get_memory_connection() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open_in_memory()?;

    conn.execute(include_str!("../resources/sqlite_table_definition.sql"), ())?;

    Ok(conn)
}
