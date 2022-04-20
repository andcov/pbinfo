/// A problem from  PbInfo. Can be constructed using an id or a name.
#[derive(Debug)]
pub struct PbInfoProblem {
    pub id: usize,
    pub name: String,
    pub text: String,

    pub input_source: IOSource,
    pub output_source: IOSource,
    pub grade: usize,

    pub time_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub difficulty: Option<String>,

    pub author: Option<String>,
    pub source: Option<String>,
}

/// Describes the input/output source of a PbInfoProblem.
#[derive(Debug, PartialEq, Eq)]
pub enum IOSource {
    /// The source is a file.
    File(String),
    /// The source is stdin/stdout.
    Std,
}

/// Errors that may be encuntered when constructing a PbInfoProblem.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PbInfoError {
    /// Stores the unknown id.
    UnknownId(usize),
    /// Stores the unknown name, as well as a list of potential known names.
    UnknownName(String, Vec<String>),
    /// Error message related to networking.
    NetworkError(String),
    /// Error message related to JSON interpretation.
    JSONError(String),
    /// Error message related to the Html text that should contatin certain regex
    /// patterns.
    RegexError(String),
    /// Errors that do not fit into any of the other categories.
    Error(String),
}
type Result<T> = std::result::Result<T, PbInfoError>;

mod extract;
mod tests;
use crate::extract::*;

/// Makes a get request to `url`
fn get_page(url: &str) -> reqwest::blocking::Response {
    reqwest::blocking::get(url).expect("Encountered an error while making a request to pbinfo.ro")
}

impl PbInfoProblem {
    /// Construct PbInfoProblem from id.
    pub fn fetch_problem_by_id(id: usize) -> Result<Self> {
        let page = get_page(&format!("https://www.pbinfo.ro/probleme/{}", id));

        match page.status() {
            reqwest::StatusCode::OK => {
                let text = page.text().unwrap();

                let name_regex =
                    regex::Regex::new(r"<title>Problema ([\w]+) \| www.pbinfo.ro</title>").unwrap();
                let name = &name_regex.captures(&text).unwrap()[1];
                let name = name.to_lowercase();
                let name = name.as_str();

                let text_regex = regex::Regex::new(r"(<h1>Cerința</h1>[\s\S]*)</article>").unwrap();
                let problem_text = match text_regex.captures(&text) {
                    Some(res) => res[1].to_owned(),
                    None => {
                        return Err(PbInfoError::RegexError(
                            "Failed to locate the problem text in the HTML".to_owned(),
                        ))
                    }
                };

                let metadata_regex =
                    regex::Regex::new(r#"<table class="table table-bordered">([\s\S]*?)</table>"#)
                        .unwrap();
                let metadata = match metadata_regex.captures(&text) {
                    Some(res) => res[1].to_owned(),
                    None => {
                        return Err(PbInfoError::RegexError(
                            "Failed to locate the problem metadata in the HTML".to_owned(),
                        ))
                    }
                };

                let input_source = extract_input_source(&metadata)?;
                let output_source = extract_output_source(&metadata)?;
                let grade = extract_grade(&metadata)?;

                Ok(PbInfoProblem {
                    id,
                    name: name.to_owned(),
                    text: problem_text,

                    input_source,
                    output_source,
                    grade,

                    time_limit: None,
                    memory_limit: None,

                    author: None,
                    source: None,
                    difficulty: None,
                })
            }
            reqwest::StatusCode::NOT_FOUND => Err(PbInfoError::UnknownId(id)), // If the page does not exist, it means the id is wrong
            s => Err(PbInfoError::NetworkError(format!(
                "Encountered an error when trying to fetch the problem. HTTP status code {}",
                s
            ))),
        }
    }

    /// Construct PbInfoProblem from name.
    pub fn fetch_problem_by_name(name: &str) -> Result<Self> {
        use std::collections::HashMap;

        // `name` is converted to lowercase
        let name = name.to_lowercase();
        let name = name.as_str();

        // Get a list of all of the problems that (partially) match `name`
        let search_json = match get_page(&format!(
            "https://www.pbinfo.ro/php/ajax-search.php?term={}",
            name
        ))
        .json::<Vec<HashMap<String, String>>>()
        {
            Ok(res) => res,
            Err(_) => {
                return Err(PbInfoError::JSONError(
                    "Could not parse JSON response".to_owned(),
                ))
            }
        };

        // A list of suggested problems; used only in case we do not find a matching name
        let mut suggested_problems: Vec<String> = Vec::new();
        for map in search_json.iter() {
            let possible_name = match map.get("value") {
                Some(res) => res,
                None => {
                    return Err(PbInfoError::JSONError(
                        "JSON should contain the 'value' attribute".to_owned(),
                    ))
                }
            };

            if possible_name.to_lowercase() == name {
                let label = match map.get("label") {
                    Some(res) => res,
                    None => {
                        return Err(PbInfoError::JSONError(
                            "JSON should contain the 'label' attribute".to_owned(),
                        ))
                    }
                };

                // Try to get the id from the JSON
                let id = extract_id_from_json(&label)?;

                // Try to get the problem associated to `id`
                return Self::fetch_problem_by_id(id);
            } else {
                // If we do not get a match, we add the name to the a list of suggested problems
                suggested_problems.push(possible_name.clone());
            }
        }

        return Err(PbInfoError::UnknownName(
            name.to_owned(),
            suggested_problems,
        ));
    }

    pub fn get_task(&self) -> String {
        let content_regex = regex::Regex::new(r"<h1.*>Cerința</h1>[\s\S]*<p>(?P<task>[\s\S]+)</p>[\s\S]*<h1.*>Date de intrare</h1>[\s\S]*<p>(?P<input>[\s\S]+)</p>[\s\S]*<h1.*>Date de ieșire</h1>[\s\S]*<p>(?P<output>[\s\S]+)</p>[\s\S]*<h1.*>Restricții și precizări</h1>").unwrap();

        let caps = content_regex.captures(&self.text).unwrap();
        let task = &caps["task"];
        let input = &caps["input"];
        let output = &caps["output"];
        String::new()
    }
}
