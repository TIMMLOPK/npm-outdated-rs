#![allow(unused)]

use std::io::prelude::*;
use clap::Parser;
use std::io::BufReader;
use std::time::{Duration, Instant};
use serde::Deserialize;
use version_compare::{compare, compare_to, Cmp, Version};
use std::io::{stdout, Write};

#[derive(Parser)]
struct Cli {
    check_lastest: bool,
}


#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: serde_json::Value,
}


fn main() {
    let args = Cli::parse();
    let pb = indicatif::ProgressBar::new(1);
    let package_json = std::fs::read_to_string("package.json").expect("package.json not found");
    println!("Found package.json file");

    for i in 0..1 {
        pb.inc(1);
        std::thread::sleep(Duration::from_millis(3));
    }
    pb.finish_and_clear();
    read_file(args.check_lastest);
}

fn read_file(check_lastest: bool) {
    let file = std::fs::File::open("package.json").expect("file not found");
    let reader = BufReader::new(file);

    let package: Package = serde_json::from_reader(reader).expect("error parsing json");
    
    println!("Name: {}", package.name);
    println!("Version: {}", package.version);

    println!("Found {} dependencies", package.dependencies.as_object().expect("unknown dependencies").len());
    for (key, value) in package.dependencies.as_object().expect("unknown dependencies").iter() {
        println!("{}: {}", key, value);
    }
    if check_lastest {
        dependencies_version_check(&package.dependencies);
    }
}


#[tokio::main]
async fn dependencies_version_check(dependencies: &serde_json::Value) -> Result<(),Box<dyn std::error::Error>> {
    let mut res = Vec::new();
    let pb = indicatif::ProgressBar::new(dependencies.as_object().expect("unknown dependencies").len() as u64);
    for (key, value) in dependencies.as_object().expect("error parsing json").iter() {
        pb.inc(1);
        let url = format!("https://registry.npmjs.org/{}", key);
        let resp = reqwest::get(&url).await?;
        let body = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body)?;
        let version = json["dist-tags"]["latest"].as_str().expect("error parsing json");

        let current_version = Version::from(value.as_str().expect("error parsing json")).expect("error parsing json");
        let latest_version = Version::from(version).expect("error parsing json");
        if current_version < latest_version {
            res.push(format!("Latest version of {} is {}", key, version));
        }

    }
    pb.finish_and_clear();
    println!("Found {} outdated dependencies", res.len());
    for i in res {
        output_color(&i);
    }
    Ok(())
}

fn output_color(s: &str) ->Result<(),Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(b"\x1b[1;31m")?;
    stdout.write_all(s.as_bytes())?;
    stdout.write_all(b"\x1b[0m")?;
    stdout.write_all(b"\n")?;
    Ok(())
}
