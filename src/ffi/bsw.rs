use schemes::bsw::*;
use std::ops::Deref;
use libc::*;
use std::ffi::CStr;
use std::mem::transmute;
use std::string::String;
use serde_json;

#[no_mangle]
pub extern "C" fn bsw_context_create() -> *mut CpAbeContext {
    let (_pk,_msk) = setup();
    let _ctx = unsafe { transmute(Box::new(CpAbeContext {
      _pk: _pk,
      _msk: _msk
    })) };
    _ctx
}

#[no_mangle]
pub extern "C" fn bsw_context_destroy(ctx: *mut CpAbeContext) {
    let _ctx: Box<CpAbeContext> = unsafe { transmute(ctx) };
    _ctx.deref();
}

#[no_mangle]
pub extern "C" fn bsw_keygen(
    ctx: *mut CpAbeContext,
    attributes: *const c_char
) -> *mut CpAbeSecretKey {
    //let _attr = unsafe { &mut *attributes };
    let _cstr = unsafe { CStr::from_ptr(attributes).to_str().unwrap() };
    let mut _attrs = _cstr.split(",");
    let mut _attr_vec = Vec::new();
    for _a in _attrs {
        _attr_vec.push(String::from(_a));
    }
    //let attr_vec: Vec<_> = _attr.iter().map(|arg| arg.to_string()).collect();
    let _ctx = unsafe { &*ctx };
    let _sk = unsafe {
        transmute(Box::new(
            keygen(&(_ctx._pk), &(_ctx._msk), &_attr_vec).unwrap(),
        ))
    };
    _sk
}

#[no_mangle]
pub extern "C" fn bsw_keygen_destroy(sk: *mut CpAbeSecretKey) {
    let _sk: Box<CpAbeSecretKey> = unsafe { transmute(sk) };
    _sk.deref();
}

#[no_mangle]
pub extern "C" fn bsw_delegate(
    ctx: *mut CpAbeContext,
    sk: *mut CpAbeSecretKey,
    attributes: *mut Vec<String>,
) -> *mut CpAbeSecretKey {
    let _attr = unsafe { &mut *attributes };
    let attr_vec: Vec<_> = _attr.iter().map(|arg| arg.to_string()).collect();
    let _ctx = unsafe { &*ctx };
    let _sk = unsafe { &*sk };
    let _dsk = unsafe { transmute(Box::new(delegate(&_ctx._pk, &_sk, &attr_vec).unwrap())) };
    _dsk
}

#[no_mangle]
pub extern "C" fn bsw_encrypt_size(
    ctx: *mut CpAbeContext,
    policy: *mut c_char,
    data: *mut u8,
    data_len: u32,
) -> i32 {
    use std::{slice,ptr};
    let p = unsafe { &mut *policy };
    let mut _pol = unsafe { CStr::from_ptr(p) };
    let mut pol = String::with_capacity(_pol.to_bytes().len());
    pol.insert_str (0, _pol.to_str().unwrap());
    let _ctx = unsafe { &*ctx };
    let _data = unsafe { &mut *data };
    let _slice = unsafe { slice::from_raw_parts(data, data_len as usize) };
    let mut _data_vec = vec![0];
    _data_vec.extend_from_slice(_slice);
    
    //TODO is there a way to calculate the CT size without actually encrypting it//TODO is there a
    //way to calculate the CT size without actually encrypting it??
    let _ct = encrypt(&_ctx._pk, &pol, &_data_vec).unwrap();
    let _ct_str = serde_json::to_string_pretty(&_ct).unwrap();
    let _len = _ct_str.len() as i32;
    _len
}

#[no_mangle]
pub extern "C" fn bsw_encrypt(
    ctx: *mut CpAbeContext,
    policy: *mut c_char,
    data: *mut u8,
    data_len: u32
) -> *mut CpAbeCiphertext {
    use std::{slice};
    let p = unsafe { &mut *policy };
    let mut _pol = unsafe { CStr::from_ptr(p) };
    let mut pol = String::with_capacity(_pol.to_bytes().len());
    pol.insert_str (0, _pol.to_str().unwrap());
    let _ctx = unsafe { &*ctx };
    let _data = unsafe { &mut *data };
    let _slice = unsafe { slice::from_raw_parts(data, data_len as usize) };
    let mut _data_vec = Vec::new();
    _data_vec.extend_from_slice(_slice);
    let _ct = unsafe { transmute(Box::new(encrypt(&(_ctx._pk), &pol, &_data_vec).unwrap())) };
    _ct
}

#[no_mangle]
pub extern "C" fn bsw_decrypt_get_size(ct: *mut CpAbeCiphertext) -> u32 {
    let _ct = unsafe { &mut *ct };
    _ct.pt_len
}

#[no_mangle]
pub extern "C" fn bsw_decrypt(
    sk: *mut CpAbeSecretKey,
    ct: *mut CpAbeCiphertext,
    buf: *mut u8) {
    use std::{ptr};
    
    let _sk = unsafe { &mut *sk };
    let _ct = unsafe { &mut *ct };
    let _pt = unsafe { decrypt(_sk, _ct).unwrap() };
    unsafe { ptr::copy_nonoverlapping(&_pt.as_slice()[0], buf, _ct.pt_len as usize) };
}