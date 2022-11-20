use prettytable::{color, format, Attr, Cell, Table};
use tokio;
use version_compare::Version;

const NPM_URL: &str = "https://registry.npmjs.org/";

#[tokio::main]
pub async fn dependencies_version_check(
    deps_list: Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total: u64 = deps_list.len() as u64;
    let pb = indicatif::ProgressBar::new(total);
    pb.set_style(indicatif::ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.green/white}] {pos}/{len} ({eta})",
    )?);
    let mut table = Table::new();

    table.set_titles(row!["Name", "Current", "Latest", "Status"]);
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    for (key, value) in deps_list {
        let res: serde_json::Value = reqwest::get(&format!("{}{}", NPM_URL, key))
            .await?
            .json()
            .await?;
        let latest_version = res["dist-tags"]["latest"].as_str().unwrap();
        let latest_version = Version::from(latest_version).unwrap();
        let current_version = Version::from(value.trim_matches('"')).unwrap();
        let status = match latest_version.compare(&current_version) {
            version_compare::Cmp::Gt => "Outdated",
            version_compare::Cmp::Eq => "Up to date",
            version_compare::Cmp::Lt => "Downgraded",
            _ => "Unknown",
        };

        let status = match status {
            "Up to date" => Cell::new(status).with_style(Attr::ForegroundColor(color::GREEN)),
            "Outdated" => Cell::new(status).with_style(Attr::ForegroundColor(color::RED)),
            _ => Cell::new(status).with_style(Attr::ForegroundColor(color::YELLOW)),
        };

        table.add_row(row![key, current_version, latest_version, status]);

        pb.inc(1);
    }

    pb.finish_and_clear();
    table.printstd();
    Ok(())
}
