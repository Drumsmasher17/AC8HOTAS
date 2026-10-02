-- AC8HOTAS: edit your bindings, save this file, then press F5 in the game.

-- nil means unbound. Copy a device GUID and input token from the detected list.

-- Axis example: Roll = { device="{detected GUID}", input="X", invert=false },
-- Button example: Gun = { device="{detected GUID}", input="Button1" },
-- Hat example: HatUp = { device="{detected GUID}", input="POV1_Up" },
-- Button numbers start at 1. Hats use POV1_Up/Down/Left/Right; only POV1 is supported.

-- Pitch/Roll/Yaw/Throttle/CameraPitch/CameraYaw require axes; other actions require buttons/hats.

-- Axis invert=true reverses direction; buttons/hats do not use invert or mode.
-- Axis mode="minus_one_to_one": -1 / 0 / +1; mode="zero_to_one": 0 / 0.5 / 1.
-- Mode defaults to minus_one_to_one except Throttle, which defaults to zero_to_one.

-- At startup and on F5, AC8HOTAS clears Windows flight-stick mappings in memory,
-- then assigns clean compatible game profile slots from this file. Nothing is saved.
-- A compatible slot must still exist for each action you bind; the mod can reuse
-- another built-in flight-stick slot when a device's own template lacks an action.

return {
    version = 1,
    bindings = {
        -- Button Controls
        -- Replace nil with a button or hat binding from the examples above.
        Gun = nil,
        Missile = nil,
        Weapon = nil,
        Target = nil,
        Radar = nil,
        Flare = nil,
        View = nil,
        Gear = nil,
        
        HatUp = nil,
        HatRight = nil,
        HatDown = nil,
        HatLeft = nil,
        
        DPadUp = nil,
        DPadRight = nil,
        DPadDown = nil,
        DPadLeft = nil,

        -- Menu navigation (if you want to use your HOTAS in menus)
        FaceButtonTop = nil,
        FaceButtonRight = nil,
        FaceButtonBottom = nil,
        FaceButtonLeft = nil,

        -- Pause and SpecialRight can share a button. SpecialRight is used for hold-to-skip prompts.
        Pause = nil,
        SpecialRight = nil,

        -- Optional High-G turn button; zero throttle can also trigger this.
        AccelDecel = nil,

        -- Axis Controls
        -- nil means unbound. Replace nil with a binding from the device list.
        -- Example: Yaw = { device = "{detected GUID}", input = "X", invert = false },
        Pitch = nil,
        Roll = nil,
        Throttle = nil,
        CameraPitch = nil,
        CameraYaw = nil,
        Yaw = nil,

        -- Optional & Untested Controls
        AutoPilot = nil,
        Platform = nil,
        LeftShoulder = nil,
        RightShoulder = nil,
        LeftTrigger = nil,
        RightTrigger = nil,
        LeftThumbstick = nil,
        RightThumbstick = nil,
        SpecialLeft = nil,
        TouchPad = nil,
        ExtraButton_01 = nil,
        ExtraButton_02 = nil,
        ExtraButton_03 = nil,
        ExtraButton_04 = nil,
        ExtraButton_05 = nil,
        ExtraButton_06 = nil,
        ExtraButton_07 = nil,
        ExtraButton_08 = nil,
        OnlineLobbyAvatarDash = nil,
        OnlineLobbyAvatarJump = nil,
    },
}
