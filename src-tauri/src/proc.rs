use std::process::Command;

pub fn stop_process(pid: u32) -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        // win_suspend_process(pid);
        Ok(())
    } else {
        Command::new("kill")
            .arg("-STOP")
            .arg(format!("{}", pid))
            .output()?;
        Ok(())
    }
}

pub fn resume_process(pid: u32) -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        // win_resume_process(pid);
        Ok(())
    } else {
        Command::new("kill")
            .arg("-CONT")
            .arg(format!("{}", pid))
            .output()?;
        Ok(())
    }
}

// use std::ffi::c_void;
// use std::mem::size_of;
// use winapi::{
//     shared::{
//         ntdef::{HANDLE, NTSTATUS},
//         ntstatus::STATUS_SUCCESS,
//     },
//     um::{
//         handleapi::CloseHandle,
//         processthreadsapi::{OpenProcess, TerminateProcess},
//         winnt::{PROCESS_ALL_ACCESS, PROCESS_SUSPEND_RESUME},
//     },
// };

// type NtSuspendProcess = unsafe extern "system" fn(HANDLE) -> NTSTATUS;
// type NtResumeProcess = unsafe extern "system" fn(HANDLE) -> NTSTATUS;

// fn win_suspend_process(pid: u32) -> bool {
//     let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS | PROCESS_SUSPEND_RESUME, 0, pid) };
//     if process_handle.is_null() {
//         return false;
//     }

//     let nt_suspend_process: NtSuspendProcess = unsafe { std::mem::transmute(get_nt_function("NtSuspendProcess")) };
//     let status = unsafe { nt_suspend_process(process_handle) };

//     unsafe { CloseHandle(process_handle) };

//     status == STATUS_SUCCESS
// }

// fn win_resume_process(pid: u32) -> bool {
//     let process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS | PROCESS_SUSPEND_RESUME, 0, pid) };
//     if process_handle.is_null() {
//         return false;
//     }

//     let nt_resume_process: NtResumeProcess = unsafe { std::mem::transmute(get_nt_function("NtResumeProcess")) };
//     let status = unsafe { nt_resume_process(process_handle) };

//     unsafe { CloseHandle(process_handle) };

//     status == STATUS_SUCCESS
// }

// unsafe fn get_nt_function(name: &str) -> *const c_void {
//     use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
//     use std::ffi::CString;

//     let module_name = CString::new("ntdll.dll").unwrap();
//     let function_name = CString::new(name).unwrap();

//     let module_handle = GetModuleHandleA(module_name.as_ptr());
//     GetProcAddress(module_handle, function_name.as_ptr()) as *const c_void
// }