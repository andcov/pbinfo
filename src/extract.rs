use crate::*;

type Result<T> = std::result::Result<T, PbInfoError>;

/// Extracts the problem id from the JSON "label" attribute. The "label" attribute is of the form `"label": "Problema #{id}: <strong>{name}</strong>`
pub fn extract_id_from_json(string: &str) -> Result<usize> {
    let error = PbInfoError::JSONError(
        "The JSON 'label' attribute should be of the form `'Problema #{id}: <strong>{name}</strong>'`".to_owned(),
    );
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
pub fn extract_input_source(string: &str) -> Result<IOSource> {
    let regex = regex::Regex::new(
        r#"<span style="background: url\(.*?>\s*([\w\.ă]+) / ([\w\.ă]+)\s*</span>"#,
    )
    .unwrap();

    let input_text = match regex.captures(string) {
        Some(res) => res[1].to_owned(),
        None => {
            return Err(PbInfoError::RegexError(
                "Failed to locate the input source in the HTML".to_owned(),
            ))
        }
    };
    let input_text = input_text.trim();

    match input_text {
        "tastatură" => Ok(IOSource::Std),
        _ => Ok(IOSource::File(input_text.to_owned())),
    }
}

/// Extracts the output source (stdout or a file name) from the metadata text.
pub fn extract_output_source(string: &str) -> Result<IOSource> {
    let regex = regex::Regex::new(
        r#"<span style="background: url\(.*?>\s*([\w\.ă]+) / ([\w\.ă]+)\s*</span>"#,
    )
    .unwrap();

    let output_text = match regex.captures(string) {
        Some(res) => res[2].to_owned(),
        None => {
            return Err(PbInfoError::RegexError(
                "Failed to locate the output source in the HTML".to_owned(),
            ))
        }
    };
    let output_text = output_text.trim();

    match output_text {
        "ecran" => Ok(IOSource::Std),
        _ => Ok(IOSource::File(output_text.to_owned())),
    }
}

const const_reg: &str = r#"<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>\s*?<td[ \S]*?>([\s\S]*?)</td>"#;

pub fn extract_grade(string: &str) -> Result<usize> {
    let regex = regex::Regex::new(const_reg).unwrap();

    let grade_str = match regex.captures(string) {
        Some(res) => res[2].to_owned(),
        None => {
            return Err(PbInfoError::RegexError(
                "Failed to locate the grade in the HTML".to_owned(),
            ))
        }
    };
    let grade_str = grade_str.trim();

    match grade_str.parse::<usize>() {
        Ok(grade) => Ok(grade),
        _ => Err(PbInfoError::RegexError(
            "Could not convert the grade into usize".to_owned(),
        )),
    }
}
