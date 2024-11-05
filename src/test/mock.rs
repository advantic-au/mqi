#![expect(clippy::allow_attributes)]

use libmqm_sys::function;
use libmqm_sys::lib as sys;

mockall::mock! {
    pub Functions {}

    #[allow(non_snake_case)]
    impl function::Mqi for Functions {
        unsafe fn MQCONNX(
            &self,
            pQMgrName: sys::PMQCHAR,
            pConnectOpts: sys::PMQCNO,
            pHconn: sys::PMQHCONN,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQCONN(
            &self,
            pQMgrName: sys::PMQCHAR,
            pHconn: sys::PMQHCONN,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQDISC(&self, pHconn: sys::PMQHCONN, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG);

        unsafe fn MQOPEN(
            &self,
            Hconn: sys::MQHCONN,
            pObjDesc: sys::PMQVOID,
            Options: sys::MQLONG,
            pHobj: sys::PMQHOBJ,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQPUT1(
            &self,
            Hconn: sys::MQHCONN,
            pObjDesc: sys::PMQVOID,
            pMsgDesc: sys::PMQVOID,
            pPutMsgOpts: sys::PMQVOID,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQCLOSE(
            &self,
            Hconn: sys::MQHCONN,
            pHobj: sys::PMQHOBJ,
            Options: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQCMIT(&self, Hconn: sys::MQHCONN, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG);

        unsafe fn MQGET(
            &self,
            Hconn: sys::MQHCONN,
            Hobj: sys::MQHOBJ,
            pMsgDesc: sys::PMQVOID,
            pGetMsgOpts: sys::PMQVOID,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            pDataLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQPUT(
            &self,
            Hconn: sys::MQHCONN,
            Hobj: sys::MQHOBJ,
            pMsgDesc: sys::PMQVOID,
            pPutMsgOpts: sys::PMQVOID,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQINQ(
            &self,
            Hconn: sys::MQHCONN,
            Hobj: sys::MQHOBJ,
            SelectorCount: sys::MQLONG,
            pSelectors: sys::PMQLONG,
            IntAttrCount: sys::MQLONG,
            pIntAttrs: sys::PMQLONG,
            CharAttrLength: sys::MQLONG,
            pCharAttrs: sys::PMQCHAR,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQSUB(
            &self,
            Hconn: sys::MQHCONN,
            pSubDesc: sys::PMQVOID,
            pHobj: sys::PMQHOBJ,
            pHsub: sys::PMQHOBJ,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQSUBRQ(
            &self,
            Hconn: sys::MQHCONN,
            Hsub: sys::MQHOBJ,
            Action: sys::MQLONG,
            pSubRqOpts: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQBEGIN(
            &self,
            Hconn: sys::MQHCONN,
            pBeginOptions: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQBACK(&self, Hconn: sys::MQHCONN, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG);

        unsafe fn MQCRTMH(
            &self,
            Hconn: sys::MQHCONN,
            pCrtMsgHOpts: sys::PMQVOID,
            pHmsg: sys::PMQHMSG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQDLTMH(
            &self,
            Hconn: sys::MQHCONN,
            pHmsg: sys::PMQHMSG,
            pDltMsgHOpts: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQMHBUF(
            &self,
            Hconn: sys::MQHCONN,
            Hmsg: sys::MQHMSG,
            pMsgHBufOpts: sys::PMQVOID,
            pName: sys::PMQVOID,
            pMsgDesc: sys::PMQVOID,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            pDataLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQBUFMH(
            &self,
            Hconn: sys::MQHCONN,
            Hmsg: sys::MQHMSG,
            pBufMsgHOpts: sys::PMQVOID,
            pMsgDesc: sys::PMQVOID,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            pDataLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn MQCB(
            &self,
            Hconn: sys::MQHCONN,
            Operation: sys::MQLONG,
            pCallbackDesc: sys::PMQVOID,
            Hobj: sys::MQHOBJ,
            pMsgDesc: sys::PMQVOID,
            pGetMsgOpts: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQCTL(
            &self,
            Hconn: sys::MQHCONN,
            Operation: sys::MQLONG,
            pControlOpts: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQSET(
            &self,
            Hconn: sys::MQHCONN,
            Hobj: sys::MQHOBJ,
            SelectorCount: sys::MQLONG,
            pSelectors: sys::PMQLONG,
            IntAttrCount: sys::MQLONG,
            pIntAttrs: sys::PMQLONG,
            CharAttrLength: sys::MQLONG,
            pCharAttrs: sys::PMQCHAR,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQSETMP(
            &self,
            Hconn: sys::MQHCONN,
            Hmsg: sys::MQHMSG,
            pSetPropOpts: sys::PMQVOID,
            pName: sys::PMQVOID,
            pPropDesc: sys::PMQVOID,
            Type: sys::MQLONG,
            ValueLength: sys::MQLONG,
            pValue: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQSTAT(
            &self,
            Hconn: sys::MQHCONN,
            Type: sys::MQLONG,
            pStatus: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQINQMP(
            &self,
            Hconn: sys::MQHCONN,
            Hmsg: sys::MQHMSG,
            pInqPropOpts: sys::PMQVOID,
            pName: sys::PMQVOID,
            pPropDesc: sys::PMQVOID,
            pType: sys::PMQLONG,
            ValueLength: sys::MQLONG,
            pValue: sys::PMQVOID,
            pDataLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQDLTMP(
            &self,
            Hconn: sys::MQHCONN,
            Hmsg: sys::MQHMSG,
            pDltPropOpts: sys::PMQVOID,
            pName: sys::PMQVOID,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;

        unsafe fn MQXCNVC(
            &self,
            Hconn: sys::MQHCONN,
            Options: sys::MQLONG,
            SourceCCSID: sys::MQLONG,
            SourceLength: sys::MQLONG,
            pSourceBuffer: sys::PMQCHAR,
            TargetCCSID: sys::MQLONG,
            TargetLength: sys::MQLONG,
            pTargetBuffer: sys::PMQCHAR,
            pDataLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        ) ;
    }
}

#[allow(dead_code)]
#[expect(non_snake_case)]
impl MockFunctions {
    pub fn connx_outcome(&mut self, hconn: sys::MQHCONN, comp_code: sys::MQLONG, reason: sys::MQLONG) {
        self.expect_MQCONNX().returning(
            move |_, _, pHconn: sys::PMQHCONN, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG| {
                unsafe {
                    *pHconn = hconn;
                }
                Self::mqi_outcome(pCompCode, pReason, comp_code, reason);
            },
        );
    }

    pub fn disc_outcome(&mut self, comp_code: sys::MQLONG, reason: sys::MQLONG) {
        self.expect_MQDISC()
            .returning(move |_, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG| {
                Self::mqi_outcome(pCompCode, pReason, comp_code, reason);
            });
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn mqi_outcome(pCompCode: sys::PMQLONG, pReason: sys::PMQLONG, comp_code: sys::MQLONG, reason: sys::MQLONG) {
        unsafe {
            *pCompCode = comp_code;
            *pReason = reason;
        }
    }

    pub fn mqi_outcome_ok(pCompCode: sys::PMQLONG, pReason: sys::PMQLONG) {
        Self::mqi_outcome(pCompCode, pReason, sys::MQCC_OK, sys::MQRC_NONE);
    }

    pub fn properties_ok(&mut self, hMsg: sys::MQHMSG, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
        self.expect_MQCRTMH().returning(move |_, _, hmsg, comp_code, reason| {
            unsafe {
                *hmsg = hMsg;
            }
            Self::mqi_outcome_ok(comp_code, reason);
        })
        .times(count)
        .in_sequence(seq);
    
        self.expect_MQDLTMH().returning(|_, _, _, comp_code, reason| {
            Self::mqi_outcome_ok(comp_code, reason);
        });
    }

    pub fn open_ok(&mut self, hObj: sys::MQHOBJ, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
        self.expect_MQOPEN().returning(move |_, _, _, hobj, comp_code, reason| {
            unsafe {
                *hobj = hObj;
            }
            Self::mqi_outcome_ok(comp_code, reason);
        })
        .times(count)
        .in_sequence(seq);

        self.expect_MQCLOSE().returning(|_, _, _, comp_code, reason| {
            Self::mqi_outcome_ok(comp_code, reason);
        });
    }
    
}

#[must_use]
#[allow(dead_code)]
pub fn connect_ok() -> MockFunctions {
    let mut mock = MockFunctions::new();
    mock.connx_outcome(0x0d0d, sys::MQCC_OK, sys::MQRC_NONE);
    mock.disc_outcome(sys::MQCC_OK, sys::MQRC_NONE);
    mock
}
