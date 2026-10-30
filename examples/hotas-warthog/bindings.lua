-- Bindings for Thrustmaster HOTAS Warthog Throttle and Joystick
-- Community example. See README.md in this folder for setup and test notes.

-- REQUIRED: replace both placeholders with your own device GUIDs, copied from
-- the generated DETECTED DEVICES section of your installed bindings.lua:
--   "Joystick - HOTAS Warthog | 0x0402044F" -> stick
--   "Throttle - HOTAS Warthog | 0x0404044F" -> throttle
-- The config is rejected ("Device not connected") until both are replaced.
local stick = "{PASTE-WARTHOG-JOYSTICK-GUID-HERE}"
local throttle = "{PASTE-WARTHOG-THROTTLE-GUID-HERE}"

return {
    version = 1,

    bindings = {
        -- COMBAT CONTROLS
        -- Trigger first stage
        Gun = { device = stick, input = "Button1" },

        -- Red thumb button
        Missile = { device = stick, input = "Button2" },

        -- Index-finger side button
        Weapon = { device = stick, input = "Button5" },

        -- TMS hat up / down
        Target = { device = stick, input = "Button7" },
        Radar = { device = stick, input = "Button9" },

        -- CMS hat down
        Flare = { device = stick, input = "Button17" },

        -- Pinkie pushbutton
        View = { device = stick, input = "Button3" },

        -- DMS hat right
        Pause = { device = stick, input = "Button12" },

        Gear = nil,
        Platform = nil,

        -- STICK POV HAT
        -- Effects depend on the game's current context.
        HatUp = nil,
        HatRight = nil,
        HatDown = nil,
        HatLeft = nil,

        DPadUp = { device = stick, input = "POV1_Up" },
        DPadRight = { device = stick, input = "POV1_Right" },
        DPadDown = { device = stick, input = "POV1_Down" },
        DPadLeft = { device = stick, input = "POV1_Left" },

        -- MENU FACE BUTTONS: STICK TMS HAT
        -- Up = Y, right = B, down = A, left = X
        FaceButtonTop = { device = stick, input = "Button7" },
        FaceButtonRight = { device = stick, input = "Button8" },
        FaceButtonBottom = { device = stick, input = "Button9" },
        FaceButtonLeft = { device = stick, input = "Button10" },

        -- FLIGHT AXES
        Pitch = {
            device = stick,
            input = "Y",
            invert = true,
            mode = "minus_one_to_one"
        },

        Roll = {
            device = stick,
            input = "X",
            invert = false,
            mode = "minus_one_to_one"
        },

        -- Right throttle lever
        Throttle = {
            device = throttle,
            input = "Z",
            invert = true,
            mode = "zero_to_one"
        },

        -- Throttle ministick left / right
        Yaw = {
            device = throttle,
            input = "X",
            invert = false,
            mode = "minus_one_to_one"
        },

        CameraPitch = nil,
        CameraYaw = nil,

        -- CMS hat up: direct autopilot action
        AutoPilot = { device = stick, input = "Button15" },

        -- CMS hat left / right: LB / RB
        LeftShoulder = { device = stick, input = "Button18" },
        RightShoulder = { device = stick, input = "Button16" },

        -- DMS hat right, shared with Pause. Suggested for hold-to-skip
        -- cutscene prompts; not yet confirmed to skip.
        SpecialRight = { device = stick, input = "Button12" },

        -- Pinkie paddle lever: high-G turn
        AccelDecel = { device = stick, input = "Button4" },

        -- REMAINING ACTIONS UNBOUND
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
