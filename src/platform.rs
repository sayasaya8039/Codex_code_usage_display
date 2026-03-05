/// Set the window's alpha opacity using Win32 API.
/// `opacity` is 0.0 (fully transparent) to 1.0 (fully opaque).
#[cfg(windows)]
pub fn set_window_opacity(window: &gpui::Window, opacity: f32) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle: raw_window_handle::WindowHandle<'_> = match HasWindowHandle::window_handle(window) {
        Ok(h) => h,
        Err(_) => return,
    };

    let hwnd_ptr = match handle.as_raw() {
        RawWindowHandle::Win32(win32) => win32.hwnd.get() as isize,
        _ => return,
    };

    let hwnd = windows::Win32::Foundation::HWND(hwnd_ptr as *mut _);
    let alpha = (opacity.clamp(0.1, 1.0) * 255.0) as u8;

    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::*;

        // Add WS_EX_LAYERED style
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED.0 as i32);

        // Set alpha
        let _ = SetLayeredWindowAttributes(
            hwnd,
            windows::Win32::Foundation::COLORREF(0),
            alpha,
            LWA_ALPHA,
        );
    }
}

#[cfg(not(windows))]
pub fn set_window_opacity(_window: &gpui::Window, _opacity: f32) {}

#[cfg(windows)]
pub fn minimize_window(window: &gpui::Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let handle: raw_window_handle::WindowHandle<'_> = match HasWindowHandle::window_handle(window) {
        Ok(h) => h,
        Err(_) => return,
    };

    let hwnd_ptr = match handle.as_raw() {
        RawWindowHandle::Win32(win32) => win32.hwnd.get() as isize,
        _ => return,
    };

    let hwnd = windows::Win32::Foundation::HWND(hwnd_ptr as *mut _);
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{SW_MINIMIZE, ShowWindow};
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
    }
}

#[cfg(not(windows))]
pub fn minimize_window(_window: &gpui::Window) {}

#[cfg(windows)]
pub fn set_startup_enabled(enabled: bool) -> Result<(), String> {
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_OPTION_NON_VOLATILE, REG_SAM_FLAGS, REG_SZ,
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegSetValueExW,
    };
    use windows::core::{PCWSTR, w};

    const APP_NAME: PCWSTR = w!("CodexUsageWidget");
    const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

    let mut key = HKEY::default();
    let create_result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY,
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            REG_SAM_FLAGS(KEY_SET_VALUE.0),
            None,
            &mut key,
            None,
        )
    };
    if create_result != windows::Win32::Foundation::ERROR_SUCCESS {
        return Err(format!("Runキーのオープンに失敗: {create_result:?}"));
    }

    let result = if enabled {
        let exe_path =
            std::env::current_exe().map_err(|e| format!("実行ファイルパス取得失敗: {e}"))?;
        let value = format!("\"{}\"", exe_path.display());
        let mut wide: Vec<u16> = value.encode_utf16().collect();
        wide.push(0);
        let bytes = unsafe {
            std::slice::from_raw_parts(
                wide.as_ptr() as *const u8,
                wide.len() * std::mem::size_of::<u16>(),
            )
        };

        let set_result = unsafe { RegSetValueExW(key, APP_NAME, Some(0), REG_SZ, Some(bytes)) };
        if set_result == windows::Win32::Foundation::ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("スタートアップ登録に失敗: {set_result:?}"))
        }
    } else {
        let delete_result = unsafe { RegDeleteValueW(key, APP_NAME) };
        if delete_result == windows::Win32::Foundation::ERROR_SUCCESS
            || delete_result == windows::Win32::Foundation::WIN32_ERROR(2)
        {
            Ok(())
        } else {
            Err(format!("スタートアップ解除に失敗: {delete_result:?}"))
        }
    };

    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}

#[cfg(not(windows))]
pub fn set_startup_enabled(_enabled: bool) -> Result<(), String> {
    Ok(())
}
