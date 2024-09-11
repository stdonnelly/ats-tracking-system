CREATE DATABASE ats_tracking;
-- Change the password before running
CREATE USER 'ats_tracking'@'localhost' IDENTIFIED BY 'ats_tracking';
GRANT DELETE, INSERT, SELECT, UPDATE ON ats_tracking.* TO ats_tracking;
CREATE TABLE job_applications (
    id INT PRIMARY KEY AUTO_INCREMENT,
    source VARCHAR(60),
    company VARCHAR(60),
    job_title VARCHAR(255),
    application_date DATE,
    time_investment TIME,
    automated_response CHAR COMMENT 'Y=true,N=FALSE',
    human_response VARCHAR(60),
    human_response_date DATE,
    application_website VARCHAR(255),
    notes TEXT
);