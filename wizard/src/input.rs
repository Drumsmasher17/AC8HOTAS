//! Controller metadata and capture decisions, independent of DirectInput/Slint.
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub token: String,
    pub axis: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub id: String,
    pub product: u32,
    pub name: String,
    pub inputs: Vec<Input>,
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub device: String,
    pub token: String,
    pub axis: bool,
    pub value: f32,
}
#[derive(Clone, Debug)]
pub struct Captured {
    pub request: u64,
    pub device: String,
    pub token: String,
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub devices: Vec<Device>,
    pub samples: Vec<Sample>,
    pub captured: Option<Captured>,
    pub error: Option<String>,
    pub catalog_ok: bool,
    pub timed_out: Option<u64>,
    pub ambiguous: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Request {
    pub id: u64,
    pub axis: bool,
    pub linked_throttle: bool,
}
enum Command {
    Active(bool),
    Capture(Option<Request>),
    Stop,
}
pub struct Worker {
    tx: mpsc::Sender<Command>,
    pub latest: Arc<Mutex<Snapshot>>,
    join: Option<thread::JoinHandle<()>>,
}
impl Worker {
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();
        let latest = Arc::new(Mutex::new(Snapshot::default()));
        let output = latest.clone();
        let join = thread::spawn(move || run(rx, output));
        Self {
            tx,
            latest,
            join: Some(join),
        }
    }
    pub fn active(&self, value: bool) {
        let _ = self.tx.send(Command::Active(value));
    }
    pub fn capture(&self, r: Option<Request>) {
        let _ = self.tx.send(Command::Capture(r));
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn run(rx: mpsc::Receiver<Command>, latest: Arc<Mutex<Snapshot>>) {
    let mut context = match crate::input_windows::Context::new() {
        Ok(c) => c,
        Err(e) => {
            latest.lock().unwrap().error = Some(e);
            return;
        }
    };
    let mut active = false;
    let mut request = None;
    let mut detector = None;
    let mut inventory_at = Instant::now() - Duration::from_secs(3);
    loop {
        while let Ok(command) = rx.try_recv() {
            match command {
                Command::Stop => return,
                Command::Active(value) => {
                    if active != value {
                        detector = request.map(|r: Request| Detector::new(r, Instant::now()));
                    }
                    active = value;
                }
                Command::Capture(value) => {
                    request = value;
                    detector = value.map(|r| Detector::new(r, Instant::now()));
                    let mut s = latest.lock().unwrap();
                    s.captured = None;
                    s.timed_out = None;
                    s.ambiguous = false;
                }
            }
        }
        let now = Instant::now();
        if now.duration_since(inventory_at) >= Duration::from_secs(2) {
            inventory_at = now;
            let result = context.refresh();
            let mut s = latest.lock().unwrap();
            match result {
                Ok(devices) => {
                    if devices != s.devices {
                        detector = request.map(|r| Detector::new(r, now));
                    }
                    s.devices = devices;
                    s.catalog_ok = true;
                    s.error = None;
                }
                Err(e) => {
                    s.catalog_ok = false;
                    s.error = Some(e);
                    s.samples.clear();
                    detector = request.map(|r| Detector::new(r, now));
                }
            }
        }
        if active {
            let samples = context.poll();
            let mut result = None;
            let mut timeout = None;
            let mut ambiguous = false;
            if let Some(d) = detector.as_mut() {
                result = d.feed(&samples, now);
                ambiguous = d.ambiguous;
                if now.duration_since(d.started) > Duration::from_secs(12) {
                    timeout = Some(d.request.id);
                }
            }
            let mut s = latest.lock().unwrap();
            s.samples = samples;
            s.ambiguous = ambiguous;
            if let Some(capture) = result {
                s.captured = Some(capture);
                detector = None;
                request = None;
            }
            if timeout.is_some() {
                s.timed_out = timeout;
                detector = None;
                request = None;
            }
        }
        thread::sleep(if active {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(100)
        });
    }
}

pub struct Detector {
    request: Request,
    pub started: Instant,
    baseline: BTreeMap<(String, String), f32>,
    previous: BTreeMap<(String, String), bool>,
    candidate: Option<((String, String), Instant)>,
    pub ambiguous: bool,
}
impl Detector {
    pub fn new(request: Request, started: Instant) -> Self {
        Self {
            request,
            started,
            baseline: BTreeMap::new(),
            previous: BTreeMap::new(),
            candidate: None,
            ambiguous: false,
        }
    }
    pub fn feed(&mut self, samples: &[Sample], now: Instant) -> Option<Captured> {
        self.ambiguous = false;
        if self.request.axis {
            let warming = now.duration_since(self.started) < Duration::from_millis(500);
            let mut candidates = Vec::new();
            for s in samples.iter().filter(|s| s.axis) {
                let key = (s.device.clone(), s.token.clone());
                let base = self.baseline.entry(key.clone()).or_insert(s.value);
                if warming {
                    *base = s.value;
                    continue;
                }
                let delta = (s.value - *base).abs();
                if delta >= 0.10 {
                    candidates.push((key, delta));
                }
            }
            candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
            let linked_pair = self.request.linked_throttle
                && candidates.len() == 2
                && candidates[0].0.0 == candidates[1].0.0
                && (candidates[0].1 - candidates[1].1).abs() <= 0.04;
            // Linked levers expose two equally strong axes. Select consistently
            // by token, rather than letting small sensor differences swap them.
            if linked_pair {
                candidates.sort_by(|a, b| a.0.cmp(&b.0));
            }
            let winner = candidates.first().filter(|first| {
                linked_pair
                    || candidates
                        .get(1)
                        .is_none_or(|second| first.1 > second.1 * 1.5)
            });
            if let Some((key, _)) = winner {
                if self.candidate.as_ref().is_none_or(|(k, _)| k != key) {
                    self.candidate = Some((key.clone(), now));
                }
                if self.candidate.as_ref().is_some_and(|(_, since)| {
                    now.duration_since(*since) >= Duration::from_millis(150)
                }) {
                    return Some(Captured {
                        request: self.request.id,
                        device: key.0.clone(),
                        token: key.1.clone(),
                    });
                }
            } else {
                self.ambiguous = !candidates.is_empty();
                self.candidate = None;
            }
        } else {
            let mut edges = Vec::new();
            for s in samples.iter().filter(|s| !s.axis) {
                let key = (s.device.clone(), s.token.clone());
                let down = s.value > 0.5;
                let previous = self.previous.insert(key.clone(), down);
                if previous == Some(false) && down {
                    edges.push(key);
                }
            }
            let hat_diagonal = samples
                .iter()
                .filter(|s| !s.axis && s.token.starts_with("POV1_") && s.value > 0.5)
                .count()
                > 1;
            if edges.len() == 1 && !hat_diagonal {
                let key = edges.pop().unwrap();
                return Some(Captured {
                    request: self.request.id,
                    device: key.0,
                    token: key.1,
                });
            }
            self.ambiguous = edges.len() > 1 || hat_diagonal;
        }
        None
    }
}
pub fn preview(samples: &[Sample], device: &str, token: &str) -> Option<f32> {
    samples
        .iter()
        .find(|s| s.device == device && s.token == token)
        .map(|s| s.value)
}
pub fn validate(
    bindings: &[Option<crate::model::Binding>],
    catalog: &Snapshot,
) -> Result<Vec<String>, String> {
    if !catalog.catalog_ok {
        return Err(catalog
            .error
            .clone()
            .unwrap_or("Controller discovery is not ready".into()));
    }
    let mut warnings = Vec::new();
    let mut used: BTreeMap<(&str, &str), &str> = BTreeMap::new();
    for (i, b) in bindings.iter().enumerate() {
        let Some(b) = b else { continue };
        crate::config::validate_binding(i, b)?;
        let d = catalog
            .devices
            .iter()
            .find(|d| d.id == b.device)
            .ok_or_else(|| {
                format!(
                    "Reconnect controller for {}",
                    crate::actions::ACTIONS[i].label
                )
            })?;
        if !d.inputs.iter().any(|input| input.token == b.input) {
            return Err(format!("{} does not expose {}", d.name, b.input));
        }
        if !crate::actions::ACTIONS[i].key.starts_with("Select")
            && catalog
                .devices
                .iter()
                .filter(|other| other.product == d.product)
                .count()
                > 1
        {
            return Err(format!(
                "The game cannot distinguish identical controllers: {}",
                d.name
            ));
        }
        if let Some(prior) = used.insert((&b.device, &b.input), crate::actions::ACTIONS[i].label) {
            warnings.push(format!(
                "{} shares {} with {prior}",
                crate::actions::ACTIONS[i].label,
                b.input
            ));
        }
    }
    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(token: &str, value: f32) -> Sample {
        Sample {
            device: "d1".into(),
            token: token.into(),
            axis: crate::config::axis_token(token),
            value,
        }
    }
    #[test]
    fn axis_requires_baseline_dominance_and_sustain() {
        let t = Instant::now();
        let mut d = Detector::new(
            Request {
                id: 1,
                axis: true,
                linked_throttle: false,
            },
            t,
        );
        d.feed(&[sample("X", 0.5), sample("Y", 0.5)], t);
        assert!(
            d.feed(&[sample("X", 0.51)], t + Duration::from_millis(510))
                .is_none()
        );
        assert!(
            d.feed(
                &[sample("X", 0.8), sample("Y", 0.8)],
                t + Duration::from_millis(520)
            )
            .is_none()
        );
        assert!(d.ambiguous);
        assert!(
            d.feed(
                &[sample("X", 0.8), sample("Y", 0.5)],
                t + Duration::from_millis(600)
            )
            .is_none()
        );
        assert_eq!(
            d.feed(&[sample("X", 0.8)], t + Duration::from_millis(760))
                .unwrap()
                .token,
            "X"
        );
    }
    #[test]
    fn held_buttons_and_hat_diagonals_are_not_captured() {
        let t = Instant::now();
        let mut d = Detector::new(
            Request {
                id: 2,
                axis: false,
                linked_throttle: false,
            },
            t,
        );
        assert!(d.feed(&[sample("Button1", 1.)], t).is_none());
        assert!(d.feed(&[sample("Button1", 0.)], t).is_none());
        assert_eq!(
            d.feed(&[sample("Button1", 1.)], t).unwrap().token,
            "Button1"
        );
        let mut h = Detector::new(
            Request {
                id: 3,
                axis: false,
                linked_throttle: false,
            },
            t,
        );
        h.feed(&[sample("POV1_Up", 0.), sample("POV1_Right", 0.)], t);
        assert!(
            h.feed(&[sample("POV1_Up", 1.), sample("POV1_Right", 1.)], t)
                .is_none()
        );
        h.feed(&[sample("POV1_Up", 0.), sample("POV1_Right", 0.)], t);
        assert_eq!(
            h.feed(&[sample("POV1_Up", 1.), sample("POV1_Right", 0.)], t)
                .unwrap()
                .token,
            "POV1_Up"
        );
    }
    #[test]
    fn a_new_baseline_does_not_capture_the_previous_motion() {
        let t = Instant::now();
        let mut d = Detector::new(
            Request {
                id: 5,
                axis: true,
                linked_throttle: false,
            },
            t,
        );
        d.feed(&[sample("Slider2", 0.9)], t);
        assert!(
            d.feed(&[sample("Slider2", 0.9)], t + Duration::from_millis(700))
                .is_none()
        );
    }
    #[test]
    fn linked_throttles_capture_without_separating_the_levers() {
        let t = Instant::now();
        let mut d = Detector::new(
            Request {
                id: 6,
                axis: true,
                linked_throttle: true,
            },
            t,
        );
        d.feed(&[sample("X", 0.), sample("Y", 0.)], t);
        assert!(
            d.feed(
                &[sample("X", 0.40), sample("Y", 0.41)],
                t + Duration::from_millis(600)
            )
            .is_none()
        );
        let captured = d.feed(
            &[sample("X", 0.70), sample("Y", 0.71)],
            t + Duration::from_millis(800),
        );
        assert_eq!(
            captured
                .expect("Linked throttle motion must capture without unlocking the levers")
                .token,
            "X"
        );
    }
    #[test]
    fn throttle_does_not_merge_different_controllers_or_unrelated_motion() {
        let t = Instant::now();
        for different_device in [false, true] {
            let mut d = Detector::new(
                Request {
                    id: 7,
                    axis: true,
                    linked_throttle: true,
                },
                t,
            );
            let mut second = sample("Y", 0.);
            if different_device {
                second.device = "d2".into();
            }
            d.feed(&[sample("X", 0.), second.clone()], t);
            second.value = if different_device { 0.7 } else { 0.5 };
            for ms in [600, 800] {
                assert!(
                    d.feed(
                        &[sample("X", 0.7), second.clone()],
                        t + Duration::from_millis(ms)
                    )
                    .is_none()
                );
                assert!(d.ambiguous);
            }
        }
    }
    #[test]
    fn validation_matches_duplicate_product_and_direct_selection_rules() {
        let id = "{11111111-2222-3333-4444-555555555555}";
        let other = "{22222222-2222-3333-4444-555555555555}";
        let d = Device {
            id: id.into(),
            product: 1,
            name: "Stick".into(),
            inputs: vec![
                Input {
                    token: "X".into(),
                    axis: true,
                },
                Input {
                    token: "Button1".into(),
                    axis: false,
                },
            ],
        };
        let mut catalog = Snapshot {
            devices: vec![d.clone()],
            catalog_ok: true,
            ..Default::default()
        };
        let mut bindings = vec![None; 52];
        bindings[0] = Some(crate::model::Binding {
            device: id.into(),
            input: "X".into(),
            invert: false,
            zero_to_one: false,
        });
        assert!(validate(&bindings, &catalog).is_ok());
        let mut same = d;
        same.id = other.into();
        catalog.devices.push(same);
        assert!(validate(&bindings, &catalog).is_err());
        bindings[0] = None;
        bindings[9] = Some(crate::model::Binding {
            device: id.into(),
            input: "Button1".into(),
            invert: false,
            zero_to_one: false,
        });
        assert!(validate(&bindings, &catalog).is_ok());
        catalog.devices.clear();
        assert!(validate(&bindings, &catalog).is_err());
    }
    #[test]
    fn shared_inputs_warn_and_missing_tokens_fail() {
        let id = "{11111111-2222-3333-4444-555555555555}";
        let catalog = Snapshot {
            devices: vec![Device {
                id: id.into(),
                product: 1,
                name: "Stick".into(),
                inputs: vec![Input {
                    token: "Button1".into(),
                    axis: false,
                }],
            }],
            catalog_ok: true,
            ..Default::default()
        };
        let mut b = vec![None; 52];
        b[6] = Some(crate::model::Binding {
            device: id.into(),
            input: "Button1".into(),
            invert: false,
            zero_to_one: false,
        });
        b[7] = b[6].clone();
        assert_eq!(validate(&b, &catalog).unwrap().len(), 1);
        b[7].as_mut().unwrap().input = "Button2".into();
        assert!(validate(&b, &catalog).is_err());
    }
}
