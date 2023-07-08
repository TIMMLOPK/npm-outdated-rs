use futures::{stream::iter, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use tabled::{builder::Builder, settings::Style};
use tokio::sync::Semaphore;
use version_compare::{Cmp, Version};

const NPM_URL: &str = "https://registry.npmjs.org/";

#[tokio::main]
pub async fn get_report_table(
    deps_list: &Vec<(String, String, String)>,
    m: &MultiProgress,
) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    let mut builder = Builder::default();
    let client = Client::new();
    let headers = ["Name", "Current", "Latest", "Status"];
    builder.set_header(headers);

    let pb = m.add(ProgressBar::new(deps_list.len() as u64));
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    pb.clone().with_style(spinner_style.clone());
    pb.set_prefix(format!("{}", "[3/3]".bold().dimmed()));

    let mut outdated_deps = Vec::new();

    let max_concurrent = num_cpus::get();
    let sem = Arc::new(Semaphore::new(max_concurrent));
    let stream = iter(
        deps_list
            .clone()
            .into_iter()
            .map(|(name, version, _)| async {
                pb.set_message(format!("Checking {}...", name));
                let permit = sem.acquire().await.unwrap();
                let latest_version = get_package(&name, &client).await?;
                let current_version = Version::from(&version).unwrap();
                let latest_version = Version::from(&latest_version).unwrap();
                let status = match current_version.compare(&latest_version) {
                    Cmp::Lt => "Outdated",
                    Cmp::Eq => "Up to date",
                    _ => panic!("Unknown status"),
                };

                pb.inc(1);

                drop(permit);

                Ok::<_, Box<dyn std::error::Error>>((
                    name,
                    version,
                    latest_version.to_string(),
                    status.to_string(),
                ))
            }),
    )
    .buffer_unordered(max_concurrent)
    .collect::<Vec<_>>();

    pb.finish_with_message("waiting...");

    let stream = stream.await;

    for x in stream {
        match x {
            Ok(x) => {
                match x.3.as_str() {
                    "Outdated" => {
                        outdated_deps.push(format!(
                            "{}{}{} {} {}{}",
                            x.0.yellow(),
                            "(".dimmed(),
                            x.1.dimmed(),
                            "->".dimmed(),
                            x.2.dimmed(),
                            ")".dimmed()
                        ));
                    }
                    _ => {}
                }
                builder.push_record(vec![x.0, x.1, x.2, colorized_status(&x.3)]);
            }
            Err(e) => {
                if e.downcast_ref::<reqwest::Error>()
                    .map_or(false, |e| e.status() == Some(StatusCode::NOT_FOUND))
                {
                    pb.inc(1);
                    continue;
                }
                panic!("Error: {}", e);
            }
        }
    }

    let table = builder.build().with(Style::modern()).to_string();

    Ok((table, outdated_deps))
}

async fn get_package(name: &str, client: &Client) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", NPM_URL, name);
    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let latest_version = resp["dist-tags"]["latest"].to_string();
    let latest_version = latest_version.replace("\"", "");
    Ok(latest_version)
}

fn colorized_status(status: &str) -> String {
    match status {
        "Outdated" => status.red().to_string(),
        "Up to date" => status.green().to_string(),
        "Newer than latest" => status.yellow().to_string(),
        _ => panic!("Unhandled status"),
    }
}
