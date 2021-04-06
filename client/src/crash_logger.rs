use winapi::shared::minwindef::{DWORD, HMODULE};
use winapi::um::errhandlingapi::{SetUnhandledExceptionFilter, PTOP_LEVEL_EXCEPTION_FILTER};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::{
    EnumProcessModules, GetModuleFileNameExA, GetModuleInformation, MODULEINFO,
};
use winapi::um::winnt::{EXCEPTION_POINTERS, LONG};
use winapi::vc::excpt::EXCEPTION_CONTINUE_SEARCH;

use std::io::Write;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static mut EXCEPTION_FILTER: PTOP_LEVEL_EXCEPTION_FILTER = None;
static mut PLAYTIME: Option<Instant> = None;
static mut ALREADY_SENT: bool = false;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CrashReport {
    // player relative
    mem_used: u32,
    mem_available: u32,
    // exception
    base_addr: usize,
    exception_addr: usize,
    exception_code: usize,
    exception_library: String,
    registers: Registers,
    modules: Vec<Module>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Registers {
    eax: DWORD,
    ebx: DWORD,
    ecx: DWORD,
    edx: DWORD,
    esi: DWORD,
    edi: DWORD,
    ebp: DWORD,
    esp: DWORD,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Module {
    name: String,
    addr: usize,
    size: usize,
}

pub fn initialize() {
    unsafe {
        EXCEPTION_FILTER = SetUnhandledExceptionFilter(Some(exception_filter));
        PLAYTIME = Some(Instant::now());
    }
}

unsafe extern "system" fn exception_filter(exception_info: *mut EXCEPTION_POINTERS) -> LONG {
    if ALREADY_SENT {
        if let Some(origin) = EXCEPTION_FILTER.as_mut() {
            return origin(exception_info);
        }

        return EXCEPTION_CONTINUE_SEARCH;
    } else {
        ALREADY_SENT = true;
    }

    let info = &mut *exception_info;
    let context = &mut *info.ContextRecord;
    let exception = &mut *info.ExceptionRecord;

    let registers = Registers {
        eax: context.Eax,
        ebx: context.Ebx,
        ecx: context.Ecx,
        edx: context.Edx,
        esi: context.Esi,
        edi: context.Edi,
        ebp: context.Ebp,
        esp: context.Esp,
    };

    let process = GetCurrentProcess();
    let mut module_handles: [HMODULE; 1024] = [0 as *mut _; 1024];
    let mut found = 0;

    EnumProcessModules(
        process,
        module_handles.as_mut_ptr(),
        module_handles.len() as _,
        &mut found,
    );

    let mut bytes = [0i8; 1024];
    let mut modules = Vec::with_capacity((found / 4) as usize);
    let mut module_information = MODULEINFO {
        lpBaseOfDll: std::ptr::null_mut(),
        SizeOfImage: 0,
        EntryPoint: std::ptr::null_mut(),
    };
    let mut exception_library = String::from("Unknown module");

    for i in 0..(found / 4) {
        if GetModuleFileNameExA(
            process,
            module_handles[i as usize],
            bytes.as_mut_ptr(),
            1024,
        ) != 0
            && GetModuleInformation(
                process,
                module_handles[i as usize],
                &mut module_information,
                std::mem::size_of::<MODULEINFO>() as _,
            ) != 0
        {
            let string = std::ffi::CStr::from_ptr(bytes.as_ptr());

            let e_addr = exception.ExceptionAddress as usize;
            let m_addr = module_handles[i as usize] as usize;
            let m_size = module_information.SizeOfImage as usize;

            if e_addr >= m_addr && e_addr < m_addr + m_size {
                exception_library = string.to_string_lossy().to_string();
            }

            modules.push(Module {
                name: string.to_string_lossy().to_string(),
                addr: m_addr,
                size: m_size,
            });
        }
    }

    let report = CrashReport {
        mem_used: *(0x8E4CB4 as *mut u32),
        mem_available: *(0x8A5A80 as *mut u32),
        base_addr: client_api::samp::handle() as usize,
        exception_addr: exception.ExceptionAddress as usize,
        exception_code: exception.ExceptionCode as usize,
        exception_library,
        registers,
        modules,
    };

    log::trace!("{}", serde_json::to_string_pretty(&report).unwrap());

    if let Some(origin) = EXCEPTION_FILTER.as_mut() {
        return origin(exception_info);
    }

    return EXCEPTION_CONTINUE_SEARCH;
}
