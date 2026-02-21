use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, FindWindowExW, FindWindowW, SendMessageTimeoutW, SMTO_NORMAL,
};

use windows_core::BOOL;

pub fn get_worker_w() -> Option<HWND> {
    unsafe {
        let progman = FindWindowW(w!("Progman"), PCWSTR::null())
            .expect("Failed to find Progman window");

        let mut result = 0;
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );

        std::thread::sleep(std::time::Duration::from_millis(500));

        let mut best_worker: HWND = HWND::default();
        
        unsafe extern "system" fn enum_child_callback(
            hwnd: HWND,
            lparam: LPARAM,
        ) -> BOOL {
            let p_best = lparam.0 as *mut HWND;
            
            let mut class_name = [0u16; 256];
            let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetClassNameW(hwnd, &mut class_name) };
            if len == 0 { return BOOL(1); }
            let class_string = String::from_utf16_lossy(&class_name[..len as usize]);
            
            if class_string == "WorkerW" {
                let mut rect = windows::Win32::Foundation::RECT::default();
                unsafe {
                    let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect);
                }
                
                
                if (rect.right - rect.left) > 500 && (rect.bottom - rect.top) > 500 {
                    unsafe { *p_best = hwnd };
                }
            }
            BOOL(1)
        }
        
        let _ = EnumChildWindows(Some(progman), Some(enum_child_callback), LPARAM(&mut best_worker as *mut _ as isize));

        if best_worker.0 != std::ptr::null_mut() {
            return Some(best_worker);
        }

        struct EnumData {
            shell_container: HWND,
            worker_candidates: Vec<HWND>,
        }

        let mut data = EnumData {
            shell_container: HWND::default(),
            worker_candidates: Vec::new(),
        };

        unsafe extern "system" fn enum_callback(
            tophandle: HWND,
            topparamhandle: LPARAM,
        ) -> BOOL {
            let data = unsafe { &mut *(topparamhandle.0 as *mut EnumData) };
            
            let mut class_name = [0u16; 256];
            let len = unsafe { windows::Win32::UI::WindowsAndMessaging::GetClassNameW(tophandle, &mut class_name) };
            if len == 0 { return BOOL(1); }
            let class_string = String::from_utf16_lossy(&class_name[..len as usize]);

            if class_string == "WorkerW" || class_string == "Progman" {
                let defview = unsafe { FindWindowExW(Some(tophandle), None, w!("SHELLDLL_DefView"), PCWSTR::null()) };
                if let Ok(dv) = defview {
                    if dv.0 != std::ptr::null_mut() {
                        data.shell_container = tophandle;
                    } else if class_string == "WorkerW" {
                        data.worker_candidates.push(tophandle);
                    }
                } else if class_string == "WorkerW" {
                    data.worker_candidates.push(tophandle);
                }
            }

            BOOL(1)
        }

        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut data as *mut _ as isize),
        );

        let mut best_top: HWND = HWND::default();
        for &hwnd in &data.worker_candidates {
            let mut rect = windows::Win32::Foundation::RECT::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            let child = windows::Win32::UI::WindowsAndMessaging::FindWindowExW(Some(hwnd), None, PCWSTR::null(), PCWSTR::null());
            let has_child = child.is_ok() && child.unwrap().0 != std::ptr::null_mut();
            
            if !has_child && width > 500 && height > 500 {
                best_top = hwnd;
            }
        }

        if best_top.0 != std::ptr::null_mut() {
            return Some(best_top);
        }

        if data.shell_container.0 != std::ptr::null_mut() {
            let next_sibling = FindWindowExW(None, Some(data.shell_container), w!("WorkerW"), PCWSTR::null());
            if let Ok(w) = next_sibling {
                if w.0 != std::ptr::null_mut() {
                    return Some(w);
                }
            }
        }

        if let Some(&last) = data.worker_candidates.last() {
            return Some(last);
        }

        None
    }
}
