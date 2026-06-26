use crate::{CaptureError, Result};
use std::mem::size_of;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE,
    REG_OPEN_CREATE_OPTIONS, REG_SZ,
};

pub(super) fn set_directx_user_global_settings() -> Result<()> {
    let key_path = wide_null("Software\\Microsoft\\DirectX\\UserGpuPreferences");
    let value_name = wide_null("DirectXUserGlobalSettings");
    let value_data = wide_null("SwapEffectUpgradeEnable=0;");
    let bytes = unsafe {
        std::slice::from_raw_parts(
            value_data.as_ptr().cast::<u8>(),
            value_data.len() * size_of::<u16>(),
        )
    };

    let mut key = HKEY::default();
    let create_error = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            None,
            PCWSTR::null(),
            REG_OPEN_CREATE_OPTIONS(0),
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
    };
    if create_error.0 != 0 {
        return Err(CaptureError::Win32(format!(
            "RegCreateKeyExW: {}",
            create_error.0
        )));
    }

    let set_error =
        unsafe { RegSetValueExW(key, PCWSTR(value_name.as_ptr()), None, REG_SZ, Some(bytes)) };
    unsafe {
        let _ = RegCloseKey(key);
    }

    if set_error.0 != 0 {
        return Err(CaptureError::Win32(format!(
            "RegSetValueExW: {}",
            set_error.0
        )));
    }

    Ok(())
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
