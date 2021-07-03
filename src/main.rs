extern crate notify;

use chrono::{NaiveDateTime};
use notify::{DebouncedEvent, RecommendedWatcher, Watcher, RecursiveMode};
use std::env;
use std::fs;
use std::sync::mpsc::channel;
use std::time::Duration;

fn print_latest_log(log_path: String, line_cnt: i32) -> i32 {
    let contents = match fs::read_to_string(log_path) {
        Ok(value) => value,
        Err(error) => panic!("{}", error.to_string())
    };

    let lines = contents.lines();
    let mut curr_line_cnt = 0;

    for line in lines {
        // don't output lines we've already output, only output the newest lines
        curr_line_cnt = curr_line_cnt + 1;

        if line.contains("<>") && curr_line_cnt > line_cnt {
            println!("{}", line);
        }
    }

    line_cnt + curr_line_cnt
}

fn watch(watch_path: String) -> notify::Result<()> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = match Watcher::new(tx, Duration::from_secs(2)) {
        Ok(value) => value,
        Err(error) => panic!("{}", error)
    };

    match watcher.watch(watch_path.clone(), RecursiveMode::Recursive) {
        Ok(value) => value,
        Err(error) => panic!("{}", error)
    };

    // maintain a count of all the lines read so we only output the newest lines
    // probably not the most efficient way to do this but it works
    let mut line_cnt: i32 = 0;

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Chmod(_) => (),
                    DebouncedEvent::Create(_) => (),
                    DebouncedEvent::Error(error, _path) => panic!("{}", error),
                    DebouncedEvent::NoticeRemove(_) => (),
                    DebouncedEvent::Remove(_) => (),
                    DebouncedEvent::Rename(_, _) => (),
                    DebouncedEvent::Rescan => (),
                    DebouncedEvent::NoticeWrite(_) => {
                        line_cnt = print_latest_log(watch_path.clone(), line_cnt);
                    },
                    DebouncedEvent::Write(_) => ()
                }
            },
            Err(error) => println!("watch error: {:?}", error),
        }
    }
}

fn main() {
    let home = match env::var("HOME") {
        Ok(value) => value,
        Err(_) => panic!("HOME environment variable is not set")
    };
    let log_location: &str = &format!("{}/.avorion", home)[..];

    if let Ok(paths) = fs::read_dir(log_location) {
        let mut files: Vec<String> = vec![];

        // collect all files that begin with `clientlog`
        for path in paths {
            if let Ok(path) = path {
                let file_name = match path.file_name().into_string() {
                    Ok(value) => value,
                    Err(error) => panic!("{:?}", error)
                };

                if file_name.starts_with("clientlog") {
                    files.push(file_name);
                }
            }
        }

        // from the list of `clientlog` files, find the newest file
        let most_recent = files.iter().reduce(|first, second| {
            let dte1_str = &first[10..29];
            let dte1 = match NaiveDateTime::parse_from_str(dte1_str, "%Y-%m-%d %H-%M-%S") {
                Ok(value) => value,
                Err(error) => panic!("{}", error)
            };

            let dte2_str = &second[10..29];
            let dte2 = match NaiveDateTime::parse_from_str(dte2_str, "%Y-%m-%d %H-%M-%S") {
                Ok(value) => value,
                Err(error) => panic!("{}", error)
            };

            if dte1 > dte2 { first } else { second }
        });

        let most_recent_str: String = match most_recent {
            None => String::from(""),
            Some(value) => value.to_string()
        };

        let most_recent_path = format!("{}/{}", log_location, most_recent_str);

        println!("Reading: {}", most_recent_path);

        if let Err(error) = watch(most_recent_path) {
            panic!("{}", error);
        }
    }
}
