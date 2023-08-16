use futures::{stream::iter, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::{Client, StatusCode};
use tabled::{builder::Builder, settings::Style};
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
    pb.set_style(spinner_style);
    pb.set_prefix(format!("{}", "[3/3]".bold().dimmed()));

    let mut outdated_deps = Vec::new();

    let max_concurrent = num_cpus::get();
    let mut stream = iter(deps_list.iter().cloned())
        .map(|x| {
            let client = client.clone();
            let pb = pb.clone();
            tokio::spawn(async move {
                pb.set_message(format!("{} {}", "Checking".dimmed(), x.0));
                let name = x.0;
                let current_version = x.1;
                let url = format!("{}{}", NPM_URL, name);
                let resp = client.get(&url).send().await.unwrap();
                let status = resp.status();

                match status {
                    StatusCode::OK => {
                        let resp = resp.json::<serde_json::Value>().await.unwrap();
                        let latest_version =
                            resp["dist-tags"]["latest"].as_str().unwrap().to_string();
                        (name, current_version, latest_version)
                    }
                    StatusCode::NOT_FOUND => (name, current_version, "Not found".to_string()),
                    _ => (name, current_version, "Unknown".to_string()),
                }
            })
        })
        .buffer_unordered(max_concurrent);

    while let Some(x) = stream.next().await {
        let x = x.unwrap();
        let current_version = Version::from(&x.1).unwrap();
        let latest_version = Version::from(&x.2).unwrap();
        let status = match current_version.compare(&latest_version) {
            Cmp::Lt => "Outdated",
            Cmp::Eq => "Up to date",
            _ => panic!("Unknown status"),
        };

        match status {
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

        builder.push_record(vec![x.0, x.1, x.2, colorized_status(&status)]);
    }

    let table = builder.build().with(Style::modern()).to_string();

    Ok((table, outdated_deps))
}

fn colorized_status(status: &str) -> String {
    match status {
        "Outdated" => status.red().to_string(),
        "Up to date" => status.green().to_string(),
        "Newer than latest" => status.yellow().to_string(),
        _ => panic!("Unhandled status"),
    }
}
