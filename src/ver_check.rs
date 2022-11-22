use prettytable::{color, format, Attr, Cell, Table};
use rayon::prelude::*;
use tokio;
use version_compare::Version;

const NPM_URL: &str = "https://registry.npmjs.org/";

#[tokio::main]
pub async fn dependencies_version_check(
    deps_list: Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut table_print = Table::new();

    table_print.set_titles(row!["Name", "Current", "Latest", "Status"]);
    table_print.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    let table = deps_list
        .par_iter()
        .map(|(name, version)| async move {
            let latest_version = get_latest_version(name).await.unwrap();
            let current_version = Version::from(version).unwrap();
            let latest_version = Version::from(&latest_version).unwrap();

            let status = match latest_version.compare(&current_version) {
                version_compare::Cmp::Gt => "Outdated",
                version_compare::Cmp::Eq => "Up to date",
                version_compare::Cmp::Lt => "Downgraded",
                _ => "Unknown",
            };

            let status = match status {
                "Outdated" => Cell::new(status).with_style(Attr::ForegroundColor(color::RED)),
                "Up to date" => Cell::new(status).with_style(Attr::ForegroundColor(color::GREEN)),
                _ => Cell::new(status),
            };
            row![name, version, latest_version, status]
        })
        .collect::<Vec<_>>();

    let table = futures::future::join_all(table).await;

    for row in table {
        table_print.add_row(row);
    }

    table_print.printstd();
    Ok(())
}

async fn get_latest_version(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}{}", NPM_URL, name);
    let resp = reqwest::get(&url)
        .await?
        .json::<serde_json::Value>()
        .await?;
    let latest_version = resp["dist-tags"]["latest"].as_str().unwrap();
    Ok(latest_version.to_string())
}
