//! Opt-in developer validation, excluded from distributed builds.
use std::path::Path;
pub fn run(args: &[String], output: &Path) {
    let result = (|| -> Result<serde_json::Value, String> {
        if args.iter().any(|a| a == "--emit-fixture") {
            return parity_fixture();
        }
        let mut context = crate::input_windows::Context::new()?;
        let devices = context.refresh()?;
        let mut samples = Vec::new();
        for _ in 0..5 {
            samples = context.poll();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let mut result = serde_json::json!({"ok":true,"devices":devices.iter().map(|d|serde_json::json!({"id":d.id,"name":d.name,"product":d.product,"axes":d.inputs.iter().filter(|i|i.axis).map(|i|&i.token).collect::<Vec<_>>(),"digital_inputs":d.inputs.iter().filter(|i|!i.axis).count(),"samples_read":samples.iter().filter(|s|s.device==d.id).count()})).collect::<Vec<_>>()});
        if let Some(root) = args.iter().find_map(|a| a.strip_prefix("--mod-dir=")) {
            let install = crate::installation::Installation::load(Path::new(root))?;
            let rendered = install.document.render(&install.document.bindings)?;
            if rendered != install.document.text {
                return Err("Unedited config roundtrip differs".into());
            }
            let catalog = crate::input::Snapshot {
                devices,
                catalog_ok: true,
                ..Default::default()
            };
            let warnings = crate::input::validate(&install.document.bindings, &catalog)?;
            result["configuration"] = serde_json::json!({"roundtrip_exact":true,"assigned":install.document.bindings.iter().filter(|b|b.is_some()).count(),"warnings":warnings,"game_closed":crate::platform::game_closed(&install.game).is_ok()});
            if args.iter().any(|a| a == "--exercise-storage") {
                let saved = crate::storage::save(
                    &install.root,
                    &install.document,
                    &install.document.bindings,
                    || crate::platform::game_closed(&install.game),
                )?;
                result["storage"] = serde_json::json!({"verified":saved.document.text==install.document.text,"backup":saved.backup});
            }
        }
        Ok(result)
    })();
    let (ok, report) = match result {
        Ok(v) => (true, v),
        Err(e) => (false, serde_json::json!({"ok":false,"error":e})),
    };
    if let Some(parent) = output.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()) {
        eprintln!("Cannot write diagnostics: {e}");
        std::process::exit(2)
    }
    if !ok {
        std::process::exit(1)
    }
}
fn parity_fixture() -> Result<serde_json::Value, String> {
    use crate::{actions::ACTIONS, config::Document, model::Binding};
    let id = "{11111111-2222-3333-4444-555555555555}";
    let doc = Document::parse("return {version=1, bindings={}}".into())?;
    let mut bindings = vec![None; ACTIONS.len()];
    let axes = ["X", "Y", "Z", "Rx", "Ry", "Rz", "Slider1", "Slider2"];
    let mut axis = 0;
    let mut expected = Vec::new();
    for (i, action) in ACTIONS.iter().enumerate() {
        let token = if action.axis {
            let token = axes[axis % axes.len()].to_owned();
            axis += 1;
            token
        } else if i % 3 == 0 {
            "POV1_Up".to_owned()
        } else {
            format!("Button{}", i + 1)
        };
        let invert = action.axis && i % 2 == 0;
        let zero = action.axis && i % 2 == 1;
        bindings[i] = Some(Binding {
            device: id.into(),
            input: token.clone(),
            invert,
            zero_to_one: zero,
        });
        expected.push(serde_json::json!({"key":action.key,"axis":action.axis,"input":token,"invert":invert,"option":(if zero {4} else {0}) + (if invert {1} else {0})}));
    }
    let config = doc.render(&bindings)?;
    Ok(serde_json::json!({"config":config,"device":id,"expected":expected}))
}
