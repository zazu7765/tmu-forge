const BASE_URL: &str = "https://www.torontomu.ca";
const COURSE_PATH: &str = "/calendar/2024-2025/courses";

#[derive(Debug, thiserror::Error)]
pub enum ScrapeError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("API error: {0}")]
    APIError(String),
}

pub mod data {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct CourseResponse {
        hits_per_page: i32,   // Always 2000
        total_matches: i32,   // Varies by department/query
        original_offset: i32, // Always 0 in examples
        data: Vec<Course>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Course {
        // Required fields - present in all courses
        page: String, // e.g., "/content/ryerson/calendar/2024-2025/courses/physics/PCS/102"
        data_url: String, // Links to JSON data
        course_code: String, // e.g., "PCS 102", "MTH 40A/B"
        long_title: String, // Course title
        course_description: String, // HTML-formatted description
        gpa_weight: String, // e.g., "1.00", "2.00", "0.00"

        // Optional time duration fields
        #[serde(default)]
        lecture_length: Option<String>, // e.g., "3 hrs.", "4 hrs."
        #[serde(default)]
        lab_length: Option<String>, // e.g., "1 hr.", "3 hrs.", "1.5 hrs."
        #[serde(default)]
        tutorial_length: Option<String>, // e.g., "1 hr.", "2 hrs."

        // Optional course unit fields
        #[serde(default)]
        billing_unit: Option<String>, // e.g., "1", "1/1", "2"
        #[serde(default)]
        course_count: Option<String>, // e.g., "1.00", "2.00"

        // Optional requisite fields (can contain HTML links)
        #[serde(default)]
        prerequisites: Option<String>,
        #[serde(default)]
        antirequisites: Option<String>,
        #[serde(default)]
        corequisites: Option<String>,
        #[serde(default)]
        customrequisites: Option<String>,

        // Other optional fields
        #[serde(default)]
        course_attribute: Option<String>, // e.g., "LL", "UL"
        #[serde(default)]
        consent: Option<String>, // e.g., "Departmental consent required"
    }
}

pub struct Scraper {
    client: reqwest::blocking::Client,
    url: String,
}

impl Scraper {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to build reqwest client");
        let url = format!("{}{}", BASE_URL, COURSE_PATH);
        Self { client, url }
    }

    pub fn get_departments(&self, faculty: &str) -> Result<Vec<String>, ScrapeError> {
        let response = self.client.get(&self.url).send()?;
        let document = scraper::Html::parse_document(&response.text()?);
        let tr = scraper::Selector::parse("tr").unwrap();
        let td = scraper::Selector::parse("td").unwrap();
        let a = scraper::Selector::parse("a").unwrap();
        let departments: Vec<String> = document
            .select(&tr)
            .filter_map(|row| {
                let tds: Vec<_> = row.select(&td).collect();
                if tds.len() == 2 && tds[1].text().collect::<String>().contains(faculty) {
                    tds[0]
                        .select(&a)
                        .next()?
                        .value()
                        .attr("href")
                        .map(|href| href.strip_suffix(".html").unwrap_or(href).to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(departments)
    }
}
