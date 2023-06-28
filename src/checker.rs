use futures::{stream::iter, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::Client;
use tabled::builder::Builder;
use tokio;
use version_compare::{Cmp, Version};

const NPM_URL: &str = "https://registry.npmjs.org/";

#[tokio::main]
pub async fn check_version(
    deps_list: Vec<(String, String)>,
    m: &MultiProgress,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut builder = Builder::default();
    let client = Client::new();
    let headers = ["Name", "Current", "Latest", "Status"];
    let pb = m.add(ProgressBar::new(deps_list.len() as u64));
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    pb.clone().with_style(spinner_style.clone());
    pb.set_prefix(format!("{}", "[3/3]".bold().dimmed()));

    builder.set_header(headers);

    let stream = iter(deps_list.into_iter().map(|(name, version)| async {
        pb.set_message(format!("Checking {}...", name));
        let latest_version = get_package(&name, &client).await?;
        let current_version = Version::from(&version).unwrap();
        let latest_version = Version::from(&latest_version).unwrap();
        let status = match current_version.compare(&latest_version) {
            Cmp::Lt => "Outdated",
            Cmp::Eq => "Up to date",
            Cmp::Gt => "Newer than latest",
            _ => panic!("Unhandled status"),
        };

        pb.inc(1);

        Ok::<_, Box<dyn std::error::Error>>((
            name,
            version,
            latest_version.to_string(),
            status.to_string(),
        ))
    }))
    .buffer_unordered(10)
    .collect::<Vec<_>>();

    pb.finish_with_message("waiting...");

    let stream = stream.await;

    for x in stream {
        match x {
            Ok(x) => {
                builder.push_record(vec![x.0, x.1, x.2, colorized_status(&x.3)]);
            }
            Err(e) => panic!("Error: {}", e),
        }
    }

    let table = builder.build().to_string();
    Ok(table)
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
