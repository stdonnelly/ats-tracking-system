//! Integration tests for the SQLite implementation

#![cfg(not(feature = "mysql"))]

use rusqlite::{named_params, Connection};
use time::{ext::NumericalDuration as _, Date, Month};

use repository::{
    job_application_model::{
        HumanResponse, JobApplication, JobApplicationField, PartialJobApplication,
    },
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
    let job_applications_inverted_query =
        conn.search_job_applications(&search_string_invert_case)?;

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

/// Test [JobApplicationRepository::update_human_response] with all three human response variants
#[test]
fn test_update_human_response() -> Result<(), Box<dyn std::error::Error>> {
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

    let inserted = conn.insert_job_application(&job_application)?;

    // Job application should now have human response `None`
    // No need to test this, because there is already a test for inserting

    // Change to rejection
    conn.update_human_response(
        inserted.id,
        HumanResponse::Rejection,
        Some(Date::from_calendar_date(2000, Month::February, 2).unwrap()),
    )?;

    // Assert rejection with that date
    assert_eq!(
        conn.get_job_application_by_id(inserted.id)?,
        Some(JobApplication {
            human_response: HumanResponse::Rejection,
            human_response_date: Some(Date::from_calendar_date(2000, Month::February, 2).unwrap()),
            ..inserted.clone()
        }),
        "Job application should have human response 'Rejection'"
    );

    // Change to interview request
    conn.update_human_response(
        inserted.id,
        HumanResponse::InterviewRequest,
        Some(Date::from_calendar_date(2000, Month::February, 3).unwrap()),
    )?;

    // Assert interview request with the given date
    assert_eq!(
        conn.get_job_application_by_id(inserted.id)?,
        Some(JobApplication {
            human_response: HumanResponse::InterviewRequest,
            human_response_date: Some(Date::from_calendar_date(2000, Month::February, 3).unwrap()),
            ..inserted.clone()
        }),
        "Job application should have human response 'Interview Request'"
    );

    // Change to None
    conn.update_human_response(inserted.id, HumanResponse::None, None)?;

    assert_eq!(
        conn.get_job_application_by_id(inserted.id)?,
        Some(JobApplication {
            human_response: HumanResponse::None,
            human_response_date: None,
            ..inserted.clone()
        }),
        "Job application should have human response 'None'"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application]
///
/// This should test that the matching job application, and only the matching job application, is updated
#[test]
fn test_update_job_application() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert the job application twice
    let job_application_1 = conn.insert_job_application(&job_application)?;
    let job_application_2 = conn.insert_job_application(&job_application)?;

    // Generated an updated job application from the second id
    let updated_job_application = JobApplication {
        id: job_application_2.id,
        source: "Updated source".to_string(),
        company: "Updated company".to_string(),
        job_title: "Updated job title".to_string(),
        application_date: Date::from_calendar_date(2001, Month::January, 1).unwrap(),
        time_investment: Some(3.seconds()),
        human_response: HumanResponse::Rejection,
        human_response_date: Some(Date::from_calendar_date(2001, Month::February, 1).unwrap()),
        application_website: Some("http://example.com".to_string()),
        notes: Some("Updated notes".to_string()),
    };

    // Perform the update
    conn.update_job_application(&updated_job_application)?;

    assert_eq!(
        conn.get_job_application_by_id(job_application_1.id)?,
        Some(job_application_1),
        "Other job applications should be unaffected"
    );

    assert_eq!(
        conn.get_job_application_by_id(job_application_2.id)?,
        Some(updated_job_application),
        "The matching job application should be updated"
    );

    Ok(())
}

#[test]
fn test_update_job_application_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert
    let inserted = conn.insert_job_application(&job_application)?;

    // Try to update without an ID
    // In order to prevent changes that would affect the MySQL backend as well, errors for invalid ids may not exist
    _ = conn.update_job_application(&JobApplication {
        id: 3,
        ..inserted.clone()
    });

    // Assert no change was made
    assert_eq!(
        conn.get_job_applications()?,
        vec![inserted],
        "Job application should not be changed when an error is encountered"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application_partial] to make sure that unreferenced fields are not affected
#[test]
fn test_update_job_application_partial_partial() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert the job application twice so we can ensure the non-matching job application is not modified
    let job_application_1 = conn.insert_job_application(&job_application)?;
    let job_application_2 = conn.insert_job_application(&job_application)?;

    // Create an update
    let update = PartialJobApplication(vec![
        JobApplicationField::Id(job_application_2.id),
        JobApplicationField::Company("Updated company".to_string()),
    ]);

    // Execute the update
    conn.update_job_application_partial(update)?;

    // Assert others are unaffected
    assert_eq!(
        conn.get_job_application_by_id(job_application_1.id)?,
        Some(job_application_1),
        "Other job applications should be unaffected"
    );

    // Assert the change happened correctly
    assert_eq!(
        conn.get_job_application_by_id(job_application_2.id)?,
        Some(JobApplication {
            company: "Updated company".to_string(),
            ..job_application_2
        }),
        "The matching job application should be updated"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application_partial] to make sure that all referenced fields are updated
#[test]
fn test_update_job_application_partial_full() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    // Most of this code is copied from `test_update_job_application()`
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

    // Insert the job application twice
    let job_application_1 = conn.insert_job_application(&job_application)?;
    let job_application_2 = conn.insert_job_application(&job_application)?;

    // Generated an updated job application from the second id
    let updated_job_application = JobApplication {
        id: job_application_2.id,
        source: "Updated source".to_string(),
        company: "Updated company".to_string(),
        job_title: "Updated job title".to_string(),
        application_date: Date::from_calendar_date(2001, Month::January, 1).unwrap(),
        time_investment: Some(3.seconds()),
        human_response: HumanResponse::Rejection,
        human_response_date: Some(Date::from_calendar_date(2001, Month::February, 1).unwrap()),
        application_website: Some("http://example.com".to_string()),
        notes: Some("Updated notes".to_string()),
    };

    // Get the updated job application as a partial job application
    let update_as_partial = PartialJobApplication(vec![
        JobApplicationField::Id(updated_job_application.id),
        JobApplicationField::Source(updated_job_application.source.clone()),
        JobApplicationField::Company(updated_job_application.company.clone()),
        JobApplicationField::JobTitle(updated_job_application.job_title.clone()),
        JobApplicationField::ApplicationDate(updated_job_application.application_date),
        JobApplicationField::TimeInvestment(updated_job_application.time_investment),
        JobApplicationField::HumanResponse(updated_job_application.human_response),
        JobApplicationField::HumanResponseDate(updated_job_application.human_response_date),
        JobApplicationField::ApplicationWebsite(
            updated_job_application.application_website.clone(),
        ),
        JobApplicationField::Notes(updated_job_application.notes.clone()),
    ]);

    // Perform the update
    conn.update_job_application_partial(update_as_partial)?;

    assert_eq!(
        conn.get_job_application_by_id(job_application_1.id)?,
        Some(job_application_1),
        "Other job applications should be unaffected"
    );

    assert_eq!(
        conn.get_job_application_by_id(job_application_2.id)?,
        Some(updated_job_application),
        "The matching job application should be updated"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application_partial] to ensure a missing ID is handled correctly
#[test]
fn test_update_job_application_partial_missing_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    let expected_error_message = "Unable to generate SQL statement because there is no id field";

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

    // Insert
    let inserted = conn.insert_job_application(&job_application)?;

    // Create an update
    let update = PartialJobApplication(vec![JobApplicationField::Company(
        "Updated company".to_string(),
    )]);

    // Try to update without an ID
    let update_result = conn.update_job_application_partial(update);

    // Assert the correct error was generated
    assert_eq!(
        update_result
            .expect_err(
                "Attempting to update a job application without an ID should generate an error"
            )
            .to_string(),
        expected_error_message,
        "Unexpected error message"
    );

    // Assert no change was made
    assert_eq!(
        conn.get_job_applications()?,
        vec![inserted],
        "Job application should not be changed when an error is encountered"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application_partial] to ensure too many IDs is handled correctly
#[test]
fn test_update_job_application_partial_too_many_ids() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    let expected_error_message =
        "Unable to generate SQL statement because there are multiple id fields";

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

    // Insert
    let inserted = conn.insert_job_application(&job_application)?;

    // Create an update
    let update = PartialJobApplication(vec![
        JobApplicationField::Id(1),
        JobApplicationField::Company("Updated company".to_string()),
        JobApplicationField::Id(2),
    ]);

    // Try to update without an ID
    let update_result = conn.update_job_application_partial(update);

    // Assert the correct error was generated
    assert_eq!(
        update_result
            .expect_err(
                "Attempting to update a job application with multiple IDs should generate an error"
            )
            .to_string(),
        expected_error_message,
        "Unexpected error message"
    );

    // Assert no change was made
    assert_eq!(
        conn.get_job_applications()?,
        vec![inserted],
        "Job application should not be changed when an error is encountered"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application_partial] to ensure an invalid ID is handled correctly
#[test]
fn test_update_job_application_partial_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert
    let inserted = conn.insert_job_application(&job_application)?;

    // Create an update
    let update = PartialJobApplication(vec![
        JobApplicationField::Id(3),
        JobApplicationField::Company("Updated company".to_string()),
    ]);

    // Try to update without an ID
    // In order to prevent changes that would affect the MySQL backend as well, errors for invalid ids may not exist
    _ = conn.update_job_application_partial(update);

    // Assert no change was made
    assert_eq!(
        conn.get_job_applications()?,
        vec![inserted],
        "Job application should not be changed when an error is encountered"
    );

    Ok(())
}

/// Test [JobApplicationRepository::update_job_application_partial] to ensure a request with no changes is handled correctly
#[test]
fn test_update_job_application_partial_no_change() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_memory_connection()?;

    let expected_error_message = "Unable to generate SQL statement because there are no changes";

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

    // Insert
    let inserted = conn.insert_job_application(&job_application)?;

    // Create an update
    let update = PartialJobApplication(vec![JobApplicationField::Id(inserted.id)]);

    // Try to update without an ID
    let update_result = conn.update_job_application_partial(update);

    // Assert the correct error was generated
    assert_eq!(
        update_result
            .expect_err("No changes should produce an error")
            .to_string(),
        expected_error_message,
        "Unexpected error message"
    );

    // Assert no change was made
    assert_eq!(
        conn.get_job_applications()?,
        vec![inserted],
        "Job application should not be changed when an error is encountered"
    );

    Ok(())
}

/// Test [JobApplicationRepository::delete_job_application] to ensure the correct job application is deleted
#[test]
fn test_delete_job_application() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert the job application twice
    let job_application_1 = conn.insert_job_application(&job_application)?;
    let job_application_2 = conn.insert_job_application(&job_application)?;

    // Delete job application 2 and only job application 2
    conn.delete_job_application(job_application_2.id)?;

    assert_eq!(
        conn.get_job_applications()?,
        vec![job_application_1],
        "Job application 2, and only job application 2, should be deleted"
    );

    Ok(())
}

/// Test [JobApplicationRepository::delete_job_application] to ensure an error is generated when an invalid id is used
#[test]
fn test_delete_job_application_invalid_id() -> Result<(), Box<dyn std::error::Error>> {
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

    // Insert the job application twice
    let job_application_1 = conn.insert_job_application(&job_application)?;

    // Delete by an invalid id
    // In order to prevent changes that would affect the MySQL backend as well, errors for invalid ids may not exist
    _ = conn.delete_job_application(3);

    assert_eq!(
        conn.get_job_applications()?,
        vec![job_application_1],
        "No job application should be deleted because the ID was invalid"
    );

    Ok(())
}

/// Not a test. Just a helper function to generate empty memory connections.
fn get_memory_connection() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        include_str!("../src/resources/sqlite_table_definition.sql"),
        (),
    )?;

    Ok(conn)
}
