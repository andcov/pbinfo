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

/// Makes a get request to `url`
fn get_page(url: &str) -> reqwest::blocking::Response {
    reqwest::blocking::get(url).expect("Encountered an error while making a request to pbinfo.ro")
}

/// Extracts the problem id from the JSON "label" attribute. The "label" attribute is of the form `"label": "Problema #{id}: <strong>{name}</strong>`
fn extract_id_from_json(string: &str) -> Result<usize> {
    let error = PbInfoError::JSONError(String::from(
        "The JSON 'label' attribute should be of the form `'Problema #{id}: <strong>{name}</strong>'`",
    ));
    let mut words = string.split_whitespace();

    let pb = match words.next() {
        Some(res) => res,
        None => return Err(error),
    };

    if pb != "Problema" {
        return Err(error);
    }

    let padded_id = match words.next() {
        Some(res) => res,
        None => return Err(error),
    };

    let id_string = padded_id.replace("#", "").replace(":", "");
    let id_string = id_string.trim();

    match id_string.parse::<usize>() {
        Ok(res) => Ok(res),
        Err(_) => return Err(error),
    }
}

/// Extracts the input source (stdin or a file name) from the metadata text.
fn extract_input_source(string: &str) -> Result<IOSource> {
    let regex = regex::Regex::new(
        r#"<span style="background: url\(.*?>\s*([\w\.ă]+) / ([\w\.ă]+)\s*</span>"#,
    )
    .unwrap();

    let input_text = match regex.captures(string) {
        Some(res) => res[1].to_owned(),
        None => {
            return Err(PbInfoError::RegexError(String::from(
                "Failed to locate the input source in the HTML",
            )))
        }
    };
    let input_text = input_text.trim();

    match input_text {
        "tastatură" => Ok(IOSource::Std),
        _ => Ok(IOSource::File(String::from(input_text))),
    }
}

/// Extracts the output source (stdout or a file name) from the metadata text.
fn extract_output_source(string: &str) -> Result<IOSource> {
    let regex = regex::Regex::new(
        r#"<span style="background: url\(.*?>\s*([\w\.ă]+) / ([\w\.ă]+)\s*</span>"#,
    )
    .unwrap();

    let output_text = match regex.captures(string) {
        Some(res) => res[2].to_owned(),
        None => {
            return Err(PbInfoError::RegexError(String::from(
                "Failed to locate the output source in the HTML",
            )))
        }
    };
    let output_text = output_text.trim();

    match output_text {
        "ecran" => Ok(IOSource::Std),
        _ => Ok(IOSource::File(String::from(output_text))),
    }
}

const const_reg: &str = r#"<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>"#;

fn extract_grade(string: &str) -> Result<usize> {
    let regex = regex::Regex::new(const_reg).unwrap();

    let grade_str = match regex.captures(string) {
        Some(res) => res[2].to_owned(),
        None => {
            return Err(PbInfoError::RegexError(String::from(
                "Failed to locate the grade in the HTML",
            )))
        }
    };
    let grade_str = grade_str.trim();

    match grade_str.parse::<usize>() {
        Ok(grade) => Ok(grade),
        _ => Err(PbInfoError::RegexError(String::from(
            "Could not convert the grade into usize",
        ))),
    }
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
                        return Err(PbInfoError::RegexError(String::from(
                            "Failed to locate the problem text in the HTML",
                        )))
                    }
                };

                let metadata_regex =
                    regex::Regex::new(r#"<table class="table table-bordered">([\s\S]*?)</table>"#)
                        .unwrap();
                let metadata = match metadata_regex.captures(&text) {
                    Some(res) => res[1].to_owned(),
                    None => {
                        return Err(PbInfoError::RegexError(String::from(
                            "Failed to locate the problem metadata in the HTML",
                        )))
                    }
                };

                let input_source = extract_input_source(&metadata)?;
                let output_source = extract_input_source(&metadata)?;
                let grade = extract_grade(&metadata)?;

                Ok(PbInfoProblem {
                    id,
                    name: String::from(name),
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
                return Err(PbInfoError::JSONError(String::from(
                    "Could not parse JSON response",
                )))
            }
        };

        // A list of suggested problems; used only in case we do not find a matching name
        let mut suggested_problems: Vec<String> = Vec::new();
        for map in search_json.iter() {
            let possible_name = match map.get("value") {
                Some(res) => res,
                None => {
                    return Err(PbInfoError::JSONError(String::from(
                        "JSON should contain the 'value' attribute",
                    )))
                }
            };

            if possible_name.to_lowercase() == name {
                let label = match map.get("label") {
                    Some(res) => res,
                    None => {
                        return Err(PbInfoError::JSONError(String::from(
                            "JSON should contain the 'label' attribute",
                        )))
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
            String::from(name),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_id_from_json() {
        assert_eq!(
            extract_id_from_json("Problema #1691: <strong>Arbore1</strong"),
            Ok(1691)
        );
        assert_eq!(
            extract_id_from_json("Proema #1691: <strong>Arbore1</strong"),
            Err(PbInfoError::JSONError(String::from("The JSON 'label' attribute should be of the form `'Problema #{id}: <strong>{name}</strong>'`")
        )));
        assert_eq!(
            extract_id_from_json("Problema "),
            Err(PbInfoError::JSONError(String::from("The JSON 'label' attribute should be of the form `'Problema #{id}: <strong>{name}</strong>'`")
        )));
    }

    #[test]
    fn text_extract_io() {
        let metadata_file = r#"			</td>
		<td class="center">
			9		</td>
		<td>
			<span style="background: url('/img/32-fisier.png') no-repeat 3px center;background-size:16px;padding-left:34px;"> numere8.in / numere8.out </span> 		</td>
		<td>
					</td>
		<td class="center""#;
        assert_eq!(
            extract_input_source(&metadata_file),
            Ok(IOSource::File(String::from("numere8.in")))
        );
        assert_eq!(
            extract_output_source(&metadata_file),
            Ok(IOSource::File(String::from("numere8.out")))
        );

        let metadata_std = r#"<td class="center">
			9		</td>
		<td>
			<span style="background: url('/img/32-terminal.png') no-repeat 3px center;background-size:16px;padding-left:34px;">tastatură / ecran</span>		</td>
		<td>
			0.1 secunde
		</td>
		<td>"#;
        assert_eq!(extract_input_source(&metadata_std), Ok(IOSource::Std));
        assert_eq!(extract_output_source(&metadata_std), Ok(IOSource::Std));
    }

    #[test]
    fn test_extract_table() {
        let text = r#"<table class="table table-bordered">
	<tr>
				<th>Postată de</th>
		<th>Clasa</th>
		<th>Intrare/ieșire</th>
		<th>Limită timp</th>
		<th>Limită memorie</th>
		<th>Sursa problemei</th>
		<th>Autor</th>
		<th>Dificultate</th>
				<th>Scorul tău</th>
			</tr>
	<tr>
				<td>
						<span class="pbi-widget-user pbi-widget-user-span">
								<a href="/profil/silviu">
								<img src="https://www.gravatar.com/avatar/529e246d070445d00b4c98ced6152ca7?d=wavatar&s=32" style="border-radius:3px;vertical-align: middle;" />
				Candale Silviu (silviu)								</a>
							</span>
					</td>
		<td class="center">
			11		</td>
		<td>
			<span style="background: url('/img/32-fisier.png') no-repeat 3px center;background-size:16px;padding-left:34px;"> arbore1.in / arbore1.out </span> 		</td>
		<td>
			0.5 secunde
		</td>
		<td>
			<span title="Memorie totală">64 MB</span> / <span  title="Dimensiunea stivei">64 MB</span>
		</td>
		<td>
			ONI 2016, clasele XI-XII		</td>
		<td>
			Denis-Gabriel Mită		</td>
		<td class="center">
			concurs		</td>
							<td>
						<div class="center"><a href="/detalii-evaluare/35494272">100</a></div>
					</td>
						</tr>
</table>"#;

        assert_eq!(extract_grade(&text), Ok(11));
    }
}
