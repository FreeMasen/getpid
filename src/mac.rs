#![cfg(target_os = "macos")]
use super::Error;

use sysctl::{kinfo_proc, sysctl};
pub(crate) fn get_processes() -> Result<Vec<(usize, String)>, Error> {
    let mut name: [i32; 4] = [1, 14, 0, 0];
    let name_len: u32 = 4;
    let mut len = 0;
    let mut err: i32 = unsafe {
        sysctl(
            name.as_mut_ptr(),
            name_len,
            ::std::ptr::null_mut(),
            &mut len,
            ::std::ptr::null_mut(),
            0
        )
    };

    if err != 0 {
        return Err(Error::Other(format!("Error getting length of list: {}", err)));

    }
    let expecting = len / ::std::mem::size_of::<kinfo_proc>();
    let layout = ::std::alloc::Layout::new::<kinfo_proc>();
    let ptr = unsafe { ::std::alloc::alloc_zeroed(layout) };

    let mut list: Vec<kinfo_proc> = unsafe { Vec::from_raw_parts(ptr as *mut kinfo_proc, expecting, expecting) };
    err = unsafe {
        sysctl(
            name.as_mut_ptr(),
            name_len,
            list.as_mut_ptr() as *mut ::std::os::raw::c_void,
            &mut len,
            ::std::ptr::null_mut(),
            0,
        )
    };
    if err != 0 {
        return Err(Error::Other(format!("Error getting kinfo_proc list: {}", err)));
    }
    Ok(list.iter().filter_map(|p| match format_name(&p.kp_proc.p_comm) {
        Ok(name) => {
            Some((p.kp_proc.p_pid as usize, name))
        },
        Err(e) => {
            eprintln!("Error parsing name: {}", e);
            None
        }
    }).collect())
}

fn format_name(buf: &[i8]) -> Result<String, Error> {
    let mut bytes = vec![];
    for n in buf {
        if *n as u8 == '\u{0}' as u8 {
            break;
        } else {
            bytes.push(*n as u8);
        }
    }
    let string = String::from_utf8_lossy(&bytes);
    Ok(string.to_string())
}
