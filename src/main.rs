use serde_json::Value;
use std::process::Command;

// store unit details in a struct
#[derive(Debug, Clone)]
struct Service {
    /// name of the unit
    unit: String,

    /// exposure value of the unit
    exposure: f64,

    /// exposure predicate of the unit
    predicate: String,

    /// happiness score of the unit, represented
    /// by emojis: 😀, 🙂, 😐, 🙁, 😨
    happy: String,
}

fn run_systemd_analyze() -> Vec<Service> {
    let output = Command::new("systemd-analyze")
        .args(&["security", "--json=short", "--no-pager"])
        .output()
        .expect("failed to execute process");

    // check if the command was successful
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        panic!("systemd-analyze failed: {}", err);
    }

    let json_output: Value = serde_json::from_slice(&output.stdout).expect("failed to parse json");

    let mut services: Vec<Service> = Vec::new();

    if let Some(entries) = json_output.as_array() {
        for entry in entries {
            let exposure = match entry.get("exposure") {
                Some(Value::Number(num)) => num.as_f64(),
                Some(Value::String(s)) => s.parse::<f64>().ok(),
                _ => None,
            };

            if let (Some(unit), Some(exposure), Some(predicate), Some(happy)) = (
                entry.get("unit").and_then(|v| v.as_str()),
                exposure,
                entry.get("predicate").and_then(|v| v.as_str()),
                entry.get("happy").and_then(|v| v.as_str()),
            ) {
                services.push(Service {
                    unit: unit.to_string(),
                    exposure,
                    predicate: predicate.to_string(),
                    happy: happy.to_string(),
                });
            } else {
                println!("Warning: could not parse entry: {:?}", entry);
            }
        }
    }
    services
}

fn calculate_exposure_average(services: &[Service]) -> f64 {
    if services.is_empty() {
        return f64::NAN;
    }
    let total_exposure: f64 = services.iter().map(|s| s.exposure).sum();
    total_exposure / services.len() as f64
}

fn calculate_happiness_average(services: &[Service]) -> f64 {
    let happiness_map = vec![
        ("😀", 5.0),
        ("🙂", 4.0),
        ("😐", 3.0),
        ("🙁", 2.0),
        ("😨", 1.0),
    ];
    let mut total_happiness = 0.0;
    let mut count = 0;
    for service in services {
        if let Some(&score) =
            happiness_map
                .iter()
                .find_map(|(h, s)| if service.happy == *h { Some(s) } else { None })
        {
            total_happiness += score;
            count += 1;
        } else {
            println!("Warning: unmatched happy value '{}'", service.happy);
        }
    }
    if count == 0 {
        f64::NAN
    } else {
        total_happiness / count as f64
    }
}

fn top_n_services(services: &[Service], predicate: &str, n: usize) -> Vec<Service> {
    let mut filtered_services: Vec<Service> = services
        .iter()
        .filter(|s| s.predicate == predicate)
        .cloned()
        .collect();

    filtered_services.sort_by(|a, b| {
        b.exposure
            .partial_cmp(&a.exposure)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    filtered_services.into_iter().take(n).collect()
}

fn main() {
    let services = run_systemd_analyze();

    let exposure_avg = calculate_exposure_average(&services);
    println!("Average Exposure: {:.2}", exposure_avg);

    let happiness_avg = calculate_happiness_average(&services);
    println!("Average Happiness: {:.2}", happiness_avg);

    // TODO: those should be arguments
    let predicate = "MEDIUM";
    let top_n = 5;
    let top_services = top_n_services(&services, predicate, top_n);

    println!("\nTop {} services with predicate '{}':", top_n, predicate);
    for service in top_services {
        println!("{:?} (Exposure: {:.2})", service.unit, service.exposure);
    }
}