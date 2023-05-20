

mod file_system;
mod db;

use file_system::*;
use mysql::{prelude::Queryable, params};

use std::collections::HashMap;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};


const BNC_DIR: &str = "D:/Work/MovaClub/trash/BritishNationalCorp/2554/download/Texts";

fn main() {
    let (tx, rx) = mpsc::channel();
    let parsed_data: Arc<Mutex<HashMap<String, Vec<HashMap<String, String>>>>> = Arc::new(Mutex::new(HashMap::new()));
    let subdirs = get_all_subdir(BNC_DIR).unwrap();
    // let total_files = get_all_files(BNC_DIR).unwrap().len();

    let mut threads = Vec::new();
    for subdir in subdirs {

        let parsed_data = Arc::clone(&parsed_data);
        let tx = tx.clone();
        let th = thread::spawn(move || {

            let files = &get_all_files(&subdir).unwrap();

            for file in files {
                let data = parse_xml_file(file.as_str()).expect("Something went wrong");

                for word in data {
                    let stem = word.get("stem").unwrap().to_owned();
                    let form = word.get("form").unwrap().to_owned();
                    let pos = word.get("pos").unwrap().to_owned();

                    let mut new_word_form:HashMap<String, String> = HashMap::new();
                    new_word_form.insert(String::from("pos"), pos);
                    new_word_form.insert(String::from("form"), form);

                    let mut obj = parsed_data
                                                                    .lock()
                                                                    .unwrap();
                    let obj = obj.entry(stem)
                                                                    .or_insert(vec![new_word_form.clone()]);
                                                                
                    if obj.len() > 1 && !obj.contains(&new_word_form) {
                        obj.push(new_word_form);
                    }
                }
                tx.send((String::from(&subdir), String::from(file), files.len())).unwrap();
            }
        });

        threads.push(th);
    }

    let mut counter = 0;
    let mut subdir_files_counter: HashMap<String, u16> = HashMap::new();
    while let Ok((subdir, file_name, total)) = rx.recv() {
        counter += 1;
        // total_files += total;
        let c_value = subdir_files_counter.entry(subdir.clone()).or_insert(total as u16);
        *c_value = total as u16;
        let mut total_files = 0_u16;
        subdir_files_counter.iter().for_each(|(_key, val)| { total_files += val; });
        println!(" ({counter}/{total_files})   {} - OK", file_name);
        if counter == total_files { break }
    }

    for th in threads {
        th.join().unwrap();
    }

    let total_stem = parsed_data.lock().unwrap().keys().count();
    println!("Total stem: {total_stem}");

    let mut connector = db::Connector::new();

    let data = &*parsed_data.lock().unwrap();
    let mut con = connector.connection().unwrap();
    let total_keys = data.keys().count();
    let mut counter = 0;

    for (key, val) in data {
        counter += 1;
        println!("\t{counter} from {total_keys} | WORD: {key}");
        con.exec_batch(
            r"INSERT INTO forms (stem, main, pos) 
            VALUES (:stem, :main, :pos)",
            val.iter().map(|h| params! {
                "main" => key,
                "stem" => h.get("form").unwrap(),
                "pos" => h.get("pos").unwrap()
            })
        ).expect("Error occured while inserting entries into DB");
    }


    


}
