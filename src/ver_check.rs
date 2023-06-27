use tabled::{
    builder::Builder
};
use owo_colors::OwoColorize;
use tokio;
use reqwest::Client;
use version_compare::{Cmp, Version};

const NPM_URL: &str = "https://registry.npmjs.org/";


#[tokio::main]
pub async fn dependencies_version_check(
    deps_list: Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::default();
    let client = Client::new();
    let headers = ["Name", "Current", "Latest", "Status"];

    builder.set_header(headers);

    for (name, version) in deps_list {
        let latest_version = get_latest_version(&name, &client).await?;
        let current_version = Version::from(&version).unwrap();
        let latest_version = Version::from(&latest_version).unwrap();
        let status = current_version.compare(&latest_version);
        let status = match status {
            Cmp::Lt => "Outdated",
            Cmp::Eq => "Up to date",
            Cmp::Gt => "Newer than latest",
            _ => panic!("Unhandled comparison result"),
        };

        builder.push_record([
            name,
            current_version.to_string(),
            latest_version.to_string(),
            colorized_status(status),
        ]);
    }

    let table = builder
    .build()
    .to_string();

    println!("{}", table);

    Ok(())
}

async fn get_latest_version(name: &str, client: &Client) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", NPM_URL, name);
    let resp = client.get(&url).send().await?.json::<serde_json::Value>().await?;
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
