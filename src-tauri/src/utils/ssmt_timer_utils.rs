use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use std::time::Instant;

static TIMERS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
static TIMER_TOTALS: OnceLock<Mutex<HashMap<String, Duration>>> = OnceLock::new();

fn get_timers() -> &'static Mutex<HashMap<String, Instant>> {
    TIMERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_timer_totals() -> &'static Mutex<HashMap<String, Duration>> {
    TIMER_TOTALS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct SSMTTimerUtils;

impl SSMTTimerUtils {
    pub fn start(name: &str) {
        let mut map = get_timers().lock().unwrap();
        map.insert(name.to_string(), Instant::now());
    }

    pub fn end(name: &str) {
        let mut map = get_timers().lock().unwrap();
        if let Some(start_instant) = map.remove(name) {
            let elapsed = start_instant.elapsed();
            {
                let mut totals = get_timer_totals().lock().unwrap();
                let entry = totals.entry(name.to_string()).or_insert(Duration::ZERO);
                *entry += elapsed;
            }
            let secs = elapsed.as_secs() as f64 + f64::from(elapsed.subsec_micros()) / 1_000_000f64;
            println!("Timer [{}] elapsed: {:.6} s", name, secs);
        } else {
            println!("Timer [{}] was not started", name);
        }
    }

    pub fn show_all() {
        let totals = get_timer_totals().lock().unwrap();

        if totals.is_empty() {
            println!("Timer summary: no completed timers.");
            return;
        }

        let mut items: Vec<(String, Duration)> = totals
            .iter()
            .map(|(name, duration)| (name.clone(), *duration))
            .collect();

        items.sort_by(|a, b| b.1.cmp(&a.1));
        println!("------------------------------------------------");
        println!("Timer summary (accumulated):");
        for (name, duration) in items {
            let secs = duration.as_secs_f64();
            println!("Timer [{}] total: {:.6} s", name, secs);
        }
        println!("------------------------------------------------");
    }
}
