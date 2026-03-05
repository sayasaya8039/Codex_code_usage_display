/// Set the window's alpha opacity using Win32 API.
/// `opacity` is 0.0 (fully transparent) to 1.0 (fully opaque).
#[cfg(windows)]
pub fn set_window_opacity(window: &gpui::Window, opacity: f32) {
    let Some(hwnd) = hwnd_from_window(window) else {
        return;
    };
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
pub fn initialize_window_icons(window: &gpui::Window) {
    let Some(hwnd) = hwnd_from_window(window) else {
        return;
    };
    if let Some((large, small)) = load_icons_from_exe() {
        unsafe {
            use windows::Win32::Foundation::{LPARAM, WPARAM};
            use windows::Win32::UI::WindowsAndMessaging::{
                ICON_BIG, ICON_SMALL, SendMessageW, WM_SETICON,
            };
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_SMALL as usize)),
                Some(LPARAM(small.0 as isize)),
            );
            let _ = SendMessageW(
                hwnd,
                WM_SETICON,
                Some(WPARAM(ICON_BIG as usize)),
                Some(LPARAM(large.0 as isize)),
            );
        }
    }
}

#[cfg(not(windows))]
pub fn initialize_window_icons(_window: &gpui::Window) {}

#[cfg(windows)]
pub fn minimize_window(window: &gpui::Window) {
    let Some(hwnd) = hwnd_from_window(window) else {
        return;
    };
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{SW_MINIMIZE, ShowWindow};
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
    }
}

#[cfg(not(windows))]
pub fn minimize_window(_window: &gpui::Window) {}

#[cfg(windows)]
pub fn hide_window_to_tray(window: &gpui::Window) -> Result<(), String> {
    let Some(hwnd) = hwnd_from_window(window) else {
        return Err("ウィンドウハンドル取得失敗".to_string());
    };

    let old = TRAY_OLD_WNDPROC.load(std::sync::atomic::Ordering::SeqCst);
    if old == 0 {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{GWLP_WNDPROC, SetWindowLongPtrW};
            let prev = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, tray_wndproc as isize);
            if prev != 0 {
                TRAY_OLD_WNDPROC.store(prev, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }

    add_tray_icon(hwnd)?;

    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
        let _ = ShowWindow(hwnd, SW_HIDE);
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn hide_window_to_tray(_window: &gpui::Window) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
pub fn remove_tray_icon() {
    let hwnd_val = TRAY_WINDOW_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if hwnd_val != 0 {
        let hwnd = windows::Win32::Foundation::HWND(hwnd_val as *mut _);
        let _ = remove_tray_icon_for(hwnd);
    }
}

#[cfg(not(windows))]
pub fn remove_tray_icon() {}

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

#[cfg(windows)]
fn hwnd_from_window(window: &gpui::Window) -> Option<windows::Win32::Foundation::HWND> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle: raw_window_handle::WindowHandle<'_> =
        HasWindowHandle::window_handle(window).ok()?;
    let hwnd_ptr = match handle.as_raw() {
        RawWindowHandle::Win32(win32) => win32.hwnd.get() as isize,
        _ => return None,
    };
    Some(windows::Win32::Foundation::HWND(hwnd_ptr as *mut _))
}

#[cfg(windows)]
fn load_icons_from_exe() -> Option<(
    windows::Win32::UI::WindowsAndMessaging::HICON,
    windows::Win32::UI::WindowsAndMessaging::HICON,
)> {
    use windows::Win32::UI::Shell::ExtractIconExW;
    use windows::core::PCWSTR;

    let exe = std::env::current_exe().ok()?;
    let mut wide: Vec<u16> = exe.as_os_str().to_string_lossy().encode_utf16().collect();
    wide.push(0);

    let mut large = windows::Win32::UI::WindowsAndMessaging::HICON::default();
    let mut small = windows::Win32::UI::WindowsAndMessaging::HICON::default();
    let count = unsafe {
        ExtractIconExW(
            PCWSTR(wide.as_ptr()),
            0,
            Some(&mut large),
            Some(&mut small),
            1,
        )
    };
    if count > 0 {
        Some((large, small))
    } else {
        None
    }
}

#[cfg(windows)]
fn copy_utf16_truncate(dst: &mut [u16], text: &str) {
    let encoded: Vec<u16> = text.encode_utf16().collect();
    let max = dst.len().saturating_sub(1).min(encoded.len());
    if max > 0 {
        dst[..max].copy_from_slice(&encoded[..max]);
    }
    dst[max] = 0;
}

#[cfg(windows)]
const TRAY_ICON_ID: u32 = 1;
#[cfg(windows)]
const WM_TRAY_ICON: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 1;
#[cfg(windows)]
static TRAY_ICON_VISIBLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(windows)]
static TRAY_WINDOW_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
#[cfg(windows)]
static TRAY_OLD_WNDPROC: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

#[cfg(windows)]
fn add_tray_icon(hwnd: windows::Win32::Foundation::HWND) -> Result<(), String> {
    if TRAY_ICON_VISIBLE.load(std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }

    use windows::Win32::UI::Shell::{
        NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_SETVERSION, NOTIFYICON_VERSION_4,
        NOTIFYICONDATAW, Shell_NotifyIconW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{HICON, IDI_APPLICATION, LoadIconW};

    let (_, small_icon) = load_icons_from_exe().unwrap_or_else(|| unsafe {
        let default_icon: HICON = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
        (default_icon, default_icon)
    });

    let mut data = NOTIFYICONDATAW::default();
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ICON_ID;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = WM_TRAY_ICON;
    data.hIcon = small_icon;
    copy_utf16_truncate(&mut data.szTip, "Codex Usage Widget");

    let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &data) }.as_bool();
    if !ok {
        return Err("トレイアイコンの追加に失敗".to_string());
    }

    unsafe {
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let _ = Shell_NotifyIconW(NIM_SETVERSION, &data);
    }

    TRAY_WINDOW_HWND.store(hwnd.0 as isize, std::sync::atomic::Ordering::SeqCst);
    TRAY_ICON_VISIBLE.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[cfg(windows)]
fn remove_tray_icon_for(hwnd: windows::Win32::Foundation::HWND) -> bool {
    if !TRAY_ICON_VISIBLE.swap(false, std::sync::atomic::Ordering::SeqCst) {
        return false;
    }

    use windows::Win32::UI::Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW};
    let mut data = NOTIFYICONDATAW::default();
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ICON_ID;
    unsafe { Shell_NotifyIconW(NIM_DELETE, &data) }.as_bool()
}

#[cfg(windows)]
unsafe extern "system" fn tray_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, SW_RESTORE, SetForegroundWindow, ShowWindow, WM_DESTROY,
        WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_RBUTTONUP, WNDPROC,
    };

    if msg == WM_TRAY_ICON {
        let event = lparam.0 as u32;
        if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK || event == WM_RBUTTONUP {
            let _ = remove_tray_icon_for(hwnd);
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
            return LRESULT(0);
        }
    }

    if msg == WM_DESTROY {
        let _ = remove_tray_icon_for(hwnd);
    }

    let old = TRAY_OLD_WNDPROC.load(std::sync::atomic::Ordering::SeqCst);
    if old != 0 {
        let old_proc: WNDPROC = std::mem::transmute(old);
        return CallWindowProcW(old_proc, hwnd, msg, wparam, lparam);
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
