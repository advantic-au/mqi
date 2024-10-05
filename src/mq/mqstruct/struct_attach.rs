use std::ptr;

use super::{impl_mqstruct_min_version, MqStruct};
use crate::{sys, values::CCSID, EncodedString};

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

fn set_mqcharv(mqcharv: &mut sys::MQCHARV, data: &[u8], ccsid: CCSID) {
    mqcharv.VSPtr = ptr::from_ref(data).cast_mut().cast();
    mqcharv.VSLength = data.len().try_into().expect("length should convert to MQLONG");
    mqcharv.VSCCSID = ccsid.0;
}

impl<'ptr> MqStruct<'ptr, sys::MQOD> {
    pub fn attach_selection_string<S: EncodedString + ?Sized>(&mut self, selection: &'ptr S) {
        self.set_min_version(sys::MQOD_VERSION_4);
        set_mqcharv(&mut self.SelectionString, selection.data(), selection.ccsid());
    }

    pub fn attach_object_string<S: EncodedString + ?Sized>(&mut self, object: &'ptr S) {
        self.set_min_version(sys::MQOD_VERSION_4);
        set_mqcharv(&mut self.ObjectString, object.data(), object.ccsid());
    }
}

impl<'ptr> MqStruct<'ptr, sys::MQSD> {
    pub fn attach_object_string<S: EncodedString + ?Sized>(&mut self, object: &'ptr S) {
        set_mqcharv(&mut self.ObjectString, object.data(), object.ccsid());
    }
}

// Functions to attach references to MQCNO
impl<'ptr> MqStruct<'ptr, sys::MQCNO> {
    pub fn attach_csp(&mut self, csp: &'ptr MqStruct<sys::MQCSP>) {
        self.set_min_version(sys::MQCNO_VERSION_5);
        self.SecurityParmsPtr = ptr::addr_of!(csp.struc).cast_mut();
    }

    pub fn attach_cd(&mut self, cd: &'ptr MqStruct<sys::MQCD>) {
        self.set_min_version(sys::MQCNO_VERSION_2);
        self.ClientConnPtr = ptr::addr_of!(cd.struc).cast_mut().cast();
    }

    pub fn attach_sco(&mut self, sco: &'ptr MqStruct<sys::MQSCO>) {
        self.set_min_version(sys::MQCNO_VERSION_4);
        self.SSLConfigPtr = ptr::addr_of!(sco.struc).cast_mut();
    }

    pub fn attach_bno(&mut self, bno: &'ptr MqStruct<sys::MQBNO>) {
        self.set_min_version(sys::MQCNO_VERSION_8);
        self.BalanceParmsPtr = ptr::addr_of!(bno.struc).cast_mut();
    }

    pub fn attach_ccdt(&mut self, url: &'ptr str) {
        self.set_min_version(sys::MQCNO_VERSION_6);
        self.CCDTUrlPtr = mq_str_ptr(url);
        self.CCDTUrlLength = url
            .len()
            .try_into()
            .expect("CCDT url length should not exceed maximum positive MQLONG");
    }
}

impl_mqstruct_min_version!(sys::MQCSP);

// Functions to attach references to MQCSP
impl<'ptr> MqStruct<'ptr, sys::MQCSP> {
    pub fn attach_password(&mut self, password: &'ptr str) {
        self.CSPPasswordPtr = mq_str_ptr(password);
        self.CSPPasswordLength = password
            .len()
            .try_into()
            .expect("Password length should not exceed maximum positive MQLONG");
    }

    pub fn attach_userid(&mut self, userid: &'ptr str) {
        self.CSPUserIdPtr = mq_str_ptr(userid);
        self.CSPUserIdLength = userid
            .len()
            .try_into()
            .expect("User length should not exceed maximum positive MQLONG");
    }

    pub fn attach_token(&mut self, token: &'ptr str) {
        self.set_min_version(sys::MQCSP_VERSION_3);
        self.TokenPtr = mq_str_ptr(token);
        self.TokenLength = token
            .len()
            .try_into()
            .expect("Token length should not exceed maximum positive MQLONG");
    }

    pub fn attach_initial_key(&mut self, initial_key: &'ptr str) {
        self.set_min_version(sys::MQCSP_VERSION_2);
        self.InitialKeyPtr = mq_str_ptr(initial_key);
        self.InitialKeyLength = initial_key
            .len()
            .try_into()
            .expect("Initial key length should not exceed maximum positive MQLONG");
    }
}

// Functions to attach references to MQSCO
impl<'ptr> MqStruct<'ptr, sys::MQSCO> {
    pub fn attach_repo_password(&mut self, password: Option<&'ptr str>) {
        self.set_min_version(sys::MQSCO_VERSION_6);
        if let Some(ps) = password {
            self.KeyRepoPasswordPtr = mq_str_ptr(ps);
            self.KeyRepoPasswordLength = ps
                .len()
                .try_into()
                .expect("Password length should not exceed maximum positive MQLONG");
        } else {
            self.KeyRepoPasswordPtr = ptr::null_mut();
            self.KeyRepoPasswordLength = 0;
        }
    }

    pub fn attach_auth_info_records(&mut self, air: &'ptr [sys::MQAIR]) {
        self.AuthInfoRecPtr = air.as_ptr().cast_mut();
        self.AuthInfoRecCount = air
            .len()
            .try_into()
            .expect("Auth info record count should not exceed maximum positive MQLONG");
    }
}
