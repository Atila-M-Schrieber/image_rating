// use std::io::Write;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::process::Command;

use colored::Colorize;
use csv;
use rand::seq::SliceRandom;
use rand::Rng;

fn rate(ratings: &mut HashMap<String, (f32, usize)>, k: f32, min_score: f32) {
    let mut just_called = true;
    println!("No ratings file found.");
    loop {
        // randomly select two images
        let mut temp_vec: Vec<(&String, &(f32, usize))> = ratings
            .iter()
            .filter(|(_, (score, _))| score >= &min_score) // only viable pictures
            .collect();

        let num_images = temp_vec.len();

        if just_called {
            just_called = false;
            println!("Pictures to rate: {num_images}");
        }

        temp_vec.sort_by(|a, b| a.1 .1.cmp(&b.1 .1)); // sort by number of games played

        let max_games = temp_vec[temp_vec.len() / 3].1 .1; // most of lower third's number of games played

        // only be able to choose pics which has been picked at most as many times as the most
        // selected image bottom third of the images
        // This ensures images are picked approximately evenly
        use std::cmp::Ordering::*;

        let mut temp_prob = 1.;
        let temp_vec: Vec<String> = temp_vec
            .into_iter()
            .filter_map(|(k, (_, games))| {
                match games.cmp(&max_games) {
                    Less => Some(k.to_owned()),
                    Equal => {
                        // if it's at the max, randomly let it be included
                        if rand::thread_rng().gen_bool(temp_prob) {
                            temp_prob -= 0.1;
                            Some(k.to_owned())
                        } else {
                            None
                        }
                    }
                    Greater => None,
                }
            })
            .collect();

        let image_names: Vec<&String> = temp_vec
            .choose_multiple(&mut rand::thread_rng(), 2)
            .collect();

        // open images in sxiv
        let mut sxiv = Command::new("sxiv")
            .args(image_names.clone())
            .spawn()
            .expect(format!("Sxiv could not open these images ({:?})", image_names).as_str());

        // get user input: l(eft), r(ight), d(raw), q(uit)
        println!("Select winner: l(eft), r(ight), d(raw), q(uit): ");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let result = match input.as_str().trim() {
            "l" => 1.,
            "a" => 1., // for convenience to use left hand for keyboard, right for mouse
            "r" => 0.,
            "d" => 0.5,
            "q" => {
                sxiv.kill().expect("Can't rid myself of sxiv");
                break;
            }
            _ => {
                println!("Invalid input.");
                continue;
            }
        };

        sxiv.kill().expect("Can't rid myself of sxiv");

        // do Élő calculations

        let (mut score1, games1) = ratings.remove(image_names[0]).unwrap();
        let (mut score2, games2) = ratings.remove(image_names[1]).unwrap();

        // Additional time penalty - more games played, harder to stay above 1100

        let penalty = ((games1 as f32 + games2 as f32) / 2.).sqrt();

        let expected_1 = 1. / (1. + 10_f32.powf((score2 - score1) / 400.));
        let expected_2 = 1. / (1. + 10_f32.powf((score1 - score2) / 400.));

        println!("Old scores: l: {score1}, r: {score2}");

        score1 += k * (result - expected_1) - (1. - result) * penalty;
        score2 += k * (1. - result - expected_2) - result * penalty;

        let mut maybe_info = String::new();

        println!(
            "New scores: l: {}, r: {} - Penalty: {penalty}{}",
            if score1 < min_score {
                maybe_info = format!("; Images left: {}", num_images - 1);
                score1.to_string().red()
            } else {
                score1.to_string().normal()
            },
            if score2 < min_score {
                maybe_info = format!("; Images left: {}", num_images - 1);
                score2.to_string().red()
            } else {
                score2.to_string().normal()
            },
            maybe_info,
        );
        println!("");

        // set new elo ratings, but only keep using them if above 1100 Élő
        ratings.insert(image_names[0].to_owned(), (score1, games1 + 1));
        ratings.insert(image_names[1].to_owned(), (score2, games2 + 2));
    }
}

fn main() -> std::io::Result<()> {
    const INITIAL_ELO: f32 = 1200.;

    // max score change, defaults to 40
    let k: f32 = match env::var("K") {
        Ok(k) => {
            println!("K set to {}", k);
            k.parse()
                .expect("K environment variable can't be parsed as f32")
        }
        _ => {
            println!("K set to 40 (default), K environment variable not found.");
            40.
        }
    };

    // minimum score to be considered, defaults to 1100
    let min_score: f32 = match env::var("MIN_SCORE") {
        Ok(min_score) => {
            println!("MIN_SCORE set to {}", min_score);
            min_score
                .parse()
                .expect("MIN_SCORE environment variable can't be parsed as f32")
        }
        _ => {
            println!("MIN_SCORE set to 1100 (default), MIN_SCORE environment variable not found.");
            1100.
        }
    };

    let mut ratings: HashMap<String, (f32, usize)> = HashMap::new();

    // get ratings from csv file
    let rdr = csv::Reader::from_path("ratings.csv");

    match rdr {
        Ok(mut rdr) => {
            for record in rdr.records() {
                let record = record?;
                if !std::path::Path::new(&record[0]).exists() {
                    panic!(
                        "File {} does not exist, but is found in the csv file, please fix!",
                        &record[0]
                    );
                }
                ratings.insert(
                    record[0].to_string(),
                    (
                        record[1].parse().expect("Can't parse score as f32"),
                        record[2]
                            .parse()
                            .expect("Can't parse number of games as usize"),
                    ),
                );
            }
        }
        _ => println!("No ratings file found."),
    }

    // populate hash map with jpg's - does not overwrite existing data from csv, if it exists
    for entry in fs::read_dir(".")? {
        let path = entry?.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jpg") {
            ratings
                .entry(path.to_string_lossy().to_string())
                .or_insert((INITIAL_ELO, 0));
        }
    }

    // perform the rating
    rate(&mut ratings, k, min_score);

    //write to csv
    let mut wrt = csv::Writer::from_path("ratings.csv")?;
    wrt.write_record(&["Path", "Rating", "Games"])?;

    for (path, rating) in &ratings {
        wrt.write_record(&[path, &rating.0.to_string(), &rating.1.to_string()])?;
    }

    wrt.flush()?;

    Ok(())
}
