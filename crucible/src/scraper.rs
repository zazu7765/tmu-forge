/// BASE_URL to scrape
const BASE_URL: &str = "https://www.torontomu.ca";
/// COURSE_PATH to current calendar year's courses
const COURSE_PATH: &str = "/calendar/2024-2025/courses";

#[derive(Debug, thiserror::Error)]
pub enum ScrapeError {
    /// Error with reqwest library, client, or response
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    /// Generic Error with fetching data
    #[error("API error: {0}")]
    APIError(String),
}

pub mod data {
    use scraper::Html;
    use serde::{Deserialize, Serialize};
    /// Cleans all HTML Tags from input string,
    /// removes whitespace and joins words with a single space
    fn clean_html(s: &str) -> String {
        let frag = Html::parse_fragment(s);

        frag.root_element()
            .text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Custom Deserializer for serde to clean HTML tags from optional fields
    fn deserialize_optional_html<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<String>::deserialize(deserializer).map(|opt| opt.map(|s| clean_html(&s)))
    }

    /// Response from University JSON CMS API format
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CourseResponse {
        hits_per_page: i32,    // Always 2000
        total_matches: i32,    // Varies by department/query
        original_offset: i32,  // mostly 0??? (none of the pages are paginated)
        pub data: Vec<Course>, // The goods
    }

    // Individual Course Data returned from University JSON CMS API
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Course {
        // Required fields - present in all courses
        page: String, // e.g., "/content/ryerson/calendar/2024-2025/courses/physics/PCS/102"
        pub course_code: String, // e.g., "PCS 102", "MTH 40A/B"
        pub long_title: String, // Course title
        pub course_description: String, // HTML-formatted description
        gpa_weight: String, // e.g., "1.00", "2.00", "0.00"

        // optional maybe there field
        #[serde(default)]
        data_url: Option<String>, // Links to JSON data

        // Optional time duration fields
        #[serde(default)]
        pub lecture_length: Option<String>, // e.g., "3 hrs.", "4 hrs."
        #[serde(default)]
        pub lab_length: Option<String>, // e.g., "1 hr.", "3 hrs.", "1.5 hrs."
        #[serde(default)]
        pub tutorial_length: Option<String>, // e.g., "1 hr.", "2 hrs."

        // Optional course unit fields
        #[serde(default)]
        pub billing_unit: Option<String>, // e.g., "1", "1/1", "2"
        #[serde(default)]
        pub course_count: Option<String>, // e.g., "1.00", "2.00"

        // Optional requisite fields (can contain HTML links, so we clean them)
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_optional_html")]
        pub prerequisites: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_optional_html")]
        pub antirequisites: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_optional_html")]
        pub corequisites: Option<String>,
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_optional_html")]
        pub customrequisites: Option<String>,

        // Other optional fields
        #[serde(default)]
        pub course_attribute: Option<String>, // e.g., "LL", "UL"
        #[serde(default)]
        pub consent: Option<String>, // e.g., "Departmental consent required", debating if this should be a Boolean but Option kinda already makes it so
    }
}

/// Reqwest Wrapper class
pub struct Scraper {
    client: reqwest::blocking::Client,
    url: String,
}

impl Scraper {
    /// Default constructor to fake web browser user agent, may or may not work
    ///
    /// Also sets URL to BASE_URL/COURSE_PATH
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to build reqwest client");
        let url = format!("{}{}", BASE_URL, COURSE_PATH);
        Self { client, url }
    }

    /// Fetches departments by scraping the first web page we point to,
    /// Then picks only the actual department/discipline links from the table rows with the faculty name in it.
    ///
    /// Example HTML:
    /// ```html
    /// <tr role="row" class="odd">
    ///     <td class="sorting_1">
    ///         <a href="/content/ryerson/calendar/2024-2025/courses/physics.html">
    ///             Physics (PCS)
    ///         </a>
    ///     </td>
    ///     <td>Faculty of Science</td>
    /// </tr>
    /// ```
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

    /// Fetches courses from a given department link.
    ///
    /// The Link is used as  a referer to the API to not trigger any weird rate limiting,
    /// then returns the JSON response as a [CourseResponse](data::CourseResponse)
    pub fn get_courses(&self, department: &str) -> Result<data::CourseResponse, ScrapeError> {
        let link = department.replace("/content/ryerson", "");
        let url = format!(
            "{}{}/jcr:content/content/rescalendarcoursestack.data.1.json",
            BASE_URL, link
        );
        let res = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Referer", format!("{}{}", BASE_URL, link))
            .send()?;

        if !res.status().is_success() {
            return Err(ScrapeError::APIError(format!(
                "Failed to fetch courses from {} with status code {}",
                url,
                res.status().as_str()
            )));
        }
        Ok(res.json()?)
    }
}
