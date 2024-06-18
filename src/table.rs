use crate::Dependency;
use futures::{stream::iter, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use num_cpus::get;
use owo_colors::OwoColorize;
use reqwest::StatusCode;
use std::sync::{Arc, Mutex};
use tabled::{builder::Builder, settings::Style};

const NPM_URL: &str = "https://registry.npmjs.org/";

pub struct ResultTable {
    pub table: String,
    pub outdated_deps: Vec<String>,
}

#[tokio::main]
pub async fn get_report_table(deps: Vec<Dependency>, m: &MultiProgress) -> ResultTable {
    let pb = m.add(ProgressBar::new(deps.len().try_into().unwrap_or(0)));
    pb.set_style(ProgressStyle::default_spinner().tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
    pb.set_prefix(format!("{}", "[3/3]".bold().dimmed()));

    let client = reqwest::Client::new();

    let deps = deps.into_iter().map(|mut dep| {
        let pb = &pb;
        let client = &client;

        async move {
            pb.set_message(format!("{} {}", "Fetching".dimmed(), dep.get_name()));
            let url = format!("{}{}", NPM_URL, dep.get_name());
            let resp = client.get(&url).send().await.expect("Cannot send request");
            let status = resp.status();

            let latest_version = match status {
                StatusCode::OK => {
                    let resp = resp
                        .json::<serde_json::Value>()
                        .await
                        .expect("Cannot parse response");
                    resp["dist-tags"]["latest"].as_str().unwrap().to_string()
                }
                StatusCode::NOT_FOUND => {
                    panic!("{:?} not found", dep.get_name());
                }
                _ => panic!("Unhandled status code"),
            };

            let status = dep.compare(&latest_version);

            pb.inc(1);
            dep.set_latest_version(latest_version);
            dep.set_status(status.to_string());
            dep
        }
    });

    let deps = iter(deps).buffer_unordered(get());

    let outdated_deps = Arc::new(Mutex::new(Vec::new()));

    deps.for_each(|dep| {
        let outdated_deps = Arc::clone(&outdated_deps);
        async move {
            if let Some(status) = dep.get_status() {
                if status == "Outdated" {
                    outdated_deps.lock().unwrap().push(dep);
                }
            }
        }
    })
    .await;

    // push records
    let mut builder = Builder::default();
    builder.push_record(["Name", "Current", "Latest", "Status"]);

    let outdated_deps = outdated_deps.lock().unwrap().to_vec();
    for dep in outdated_deps.iter() {
        if *dep.get_status() != Some("Outdated".to_string()) {
            continue;
        }

        builder.push_record([
            dep.get_name(),
            dep.get_current_version(),
            dep.get_latest_version(),
            &"Outdated".red().to_string(),
        ]);
    }

    let outdated_deps = outdated_deps
        .into_iter()
        .map(|dep| {
            format!(
                "{}{}{} {} {}{}",
                dep.get_name().yellow(),
                "(".dimmed(),
                dep.get_current_version().dimmed(),
                "->".dimmed(),
                dep.get_latest_version().dimmed(),
                ")".dimmed()
            )
        })
        .collect::<Vec<String>>();
    let table = builder.build().with(Style::modern()).to_string();

    ResultTable {
        table,
        outdated_deps,
    }
}
