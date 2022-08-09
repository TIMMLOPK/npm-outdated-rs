use futures::stream;
use futures::StreamExt;
use reqwest::Client;
use std::io::{stdout, Write};
use tokio;
use version_compare::Version;

const CONCURRENT_REQUESTS: usize = 3;

#[tokio::main]
pub async fn dependencies_version_check(
    dependencies: &serde_json::Value,
    dev_dependencies: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut res = Vec::new();
    let mut deps_list = Vec::new();

    let total: u64 = dependencies
        .as_object()
        .expect("unknown dependencies")
        .len() as u64
        + dev_dependencies
            .as_object()
            .expect("unknown dependencies")
            .len() as u64;

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
    let pb = indicatif::ProgressBar::new(total);

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

    for i in res {
        output_color(&i).expect("failed to write to stdout");
    }
    Ok(())
}

fn output_color(s: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(b"\x1b[1;31m")?;
    stdout.write_all(s.as_bytes())?;
    stdout.write_all(b"\x1b[0m")?;
    stdout.write_all(b"\n")?;
    Ok(())
}
