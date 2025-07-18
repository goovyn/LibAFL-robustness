use core::{
    ffi::{c_char, c_long, c_ulong, c_void},
    slice::from_raw_parts,
};

use log::trace;

use crate::{asan_load, asan_panic};

/// # Safety
/// See man pages
#[unsafe(export_name = "patch_atol")]
pub unsafe extern "C" fn atol(s: *const c_char) -> c_long {
    unsafe {
        trace!("atol - s: {s:p}");

        if s.is_null() {
            asan_panic(c"atol - s is null".as_ptr() as *const c_char);
        }

        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        asan_load(s as *const c_void, len + 1);
        let slice = from_raw_parts(s, len);

        let mut i = 0;

        let ws = [
            0x20, /* ' ' */
            0xc,  /* \f */
            0xa,  /* \n */
            0xd,  /* \r */
            0x9,  /* \t */
            0xb,  /* \v */
        ];
        while ws.contains(&slice[i]) {
            i += 1;
        }

        let mut negative = false;
        if slice[i] == 0x2d
        /* '-' */
        {
            negative = true;
            i += 1;
        } else if slice[i] == 0x2b
        /* '+' */
        {
            i += 1;
        }

        let mut val = 0 as c_ulong;
        for c in slice.iter().skip(i) {
            if ('0' as c_char..='9' as c_char).contains(c) {
                match val.checked_mul(10) {
                    Some(m) => val = m,
                    None => asan_panic(c"atoi - overflow #1".as_ptr() as *const c_char),
                }
                let digit = (c - '0' as c_char) as c_ulong;
                match val.checked_add(digit) {
                    Some(a) => val = a,
                    None => asan_panic(c"atoi - overflow #2".as_ptr() as *const c_char),
                }
            } else {
                break;
            }
        }

        if val == 0 {
            0
        } else if negative {
            if val > (c_long::MAX as c_ulong) + 1 {
                asan_panic(c"atoi - overflow #3".as_ptr() as *const c_char);
            }
            -((val - 1) as c_long) - 1
        } else {
            if val > c_long::MAX as c_ulong {
                asan_panic(c"atoi - overflow #4".as_ptr() as *const c_char);
            }
            val as c_long
        }
    }
}
