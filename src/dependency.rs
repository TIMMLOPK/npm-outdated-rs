use version_compare::{Cmp, Version};

#[derive(Clone)]
pub struct Dependency {
    name: String,
    current_version: String,
    latest_version: String,
    status: Option<String>,
}

impl Dependency {
    pub fn new(name: String, current_version: String) -> Self {
        Self {
            name,
            current_version,
            latest_version: String::new(),
            status: None,
        }
    }

    pub fn set_latest_version(&mut self, latest_version: String) {
        self.latest_version = latest_version;
    }

    pub fn set_status(&mut self, status: String) {
        self.status = Some(status);
    }

    pub fn compare(&mut self, latest_version: &String) -> String {
        let current_version = Version::from(&self.current_version).unwrap();
        let latest_version = Version::from(&latest_version).unwrap();
        let status = match current_version.compare(&latest_version) {
            Cmp::Lt => "Outdated",
            Cmp::Eq => "Up to date",
            Cmp::Gt => "Newer than latest",
            _ => panic!("Unknown status"),
        };

        status.to_string()
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_current_version(&self) -> &String {
        &self.current_version
    }

    pub fn get_latest_version(&self) -> &String {
        &self.latest_version
    }

    pub fn get_status(&self) -> &Option<String> {
        &self.status
    }
}
