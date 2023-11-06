use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

use crate::utils::strip_ansi;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "devDependencies")]
    pub dev_dependencies: Option<serde_json::Value>,
    #[serde(flatten)]
    pub other: serde_json::Value,
}

impl PackageJson {
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        if !path.as_ref().exists() {
            return Err(format!("File {} not found", path.as_ref().display()).into());
        }

        let file = File::open(&path);

        let reader = BufReader::new(file.expect("Cannot create file"));
        let package = serde_json::from_reader(reader);

        Ok(package.expect("Cannot parse package.json"))
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(&path);

        let writer = BufWriter::new(file.expect("Cannot create file"));

        serde_json::to_writer_pretty(writer, self).expect("Cannot write to file");

        Ok(())
    }

    pub fn update_dependencies_version(
        &mut self,
        dependencies: Vec<&str>,
        path: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut deps = self
            .dependencies
            .as_ref()
            .unwrap_or(&serde_json::Value::Object(serde_json::Map::new()))
            .clone();
        let mut dev_deps = self
            .dev_dependencies
            .as_ref()
            .unwrap_or(&serde_json::Value::Object(serde_json::Map::new()))
            .clone();

        for dep in dependencies {
            let dep = strip_ansi(dep);
            let name = dep.split("(").next().expect("Invalid dependency name");
            let old_version = dep
                .split("(")
                .last()
                .expect("Invalid dependency version")
                .split("->")
                .next()
                .expect("Invalid dependency version");
            let old_version = old_version.replace("\"", "");
            let latest_version = dep
                .split("->")
                .last()
                .expect("Invalid dependency version")
                .replace(")", "");
            let prefix = match old_version.chars().next() {
                Some('^') => "^",
                Some('~') => "~",
                _ => "",
            };

            let new_version = format!("{}{}", prefix, latest_version.trim());

            if deps.get(&name).is_some() {
                deps[&name] = serde_json::Value::String(new_version);
            } else if dev_deps.get(&name).is_some() {
                dev_deps[&name] = serde_json::Value::String(new_version);
            }
        }

        if deps != serde_json::Value::Object(serde_json::Map::new()) {
            self.dependencies = Some(deps);
        }

        if dev_deps != serde_json::Value::Object(serde_json::Map::new()) {
            self.dev_dependencies = Some(dev_deps);
        }

        self.write_to_file(path).expect("Cannot write to file");

        Ok(())
    }
}
