CREATE DATABASE ats_tracking;
-- Change the password before running
CREATE USER 'ats_tracking'@'localhost' IDENTIFIED BY 'ats_tracking';
GRANT DELETE,
    INSERT,
    SELECT,
    UPDATE ON ats_tracking.* TO ats_tracking;
CREATE TABLE job_applications (
    id INT PRIMARY KEY AUTO_INCREMENT,
    source VARCHAR(60) NOT NULL,
    company VARCHAR(60) NOT NULL,
    job_title VARCHAR(255) NOT NULL,
    application_date DATE NOT NULL,
    time_investment TIME,
    automated_response CHAR NOT NULL DEFAULT 'N' COMMENT 'Y=true,N=FALSE',
    human_response VARCHAR(60),
    human_response_date DATE,
    application_website VARCHAR(255),
    notes TEXT
);