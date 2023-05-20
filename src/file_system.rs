

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Error, ErrorKind, BufReader};
use walkdir::WalkDir;
use xml::reader::{EventReader, XmlEvent};
use regex::Regex;

const WORD_TAG_NAME: &str = "w";

pub fn get_all_subdir(dir: &str) -> Result<Vec<String>, Error> {

    let subdirs = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|el| el.path().display().to_string())
        .collect::<Vec<_>>();

    Ok(subdirs)
}

pub fn get_all_files(dir: &str) -> Result<Vec<String>, Error> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_path = entry.path().canonicalize()?;
            let file_path_str = file_path.to_str().ok_or_else(|| Error::new(ErrorKind::Other, "invalid file path"))?.to_owned();
            files.push(file_path_str);
        }
    }

    Ok(files)
}


pub fn parse_xml_file(file_name: &str) -> Result<Vec<HashMap<String, String>>, Error> {
    let file = File::open(file_name)?;
    let file = BufReader::new(file);
    let parser = EventReader::new(file);

    let mut results: Vec<HashMap<String, String>> = Vec::new();
    let mut curr_obj: HashMap<String, String> = HashMap::new();
    let mut is_prev_word = false;
    let re = Regex::new(r"(?i)^[a-z\s_-]{1,45}$").unwrap();

    for event in parser {
        match event {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if !WORD_TAG_NAME.eq(&name.local_name) {continue;}
                for attr in attributes {
                    match attr.name.local_name.as_str() {
                        "pos" => { curr_obj.insert(String::from("pos"), String::from(attr.value)); }
                        "hw" => { curr_obj.insert(String::from("stem"), String::from(attr.value)); }
                        _ => ()
                    }
                }
                is_prev_word = true;
            }

            Ok(XmlEvent::Characters(content)) => {
                if !is_prev_word { continue; }
                is_prev_word = false;
                if !re.is_match(curr_obj.get("stem").unwrap()) { continue; }
                let content = content.trim();
                if !re.is_match(content) { continue; }
                curr_obj.insert(String::from("form"), String::from(content));
                results.push(curr_obj.clone());
            }

            _ => { is_prev_word = false; }
        }
    }

    Ok(results)
}