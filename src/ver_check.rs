use futures::stream;
use futures::StreamExt;
use prettytable::{color, format, Attr, Cell, Row, Table};
use reqwest::Client;
use tokio;
use version_compare::Version;

const CONCURRENT_REQUESTS: usize = 3;

#[tokio::main]
pub async fn dependencies_version_check(
    deps_list: Vec<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut res = Vec::new();

    let total: u64 = deps_list.len() as u64;
    let pb = indicatif::ProgressBar::new(total);
    pb.inc(0);

    let bodies = stream::iter(deps_list)
        .map(|(key, value)| {
            let client = &client;
            async move {
                let res = client
                    .get(&format!("https://registry.npmjs.org/{}", key))
                    .send()
                    .await
                    .expect("failed to send request");
                let body = res.text().await.expect("failed to read body");
                let new = serde_json::from_str::<serde_json::Value>(&body)
                    .expect("failed to parse json")
                    .get("dist-tags")
                    .unwrap()
                    .get("latest")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                (key, value, new)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    for (key, value, new) in bodies {
        let current = Version::from(&value).unwrap();
        let latest = Version::from(&new).unwrap();

        if current < latest {
            res.push((key, value, new));
        }
    }

    pb.finish_and_clear();

    println!("Found {} outdated dependencies", res.len());

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.add_row(row!["Package", "Current", Fr-> "Latest"]);

    for i in res {
        table.add_row(Row::new(vec![
            Cell::new(&i.0),
            Cell::new(&i.1.to_string()),
            Cell::new(&i.2.to_string()).with_style(Attr::ForegroundColor(color::RED)),
        ]));
    }

    table.printstd();

    Ok(())
}
