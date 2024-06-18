use std::ptr;

use crate::sys;
use super::MqStruct;

const C_EMPTY: *mut std::ffi::c_void = c"".as_ptr().cast_mut().cast();

// Zero length strings seem to require null termination
/// Returns a pointer to a string, with a nul termination for empty strings
const fn mq_str_ptr<T>(value: &str) -> *mut T {
    if value.is_empty() {
        C_EMPTY.cast()
    } else {
        value.as_ptr().cast_mut().cast()
    }
}

// Functions to attach references to MQCNO
impl<'ptr> MqStruct<'ptr, sys::MQCNO> {
    pub fn attach_csp(&mut self, csp: &'ptr sys::MQCSP) {
        self.SecurityParmsPtr = ptr::addr_of!(*csp).cast_mut();
    }

    pub fn attach_cd(&mut self, cd: &'ptr sys::MQCD) {
        self.ClientConnPtr = ptr::addr_of!(*cd).cast_mut().cast();
    }

    pub fn attach_sco(&mut self, sco: &'ptr sys::MQSCO) {
        self.SSLConfigPtr = ptr::addr_of!(*sco).cast_mut();
    }

    pub fn attach_bno(&mut self, bno: &'ptr sys::MQBNO) {
        self.BalanceParmsPtr = ptr::addr_of!(*bno).cast_mut();
    }

    pub fn attach_ccdt(&mut self, url: &'ptr str) {
        self.CCDTUrlPtr = mq_str_ptr(url);
        self.CCDTUrlLength = url
            .len()
            .try_into()
            .expect("CCDT url length exceeds maximum positive MQLONG");
    }
}

// Functions to attach references to MQCSP
impl<'ptr> MqStruct<'ptr, sys::MQCSP> {
    pub fn attach_password(&mut self, password: &'ptr str) {
        self.CSPPasswordPtr = mq_str_ptr(password);
        self.CSPPasswordLength = password
            .len()
            .try_into()
            .expect("Password length exceeds maximum positive MQLONG");
    }

    pub fn attach_userid(&mut self, userid: &'ptr str) {
        self.CSPUserIdPtr = mq_str_ptr(userid);
        self.CSPUserIdLength = userid
            .len()
            .try_into()
            .expect("User length exceeds maximum positive MQLONG");
    }

    pub fn attach_token(&mut self, token: &'ptr str) {
        self.TokenPtr = mq_str_ptr(token);
        self.TokenLength = token
            .len()
            .try_into()
            .expect("Token length exceeds maximum positive MQLONG");
    }

    pub fn attach_initial_key(&mut self, initial_key: &'ptr str) {
        self.InitialKeyPtr = mq_str_ptr(initial_key);
        self.InitialKeyLength = initial_key
            .len()
            .try_into()
            .expect("Initial key length exceeds maximum positive MQLONG");
    }
}

// Functions to attach references to MQSCO
impl<'ptr> MqStruct<'ptr, sys::MQSCO> {
    pub fn attach_repo_password(&mut self, password: &'ptr str) {
        self.KeyRepoPasswordPtr = mq_str_ptr(password);
        self.KeyRepoPasswordLength = password
            .len()
            .try_into()
            .expect("Password length exceeds maximum positive MQLONG");
    }

    pub fn attach_auth_info_records(&mut self, air: &'ptr [sys::MQAIR]) {
        self.AuthInfoRecPtr = air.as_ptr().cast_mut();
        self.AuthInfoRecCount = air
            .len()
            .try_into()
            .expect("Auth info record count exceeds maximum positive MQLONG");
    }
}
