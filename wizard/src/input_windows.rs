//! All DirectInput objects and the hidden cooperative window stay on one worker.
#![allow(unsafe_code)]
use crate::input::{Device, Input, Sample};
use std::{collections::BTreeMap, ffi::c_void, mem::size_of};
use windows::{
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::{HINSTANCE, HWND},
        System::{
            Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
            LibraryLoader::GetModuleHandleW,
        },
        UI::WindowsAndMessaging::*,
    },
    core::{BOOL, GUID, Interface, w},
};
#[link(name = "dinput8", kind = "static")]
unsafe extern "C" {
    static c_dfDIJoystick2: DIDATAFORMAT;
}
#[link(name = "dxguid", kind = "static")]
unsafe extern "C" {}
struct Reader {
    device: IDirectInputDevice8W,
    meta: Device,
    ranges: BTreeMap<String, (i32, i32)>,
}
impl Drop for Reader {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.Unacquire();
        }
    }
}
struct Apartment(bool);
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
pub struct Context {
    readers: BTreeMap<String, Reader>,
    input: IDirectInput8W,
    window: HWND,
    _com: Apartment,
}
impl Context {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let com = Apartment(CoInitializeEx(None, COINIT_MULTITHREADED).is_ok());
            let instance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
            let window = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("AC8HOTAS input"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(HINSTANCE(instance.0)),
                None,
            )
            .map_err(|e| e.to_string())?;
            let mut raw = std::ptr::null_mut();
            if let Err(e) = DirectInput8Create(
                HINSTANCE(instance.0),
                0x800,
                &IDirectInput8W::IID,
                &mut raw,
                None,
            ) {
                let _ = DestroyWindow(window);
                return Err(e.to_string());
            }
            Ok(Self {
                readers: BTreeMap::new(),
                input: IDirectInput8W::from_raw(raw),
                window,
                _com: com,
            })
        }
    }
    pub fn refresh(&mut self) -> Result<Vec<Device>, String> {
        // DirectInput invokes the callback synchronously. It only copies POD data;
        // no panic or reference can escape across the C ABI.
        unsafe extern "system" fn callback(
            info: *mut DIDEVICEINSTANCEW,
            context: *mut c_void,
        ) -> BOOL {
            unsafe {
                if !info.is_null() && !context.is_null() {
                    (*(context as *mut Vec<DIDEVICEINSTANCEW>)).push(*info);
                }
            }
            BOOL(1)
        }
        let mut instances: Vec<DIDEVICEINSTANCEW> = Vec::new();
        unsafe {
            self.input
                .EnumDevices(
                    DI8DEVCLASS_GAMECTRL,
                    Some(callback),
                    &mut instances as *mut _ as *mut c_void,
                    DIEDFL_ATTACHEDONLY,
                )
                .map_err(|e| e.to_string())?;
        }
        let ids: Vec<_> = instances.iter().map(|i| guid(i.guidInstance)).collect();
        self.readers.retain(|id, _| ids.contains(id));
        let mut failures = Vec::new();
        for info in instances {
            let id = guid(info.guidInstance);
            if self.readers.contains_key(&id) {
                continue;
            }
            match self.connect(&info) {
                Ok(reader) => {
                    self.readers.insert(id, reader);
                }
                Err(e) => failures.push(e),
            }
        }
        if !failures.is_empty() {
            return Err(format!(
                "Controller discovery failed: {}",
                failures.join("; ")
            ));
        }
        Ok(self.readers.values().map(|r| r.meta.clone()).collect())
    }
    fn connect(&self, info: &DIDEVICEINSTANCEW) -> Result<Reader, String> {
        unsafe {
            let mut out = None;
            self.input
                .CreateDevice(&info.guidInstance, &mut out, None)
                .map_err(|e| e.to_string())?;
            let device = out.ok_or("DirectInput returned no device")?;
            // Use the Windows SDK's exact DIJOYSTATE2 format, shared with the mod.
            let mut format = c_dfDIJoystick2;
            device
                .SetDataFormat(&mut format)
                .map_err(|e| e.to_string())?;
            device
                .SetCooperativeLevel(self.window, DISCL_NONEXCLUSIVE | DISCL_BACKGROUND)
                .map_err(|e| e.to_string())?;
            let mut inputs = Vec::new();
            let mut ranges = BTreeMap::new();
            for (offset, token) in [
                (0, "X"),
                (4, "Y"),
                (8, "Z"),
                (12, "Rx"),
                (16, "Ry"),
                (20, "Rz"),
                (24, "Slider1"),
                (28, "Slider2"),
            ] {
                let mut object = DIDEVICEOBJECTINSTANCEW {
                    dwSize: size_of::<DIDEVICEOBJECTINSTANCEW>() as u32,
                    ..Default::default()
                };
                if device
                    .GetObjectInfo(&mut object, offset, DIPH_BYOFFSET)
                    .is_ok()
                {
                    let mut range = DIPROPRANGE {
                        diph: DIPROPHEADER {
                            dwSize: size_of::<DIPROPRANGE>() as u32,
                            dwHeaderSize: size_of::<DIPROPHEADER>() as u32,
                            dwObj: offset,
                            dwHow: DIPH_BYOFFSET,
                        },
                        ..Default::default()
                    };
                    // DIPROP_RANGE is MAKEDIPROP(4), a special pointer-valued ID, not
                    // a pointer to a GUID object. DirectInput recognizes this sentinel.
                    device
                        .GetProperty(std::ptr::without_provenance::<GUID>(4), &mut range.diph)
                        .map_err(|e| format!("Cannot read {token} range: {e}"))?;
                    if range.lMax <= range.lMin {
                        return Err(format!("Invalid {token} range"));
                    }
                    ranges.insert(token.to_owned(), (range.lMin, range.lMax));
                    inputs.push(Input {
                        token: token.into(),
                        axis: true,
                    });
                }
            }
            for button in 0..128 {
                let mut object = DIDEVICEOBJECTINSTANCEW {
                    dwSize: size_of::<DIDEVICEOBJECTINSTANCEW>() as u32,
                    ..Default::default()
                };
                if device
                    .GetObjectInfo(&mut object, 48 + button, DIPH_BYOFFSET)
                    .is_ok()
                {
                    inputs.push(Input {
                        token: format!("Button{}", button + 1),
                        axis: false,
                    });
                }
            }
            let mut hat = DIDEVICEOBJECTINSTANCEW {
                dwSize: size_of::<DIDEVICEOBJECTINSTANCEW>() as u32,
                ..Default::default()
            };
            if device.GetObjectInfo(&mut hat, 32, DIPH_BYOFFSET).is_ok() {
                for direction in ["Up", "Down", "Left", "Right"] {
                    inputs.push(Input {
                        token: format!("POV1_{direction}"),
                        axis: false,
                    });
                }
            }
            let _ = device.Acquire();
            let name = String::from_utf16_lossy(
                &info.tszProductName[..info
                    .tszProductName
                    .iter()
                    .position(|c| *c == 0)
                    .unwrap_or(info.tszProductName.len())],
            );
            Ok(Reader {
                device,
                ranges,
                meta: Device {
                    id: guid(info.guidInstance),
                    product: info.guidProduct.data1,
                    name,
                    inputs,
                },
            })
        }
    }
    pub fn poll(&mut self) -> Vec<Sample> {
        let mut output = Vec::new();
        for reader in self.readers.values_mut() {
            unsafe {
                let mut state = DIJOYSTATE2::default();
                let mut result = reader.device.Poll().and_then(|_| {
                    reader.device.GetDeviceState(
                        size_of::<DIJOYSTATE2>() as u32,
                        &mut state as *mut _ as *mut c_void,
                    )
                });
                if result.is_err() {
                    let _ = reader.device.Acquire();
                    result = reader.device.Poll().and_then(|_| {
                        reader.device.GetDeviceState(
                            size_of::<DIJOYSTATE2>() as u32,
                            &mut state as *mut _ as *mut c_void,
                        )
                    });
                }
                if result.is_err() {
                    continue;
                }
                let axes = [
                    state.lX,
                    state.lY,
                    state.lZ,
                    state.lRx,
                    state.lRy,
                    state.lRz,
                    state.rglSlider[0],
                    state.rglSlider[1],
                ];
                for input in &reader.meta.inputs {
                    let value = if input.axis {
                        let n = ["X", "Y", "Z", "Rx", "Ry", "Rz", "Slider1", "Slider2"]
                            .iter()
                            .position(|s| *s == input.token)
                            .unwrap();
                        let (min, max) = reader.ranges[&input.token];
                        ((axes[n] as f64 - min as f64) / (max as f64 - min as f64)).clamp(0., 1.)
                            as f32
                    } else if let Some(button) = input.token.strip_prefix("Button") {
                        if state.rgbButtons[button.parse::<usize>().unwrap() - 1] & 0x80 != 0 {
                            1.
                        } else {
                            0.
                        }
                    } else {
                        let pov = state.rgdwPOV[0];
                        let pressed = pov & 0xffff != 0xffff
                            && match input.token.as_str() {
                                "POV1_Up" => pov >= 31500 || pov <= 4500,
                                "POV1_Down" => (13500..=22500).contains(&pov),
                                "POV1_Left" => (22500..=31500).contains(&pov),
                                _ => (4500..=13500).contains(&pov),
                            };
                        if pressed { 1. } else { 0. }
                    };
                    output.push(Sample {
                        device: reader.meta.id.clone(),
                        token: input.token.clone(),
                        axis: input.axis,
                        value,
                    });
                }
            }
        }
        output
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        self.readers.clear();
        unsafe {
            let _ = DestroyWindow(self.window);
        }
    }
}
fn guid(g: GUID) -> String {
    format!("{{{g:?}}}").to_ascii_uppercase()
}
pub fn foreground_is_ours() -> bool {
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid));
        pid == std::process::id()
    }
}
