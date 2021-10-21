static USER_AGENT: &str = "User-Agent: mite.app/v1.1 (https://github.com/yolk); mite-rs/0.0.1";

use std::process::exit;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
struct CustomerFields {
    id: u32,
    name: String,
    archived: bool,
}

#[derive(Deserialize, Serialize, Debug)]
struct Customer {
    customer: CustomerFields,
}

impl std::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.customer.name)
    }
}

type Customers = Vec<Customer>;

#[derive(Deserialize, Serialize, Debug)]
struct ProjectFields {
    id: u32,
    name: String,
    note: String,
    customer_id: u32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Project {
    project: ProjectFields,
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project.name)
    }
}

type Projects = Vec<Project>;

#[derive(Deserialize, Serialize, Debug)]
struct ServiceFields {
    id: u32,
    name: String,
    archived: bool,
    billable: bool,
    note: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Service {
    service: ServiceFields,
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.service.name)
    }
}

type Services = Vec<Service>;

fn main() {
    let api_key = std::env::var("MITE_API_KEY").unwrap();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "X-MiteApiKey",
        reqwest::header::HeaderValue::from_str(&api_key).unwrap(),
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .default_headers(headers)
        .build()
        .unwrap();

    let res: Customers = client
        .get("https://simplabs.mite.yo.lk/customers.json")
        .send()
        .unwrap()
        .json()
        .unwrap();
    let selection = dialoguer::Select::new()
        .with_prompt("Select customer")
        .items(&res)
        .default(0)
        .interact()
        .unwrap();
    let customer = res.get(selection).unwrap();

    let res: Projects = client
        .get("https://simplabs.mite.yo.lk/projects.json")
        .send()
        .unwrap()
        .json::<Vec<Project>>()
        .unwrap()
        .into_iter()
        .filter(|proj| proj.project.customer_id == customer.customer.id)
        .collect();
    let selection = dialoguer::Select::new()
        .with_prompt("Select project")
        .items(&res)
        .default(0)
        .interact()
        .unwrap();
    let project = res.get(selection).unwrap();

    let res: Services = client
        .get("https://simplabs.mite.yo.lk/services.json")
        .send()
        .unwrap()
        .json()
        .unwrap();
    let selection = dialoguer::Select::new()
        .with_prompt("Select project")
        .items(&res)
        .default(0)
        .paged(true)
        .interact()
        .unwrap();
    let service = res.get(selection).unwrap();

    let hours: u32 = dialoguer::Input::new()
        .with_prompt("Hours")
        .default(8)
        .interact()
        .unwrap();
    let note: String = dialoguer::Input::new()
        .with_prompt("Note")
        .allow_empty(true)
        .interact()
        .unwrap();

    #[derive(Serialize, Deserialize)]
    struct TimeEntryFields {
        minutes: u32,
        project_id: u32,
        service_id: u32,
        note: String,
    }

    #[derive(Serialize, Deserialize)]
    struct TimeEntry {
        time_entry: TimeEntryFields,
    }

    impl TimeEntry {
        fn new(hour: u32, project: &Project, service: &Service, note: String) -> Self {
            Self {
                time_entry: TimeEntryFields {
                    minutes: hour * 60,
                    project_id: project.project.id,
                    service_id: service.service.id,
                    note,
                },
            }
        }
    }
    let time_entry = TimeEntry::new(hours, project, service, note);

    let res = client
        .post("https://simplabs.mite.yo.lk/time_entries.json")
        .json(&time_entry)
        .send()
        .unwrap();
    if res.status() == 201 {
        let json = res.json::<Value>().unwrap();
        let entry = &json["time_entry"];
        println!(
            "{}\n{} / {} / {}\t\t{}h",
            entry["date_at"].as_str().unwrap(),
            entry["customer_name"].as_str().unwrap(),
            entry["project_name"].as_str().unwrap(),
            entry["service_name"].as_str().unwrap(),
            entry["minutes"].as_u64().unwrap() / 60
        );
    } else {
        println!(
            "Something happened trying to create an entry: {}",
            res.status()
        );
    }
}
