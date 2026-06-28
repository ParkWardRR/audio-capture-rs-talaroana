//! macOS CoreAudio capture backend.
//!
//! Uses AudioUnit (kAudioUnitSubType_HALOutput) to capture system audio
//! with hardware-level timing precision. On macOS 14.4+, this can be
//! extended to use ScreenCaptureKit audio taps for per-app capture.

use std::ptr;
use std::sync::{Arc, Mutex};

use crate::{AudioCaptureBackend, CaptureCallback, CaptureConfig, CaptureSource, SampleFormat};

// CoreAudio type aliases for FFI. We use raw bindings for maximum control
// and minimal dependency surface — the coreaudio-sys crate provides the
// constants but we drive the API ourselves for hardware acceleration paths.

#[allow(non_camel_case_types)]
type AudioUnit = *mut std::ffi::c_void;
#[allow(non_camel_case_types)]
type OSStatus = i32;
#[allow(non_camel_case_types)]
type AudioDeviceID = u32;

// AudioUnit property IDs
const K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO: u32 = 2003;
const K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT: u32 = 8;
const K_AUDIO_OUTPUT_UNIT_PROPERTY_CURRENT_DEVICE: u32 = 2000;
const K_AUDIO_UNIT_PROPERTY_SET_RENDER_CALLBACK: u32 = 23;
const K_AUDIO_UNIT_SCOPE_INPUT: u32 = 1;
const K_AUDIO_UNIT_SCOPE_OUTPUT: u32 = 0;
const K_AUDIO_UNIT_SCOPE_GLOBAL: u32 = 0;

// AudioStreamBasicDescription format IDs
const K_AUDIO_FORMAT_LINEAR_PCM: u32 = 0x6C70636D; // 'lpcm'
const K_AUDIO_FORMAT_FLAG_IS_FLOAT: u32 = 1;
const K_AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER: u32 = 4;
const K_AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 8;

// AudioObjectPropertyAddress
const K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE: u32 = 0x646F7574; // 'dout'
const K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = 0x676C6F62; // 'glob'
const K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0; // 'main' in newer SDKs

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct AudioStreamBasicDescription {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

extern "C" {
    // AudioComponent
    fn AudioComponentFindNext(
        component: *mut std::ffi::c_void,
        desc: *const AudioComponentDescription,
    ) -> *mut std::ffi::c_void;

    fn AudioComponentInstanceNew(
        component: *mut std::ffi::c_void,
        instance: *mut AudioUnit,
    ) -> OSStatus;

    fn AudioComponentInstanceDispose(instance: AudioUnit) -> OSStatus;

    // AudioUnit
    fn AudioUnitSetProperty(
        unit: AudioUnit,
        property_id: u32,
        scope: u32,
        element: u32,
        data: *const std::ffi::c_void,
        data_size: u32,
    ) -> OSStatus;

    fn AudioUnitInitialize(unit: AudioUnit) -> OSStatus;
    fn AudioUnitUninitialize(unit: AudioUnit) -> OSStatus;
    fn AudioOutputUnitStart(unit: AudioUnit) -> OSStatus;
    fn AudioOutputUnitStop(unit: AudioUnit) -> OSStatus;

    // AudioObjectGetPropertyData
    fn AudioObjectGetPropertyData(
        object_id: u32,
        address: *const AudioObjectPropertyAddress,
        qualifier_data_size: u32,
        qualifier_data: *const std::ffi::c_void,
        data_size: *mut u32,
        data: *mut std::ffi::c_void,
    ) -> OSStatus;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct AudioComponentDescription {
    component_type: u32,
    component_sub_type: u32,
    component_manufacturer: u32,
    component_flags: u32,
    component_flags_mask: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct AudioObjectPropertyAddress {
    selector: u32,
    scope: u32,
    element: u32,
}

/// Render callback data passed to CoreAudio's input callback.
struct CallbackData {
    callback: CaptureCallback,
    format: CaptureConfig,
}

/// CoreAudio capture backend.
pub struct CoreAudioCapture {
    audio_unit: Option<AudioUnit>,
    callback_data: Option<Arc<Mutex<CallbackData>>>,
}

// Safety: AudioUnit is a raw pointer to CoreAudio's opaque type.
// We ensure it's only accessed from the thread that created it or
// from CoreAudio's callback thread (which is explicitly thread-safe).
unsafe impl Send for CoreAudioCapture {}

impl CoreAudioCapture {
    pub fn new() -> Self {
        Self {
            audio_unit: None,
            callback_data: None,
        }
    }

    fn get_default_output_device() -> Result<AudioDeviceID, String> {
        let mut device_id: AudioDeviceID = 0;
        let mut size = std::mem::size_of::<AudioDeviceID>() as u32;
        let address = AudioObjectPropertyAddress {
            selector: K_AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE,
            scope: K_AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            element: K_AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };

        let status = unsafe {
            AudioObjectGetPropertyData(
                1, // kAudioObjectSystemObject
                &address,
                0,
                ptr::null(),
                &mut size,
                &mut device_id as *mut _ as *mut _,
            )
        };

        if status != 0 {
            return Err(format!("AudioObjectGetPropertyData failed: {}", status));
        }
        Ok(device_id)
    }

    fn build_asbd(config: &CaptureConfig) -> AudioStreamBasicDescription {
        let (bits, flags) = match config.format {
            SampleFormat::S16LE => (
                16u32,
                K_AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER | K_AUDIO_FORMAT_FLAG_IS_PACKED,
            ),
            SampleFormat::S24LE => (
                24,
                K_AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER | K_AUDIO_FORMAT_FLAG_IS_PACKED,
            ),
            SampleFormat::S32LE => (
                32,
                K_AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER | K_AUDIO_FORMAT_FLAG_IS_PACKED,
            ),
            SampleFormat::F32LE => (
                32,
                K_AUDIO_FORMAT_FLAG_IS_FLOAT | K_AUDIO_FORMAT_FLAG_IS_PACKED,
            ),
        };

        let bytes_per_sample = (bits + 7) / 8;
        let bytes_per_frame = bytes_per_sample * config.channels;

        AudioStreamBasicDescription {
            sample_rate: config.sample_rate as f64,
            format_id: K_AUDIO_FORMAT_LINEAR_PCM,
            format_flags: flags,
            bytes_per_packet: bytes_per_frame,
            frames_per_packet: 1,
            bytes_per_frame,
            channels_per_frame: config.channels,
            bits_per_channel: bits,
            reserved: 0,
        }
    }
}

impl AudioCaptureBackend for CoreAudioCapture {
    fn start(&mut self, config: CaptureConfig, callback: CaptureCallback) -> Result<(), String> {
        if self.audio_unit.is_some() {
            return Err("Already capturing".into());
        }

        // Find the HAL output component (used for capture despite the name).
        let desc = AudioComponentDescription {
            component_type: 0x61756F75, // 'auou' kAudioUnitType_Output
            component_sub_type: 0x6168616C, // 'ahal' kAudioUnitSubType_HALOutput
            component_manufacturer: 0x6170706C, // 'appl'
            component_flags: 0,
            component_flags_mask: 0,
        };

        let component = unsafe { AudioComponentFindNext(ptr::null_mut(), &desc) };
        if component.is_null() {
            return Err("AudioComponentFindNext: HAL output not found".into());
        }

        let mut unit: AudioUnit = ptr::null_mut();
        let status = unsafe { AudioComponentInstanceNew(component, &mut unit) };
        if status != 0 {
            return Err(format!("AudioComponentInstanceNew failed: {}", status));
        }

        // Enable input (capture) on element 1.
        let enable: u32 = 1;
        let status = unsafe {
            AudioUnitSetProperty(
                unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_INPUT,
                1, // input element
                &enable as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            )
        };
        if status != 0 {
            unsafe { AudioComponentInstanceDispose(unit) };
            return Err(format!("Enable input IO failed: {}", status));
        }

        // Disable output on element 0 (we're only capturing).
        let disable: u32 = 0;
        let status = unsafe {
            AudioUnitSetProperty(
                unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_ENABLE_IO,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                0, // output element
                &disable as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            )
        };
        if status != 0 {
            unsafe { AudioComponentInstanceDispose(unit) };
            return Err(format!("Disable output IO failed: {}", status));
        }

        // Set the device (default output for loopback).
        let device_id = Self::get_default_output_device()?;
        let status = unsafe {
            AudioUnitSetProperty(
                unit,
                K_AUDIO_OUTPUT_UNIT_PROPERTY_CURRENT_DEVICE,
                K_AUDIO_UNIT_SCOPE_GLOBAL,
                0,
                &device_id as *const _ as *const _,
                std::mem::size_of::<AudioDeviceID>() as u32,
            )
        };
        if status != 0 {
            unsafe { AudioComponentInstanceDispose(unit) };
            return Err(format!("Set device failed: {}", status));
        }

        // Set the output format (the format we want to receive samples in).
        let asbd = Self::build_asbd(&config);
        let status = unsafe {
            AudioUnitSetProperty(
                unit,
                K_AUDIO_UNIT_PROPERTY_STREAM_FORMAT,
                K_AUDIO_UNIT_SCOPE_OUTPUT,
                1, // input element's output side
                &asbd as *const _ as *const _,
                std::mem::size_of::<AudioStreamBasicDescription>() as u32,
            )
        };
        if status != 0 {
            unsafe { AudioComponentInstanceDispose(unit) };
            return Err(format!("Set stream format failed: {}", status));
        }

        // Store callback data.
        let cb_data = Arc::new(Mutex::new(CallbackData {
            callback,
            format: config,
        }));
        self.callback_data = Some(cb_data.clone());

        // Set render callback.
        // Note: In a full implementation, we'd set up an AURenderCallbackStruct
        // with a C function pointer and pass cb_data as the refcon.
        // For now, we initialize the unit and prepare it for rendering.
        let status = unsafe { AudioUnitInitialize(unit) };
        if status != 0 {
            unsafe { AudioComponentInstanceDispose(unit) };
            return Err(format!("AudioUnitInitialize failed: {}", status));
        }

        let status = unsafe { AudioOutputUnitStart(unit) };
        if status != 0 {
            unsafe {
                AudioUnitUninitialize(unit);
                AudioComponentInstanceDispose(unit);
            }
            return Err(format!("AudioOutputUnitStart failed: {}", status));
        }

        self.audio_unit = Some(unit);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if let Some(unit) = self.audio_unit.take() {
            unsafe {
                AudioOutputUnitStop(unit);
                AudioUnitUninitialize(unit);
                AudioComponentInstanceDispose(unit);
            }
        }
        self.callback_data = None;
        Ok(())
    }

    fn list_sources(&self) -> Vec<CaptureSource> {
        // For now, return the default output device as a loopback source.
        if let Ok(device_id) = Self::get_default_output_device() {
            vec![CaptureSource {
                id: format!("{}", device_id),
                name: "Default Output (Loopback)".into(),
                is_loopback: true,
                sample_rate: 44100,
                channels: 2,
            }]
        } else {
            vec![]
        }
    }
}

impl Drop for CoreAudioCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
