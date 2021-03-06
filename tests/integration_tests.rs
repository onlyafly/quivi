// extern crate we're testing, same as any other code would do.
extern crate colored;
extern crate macaroon;

use colored::*;
use macaroon::ast::{ReaderObj, WriterObj};
use macaroon::back;
use std::cell::RefCell;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

fn reader_function() -> Result<String, String> {
    Ok("this is a dummy string".to_string())
}

#[test]
fn test_suite() {
    let mut failures = Vec::new();

    for folder_entry in fs::read_dir("./testsuite/").unwrap() {
        let unwrapped_folder_entry = folder_entry.unwrap();
        let folder_entry_path = unwrapped_folder_entry.path();

        if unwrapped_folder_entry.file_type().unwrap().is_dir() {
            for file_entry_result in fs::read_dir(folder_entry_path).unwrap() {
                let path = file_entry_result.unwrap().path();

                if Some(OsStr::new("mn")) == path.extension() {
                    let input_contents = read_text_contents(&path);

                    let buffer = Rc::new(RefCell::new(Vec::<u8>::new()));
                    let w = WriterObj::Buffer(Rc::clone(&buffer));
                    let r = ReaderObj { reader_function };
                    let env = match back::create_root_env(w, r) {
                        Ok(env) => env,
                        Err(_err) => panic!("Problem creating root env"),
                    };

                    let interpreter_output = macaroon::parse_eval_print(
                        env,
                        path.to_str().unwrap(),
                        input_contents.trim_right(),
                    );

                    let raw_buffer = buffer.borrow_mut().clone();
                    let buffer_output = String::from_utf8(raw_buffer).expect("Not UTF-8");
                    let total_output = format!("{}{}", buffer_output, interpreter_output);

                    if let Some(output_file_stem) = path.file_stem() {
                        let case: String = output_file_stem.to_str().unwrap().to_owned();
                        let testsuite_case_name = format!("{}", path.to_str().unwrap().to_owned());
                        let output_path = path.parent().unwrap().join(case.clone() + ".out");
                        let expected_output = read_text_contents(&output_path);

                        if expected_output.trim_right() != total_output.trim_right() {
                            failures.push((testsuite_case_name, expected_output, total_output));
                        }
                    }
                }
            }
        }
    }

    if failures.len() > 0 {
        println!("\n{}\n", "Macaroon Test Suite Failures".magenta().bold());

        let mut count: i32 = 1;
        for failure in failures {
            let (case_name, expected, actual) = failure;
            println!(
                "{}{} ({})\n\n   {}:\n\n\t{}\n\n   {}:\n\n\t{}\n\n",
                "Failure #".magenta().bold(),
                count.to_string().magenta().bold(),
                case_name.blue(),
                "Expected".bold(),
                expected.trim_right().green().bold(),
                "Actual".bold(),
                actual.red().bold(),
            );

            count += 1;
        }

        panic!("Test cases failed.");
    }
}

fn read_text_contents(path: &PathBuf) -> String {
    // Create a path to the desired file
    //let path = Path::new("testsuite/1.mn");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => {}
    }

    // `file` goes out of scope, and the "hello.txt" file gets closed

    s
}
