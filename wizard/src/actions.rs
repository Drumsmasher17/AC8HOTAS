#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub key: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub axis: bool,
}
impl Action {
    pub fn default_inverted(self) -> bool {
        matches!(self.key, "Pitch" | "Throttle")
    }
}

macro_rules! action {
    ($key:ident, $label:literal, $group:literal, $axis:literal) => {
        Action {
            key: stringify!($key),
            label: $label,
            group: $group,
            axis: $axis,
        }
    };
}

pub const ACTIONS: [Action; 52] = [
    action!(Roll, "Roll", "Flight axes", true),
    action!(Pitch, "Pitch", "Flight axes", true),
    action!(Yaw, "Yaw", "Flight axes", true),
    action!(Throttle, "Throttle", "Flight axes", true),
    action!(CameraPitch, "Camera pitch", "Flight axes", true),
    action!(CameraYaw, "Camera yaw", "Flight axes", true),
    action!(Gun, "Gun", "Essential controls", false),
    action!(Missile, "Missile", "Essential controls", false),
    action!(Weapon, "Switch weapon", "Essential controls", false),
    action!(
        SelectMissiles,
        "Select missiles",
        "Essential controls",
        false
    ),
    action!(
        SelectSpecialWeapon1,
        "Special weapon 1",
        "Essential controls",
        false
    ),
    action!(
        SelectSpecialWeapon2,
        "Special weapon 2",
        "Essential controls",
        false
    ),
    action!(Target, "Target", "Essential controls", false),
    action!(Radar, "Radar", "Essential controls", false),
    action!(Flare, "Flare", "Essential controls", false),
    action!(View, "View", "Essential controls", false),
    action!(Gear, "Gear", "Essential controls", false),
    action!(HatUp, "Hat up", "Menus & hats", false),
    action!(HatRight, "Hat right", "Menus & hats", false),
    action!(HatDown, "Hat down", "Menus & hats", false),
    action!(HatLeft, "Hat left", "Menus & hats", false),
    action!(DPadUp, "D-pad up", "Menus & hats", false),
    action!(DPadRight, "D-pad right", "Menus & hats", false),
    action!(DPadDown, "D-pad down", "Menus & hats", false),
    action!(DPadLeft, "D-pad left", "Menus & hats", false),
    action!(FaceButtonTop, "Menu · top", "Menus & hats", false),
    action!(FaceButtonRight, "Menu · right", "Menus & hats", false),
    action!(FaceButtonBottom, "Menu · bottom", "Menus & hats", false),
    action!(FaceButtonLeft, "Menu · left", "Menus & hats", false),
    action!(Pause, "Pause", "Menus & hats", false),
    action!(SpecialRight, "Hold to skip", "Menus & hats", false),
    action!(AccelDecel, "High-G turn", "Other controls", false),
    action!(AutoPilot, "Autopilot", "Other controls", false),
    action!(Platform, "Platform", "Other controls", false),
    action!(LeftShoulder, "Left shoulder", "Other controls", false),
    action!(RightShoulder, "Right shoulder", "Other controls", false),
    action!(LeftTrigger, "Left trigger", "Other controls", false),
    action!(RightTrigger, "Right trigger", "Other controls", false),
    action!(LeftThumbstick, "Left thumbstick", "Other controls", false),
    action!(RightThumbstick, "Right thumbstick", "Other controls", false),
    action!(SpecialLeft, "Special left", "Other controls", false),
    action!(TouchPad, "Touch pad", "Other controls", false),
    action!(ExtraButton_01, "Extra button 01", "Other controls", false),
    action!(ExtraButton_02, "Extra button 02", "Other controls", false),
    action!(ExtraButton_03, "Extra button 03", "Other controls", false),
    action!(ExtraButton_04, "Extra button 04", "Other controls", false),
    action!(ExtraButton_05, "Extra button 05", "Other controls", false),
    action!(ExtraButton_06, "Extra button 06", "Other controls", false),
    action!(ExtraButton_07, "Extra button 07", "Other controls", false),
    action!(ExtraButton_08, "Extra button 08", "Other controls", false),
    action!(
        OnlineLobbyAvatarDash,
        "Lobby avatar dash",
        "Other controls",
        false
    ),
    action!(
        OnlineLobbyAvatarJump,
        "Lobby avatar jump",
        "Other controls",
        false
    ),
];

pub const SETUP: [usize; 15] = [0, 1, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

pub fn instruction(index: usize, pedals: bool) -> &'static str {
    match ACTIONS[index].key {
        "Roll" => "Move the stick left and right.",
        "Pitch" => "Move the stick forward and back.",
        "Yaw" if pedals => "Move the rudder pedals.",
        "Yaw" => "Twist the stick.",
        "Throttle" => "Move the throttle through its range.",
        "CameraPitch" => "Move the camera axis up and down.",
        "CameraYaw" => "Move the camera axis left and right.",
        _ => "Press a button or hat direction.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn registry_matches_mod_template_and_native_actions() {
        let template = include_str!("../../mod/bindings.lua");
        let template_keys: BTreeSet<_> = template
            .lines()
            .filter_map(|line| {
                let (key, value) = line.trim().split_once('=')?;
                (value.trim() == "nil,").then_some(key.trim())
            })
            .collect();
        let keys: BTreeSet<_> = ACTIONS.iter().map(|a| a.key).collect();
        assert_eq!(keys.len(), ACTIONS.len());
        assert_eq!(keys, template_keys);
        let native = include_str!("../../mod/Scripts/actions.lua");
        for action in &ACTIONS {
            assert!(
                native.contains(&format!("{} =", action.key)) || action.key.starts_with("Select")
            );
        }
        assert_eq!(ACTIONS.iter().filter(|a| a.axis).count(), 6);
    }

    #[test]
    fn setup_is_the_approved_order() {
        assert_eq!(
            SETUP.map(|i| ACTIONS[i].key),
            [
                "Roll",
                "Pitch",
                "Yaw",
                "Throttle",
                "Gun",
                "Missile",
                "Weapon",
                "SelectMissiles",
                "SelectSpecialWeapon1",
                "SelectSpecialWeapon2",
                "Target",
                "Radar",
                "Flare",
                "View",
                "Gear"
            ]
        );
    }
}
