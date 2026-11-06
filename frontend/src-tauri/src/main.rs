// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

type DvRunFn = unsafe extern "C" fn() -> i32;

#[cfg(target_os = "windows")]
extern "system" {
    fn SetDllDirectoryW(lpPathName: *const u16) -> i32;
    fn LoadLibraryW(lpLibFileName: *const u16) -> *mut std::ffi::c_void;
    fn GetProcAddress(
        hModule: *mut std::ffi::c_void,
        lpProcName: *const u8,
    ) -> Option<unsafe extern "system" fn() -> isize>;
    fn MessageBoxW(
        hWnd: *mut std::ffi::c_void,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

fn append_startup_log(msg: &str) {
    if let Some(mut dir) = dirs::data_dir() {
        dir.push("DigitalVigital");
        let _ = std::fs::create_dir_all(&dir);
        dir.push("startup.log");
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir) {
            let _ = writeln!(f, "[Launcher EXE] {}", msg);
        }
    }
}

fn main() {
    append_startup_log("Process main() entry point reached");

    #[cfg(target_os = "windows")]
    {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        append_startup_log(&format!("Application root directory: {:?}", exe_dir));

        // 1. Tell Windows to resolve DLLs from application directory
        use std::os::windows::ffi::OsStrExt;
        let dir_w: Vec<u16> = exe_dir.as_os_str().encode_wide().chain(Some(0)).collect();
        unsafe {
            SetDllDirectoryW(dir_w.as_ptr());
        }

        // 2. Load dv_core.dll
        let dll_path = exe_dir.join("dv_core.dll");
        append_startup_log(&format!("Loading core engine from: {:?}", dll_path));

        let dll_w: Vec<u16> = dll_path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut h_module = unsafe { LoadLibraryW(dll_w.as_ptr()) };

        if h_module.is_null() {
            // Fallback: try relative load
            let fallback_w = to_wide("dv_core.dll");
            h_module = unsafe { LoadLibraryW(fallback_w.as_ptr()) };
        }

        if h_module.is_null() {
            append_startup_log("CRITICAL ERROR: Failed to load dv_core.dll");
            let title = to_wide("Digital Vigital Launcher - Missing Core Engine");
            let msg = to_wide(
                "Could not load 'dv_core.dll'.\n\n\
                Please reinstall Digital Vigital Launcher or ensure 'dv_core.dll' \
                and 'WebView2Loader.dll' exist in the application directory.",
            );
            unsafe {
                MessageBoxW(std::ptr::null_mut(), msg.as_ptr(), title.as_ptr(), 0x10); // MB_ICONERROR
            }
            std::process::exit(1);
        }

        // 3. Resolve entry point `dv_run`
        let sym = unsafe { GetProcAddress(h_module, b"dv_run\0".as_ptr()) };
        if let Some(proc) = sym {
            append_startup_log("Executing dv_run entry point");
            let dv_run: DvRunFn = unsafe { std::mem::transmute(proc) };
            let exit_code = unsafe { dv_run() };
            append_startup_log(&format!("Process completed with exit code: {}", exit_code));
            std::process::exit(exit_code);
        } else {
            append_startup_log("CRITICAL ERROR: dv_run symbol missing in dv_core.dll");
            let title = to_wide("Digital Vigital Launcher - Corrupt Core");
            let msg = to_wide("The entry point 'dv_run' was not found in 'dv_core.dll'.");
            unsafe {
                MessageBoxW(std::ptr::null_mut(), msg.as_ptr(), title.as_ptr(), 0x10);
            }
            std::process::exit(1);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("Digital Vigital Launcher requires Windows.");
    }
}
