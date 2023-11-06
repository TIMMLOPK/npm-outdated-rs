use futures::{stream::iter, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use reqwest::{Client, StatusCode};
use std::sync::{Arc, Mutex};
use tabled::{builder::Builder, settings::Style};
use version_compare::{Cmp, Version};

const NPM_URL: &str = "https://registry.npmjs.org/";

struct Dependency {
    name: String,
    current_version: String,
    latest_version: Option<String>,
    status: Option<String>,
    dependencies_type: String,
}

impl Dependency {
    fn new(name: String, current_version: String, dependencies_type: String) -> Self {
        Self {
            name,
            current_version,
            dependencies_type,
            latest_version: None,
            status: None,
        }
    }

    fn from_tuple((name, current_version, dependencies_type): (String, String, String)) -> Self {
        Self {
            name,
            current_version,
            dependencies_type,
            latest_version: None,
            status: None,
        }
    }

    fn set_latest_version(&mut self, latest_version: String) {
        self.latest_version = Some(latest_version);
    }

    fn set_status(&mut self, status: String) {
        self.status = Some(status);
    }

    fn compare(&mut self, latest_version: &String) -> String {
        let current_version = Version::from(&self.current_version).unwrap();
        let latest_version = Version::from(&latest_version).unwrap();
        let status = match current_version.compare(&latest_version) {
            Cmp::Lt => "Outdated",
            Cmp::Eq => "Up to date",
            _ => panic!("Unknown status"),
        };

        status.to_string()
    }
}

struct DependencyChecker {
    client: Client,
    pb: ProgressBar,
    outdated_deps: Arc<Mutex<Vec<String>>>,
}

impl DependencyChecker {
    async fn check(&mut self, dep: Dependency) -> Dependency {
        self.pb
            .set_message(format!("{} {}", "Checking".dimmed(), dep.name));
        let url = format!("{}{}", NPM_URL, dep.name);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .expect("Cannot send request");
        let status = resp.status();

        let (name, current_version, latest_version) = match status {
            StatusCode::OK => {
                let resp = resp
                    .json::<serde_json::Value>()
                    .await
                    .expect("Cannot parse response");
                let latest_version = resp["dist-tags"]["latest"].as_str().unwrap().to_string();
                (dep.name, dep.current_version, latest_version)
            }
            StatusCode::NOT_FOUND => {
                panic!("{:?} not found", dep.name);
            }
            _ => panic!("Unhandled status code"),
        };

        let mut dep = Dependency::new(name, current_version, dep.dependencies_type);

        let status = dep.compare(&latest_version);

        if status == "Outdated" {
            self.outdated_deps.lock().unwrap().push(format!(
                "{}{}{} {} {}{}",
                dep.name.yellow(),
                "(".dimmed(),
                dep.current_version.dimmed(),
                "->".dimmed(),
                latest_version.dimmed(),
                ")".dimmed()
            ));
        }

        dep.set_latest_version(latest_version);
        dep.set_status(status);

        dep
    }
}

#[tokio::main]
pub async fn get_report_table(
    deps_list: &Vec<(String, String, String)>,
    m: &MultiProgress,
) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    let mut builder = Builder::default();
    let client = Client::new();
    let headers = ["Name", "Current", "Latest", "Status"];
    builder.set_header(headers);

    let pb = m.add(ProgressBar::new(deps_list.len().try_into().unwrap_or(0)));
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .expect("Cannot create spinner style")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    pb.set_style(spinner_style);
    pb.set_prefix(format!("{}", "[3/3]".bold().dimmed()));

    let outdated_deps = Arc::new(Mutex::new(Vec::new()));

    let max_concurrent = num_cpus::get();
    let stream = iter(deps_list.iter().cloned())
        .map(|x| Dependency::from_tuple(x))
        .map(|dep| {
            let mut checker = DependencyChecker {
                client: client.clone(),
                pb: pb.clone(),
                outdated_deps: Arc::clone(&outdated_deps),
            };
            async move { checker.check(dep).await }
        })
        .buffer_unordered(max_concurrent);

    stream
        .for_each(|dep| {
            builder.push_record(vec![
                dep.name,
                dep.current_version,
                dep.latest_version.unwrap(),
                colorized_status(dep.status.as_ref().unwrap()),
            ]);
            async {}
        })
        .await;

    let table = builder.build().with(Style::modern()).to_string();

    let outdated_deps = outdated_deps
        .lock()
        .expect("Cannot get outdated deps")
        .clone();

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
