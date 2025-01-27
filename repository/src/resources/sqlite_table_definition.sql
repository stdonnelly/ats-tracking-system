CREATE TABLE IF NOT EXISTS job_applications (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    company TEXT NOT NULL,
    job_title TEXT NOT NULL,
    application_date TEXT NOT NULL,
    time_investment INTEGER,
    human_response TEXT CHECK(human_response IN ('N','R','I','IR','J')) NOT NULL DEFAULT 'N',
    human_response_date TEXT,
    application_website TEXT,
    notes TEXT
);
