#![allow(unused)]

use clap::Parser;
use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use std::{fs, panic, thread};
use tokio;
use version_compare::{compare, compare_to, Cmp, Version};

#[derive(Parser)]
struct Cli {
    check_lastest: bool,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    dependencies: serde_json::Value,
    #[serde(alias = "devDependencies")]
    dev_dependencies: serde_json::Value,
}

fn main() {
    let args = Cli::parse();
    let pb = indicatif::ProgressBar::new(1);
    let package_json = std::fs::read_to_string("package.json").expect("package.json not found");
    println!("Found package.json file");

    for i in 0..1 {
        pb.inc(1);
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

    println!(
        "Found {} dependencies",
        package
            .dependencies
            .as_object()
            .expect("unknown dependencies")
            .len()
    );
    for (key, value) in package
        .dependencies
        .as_object()
        .expect("unknown dependencies")
        .iter()
    {
        println!("{}: {}", key, value);
    }
    println!(
        "Found {} devDependencies",
        package
            .dev_dependencies
            .as_object()
            .expect("unknown devDependencies")
            .len()
    );

    for (key, value) in package
        .dev_dependencies
        .as_object()
        .expect("unknown devDependencies")
        .iter()
    {
        println!("{}: {}", key, value);
    }
    if check_lastest {
        dependencies_version_check(&package.dependencies, &package.dev_dependencies);
    }
}

const CONCURRENT_REQUESTS: usize = 2;

#[tokio::main]
async fn dependencies_version_check(
    dependencies: &serde_json::Value,
    dev_dependencies: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut res = Vec::new();
    let mut deps_list = Vec::new();

    let pb = indicatif::ProgressBar::new(
        dependencies.as_object().unwrap().len() as u64
            + dev_dependencies.as_object().unwrap().len() as u64,
    );

    for (key, value) in dependencies.as_object().expect("failed to fetch").iter() {
        let url = format!("https://registry.npmjs.org/{}", key);
        deps_list.push((key.to_string(), value.to_string(), url));
    }
    for (key, value) in dev_dependencies
        .as_object()
        .expect("failed to fetch")
        .iter()
    {
        let url = format!("https://registry.npmjs.org/{}", key);
        deps_list.push((key.to_string(), value.to_string(), url));
    }

    let bodies = stream::iter(deps_list)
        .map(|(key, value, url)| {
            let client = &client;
            async move {
                let res = client
                    .get(&url)
                    .send()
                    .await
                    .expect("failed to send request");
                let body = res.text().await.expect("failed to read body");
                (key, value, body)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    for (key, value, body) in bodies {
        let current = Version::from(&value).expect("failed to parse version");
        let json: serde_json::Value = serde_json::from_str(&body).expect("failed to parse json");
        let version = json["dist-tags"]["latest"]
            .as_str()
            .expect("failed to parse version");
        let latest = Version::from(version).expect("failed to parse version");
        if current < latest {
            res.push(format!("{} {} < {}", key, current, latest));
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    println!("Found {} outdated dependencies", res.len());
    for res in res {
        output_color(&res);
    }
    Ok(())
}

fn output_color(s: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(b"\x1b[1;31m")?;
    stdout.write_all(s.as_bytes())?;
    stdout.write_all(b"\x1b[0m")?;
    stdout.write_all(b"\n")?;
    Ok(())
}
