#![expect(clippy::allow_attributes)]

use std::cmp;
use std::slice::from_raw_parts_mut;
use crate::core::Library;

use libmqm_sys::{Mqi, Mqai};
use libmqm_sys::lib as sys;

use crate::put::PutMessage;

mockall::mock! {
    pub Functions {}

    #[allow(non_snake_case)]
    impl Mqi for Functions {

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

    #[allow(non_snake_case)]
    #[cfg(feature = "mqai")]
    impl Mqai for Functions {
        unsafe fn mqCreateBag(
            &self,
            Options: sys::MQLONG,
            pBag: sys::PMQHBAG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqClearBag(&self, Bag: sys::MQHBAG, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG);

        unsafe fn mqDeleteBag(&self, pBag: sys::PMQHBAG, pCompCode: sys::PMQLONG, pReason: sys::PMQLONG);
        unsafe fn mqGetBag(
            &self,
            Hconn: sys::MQHCONN,
            Hobj: sys::MQHOBJ,
            pMsgDesc: sys::PMQVOID,
            pGetMsgOpts: sys::PMQVOID,
            Bag: sys::MQHBAG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqPutBag(
            &self,
            Hconn: sys::MQHCONN,
            Hobj: sys::MQHOBJ,
            pMsgDesc: sys::PMQVOID,
            pPutMsgOpts: sys::PMQVOID,
            Bag: sys::MQHBAG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqTruncateBag(
            &self,
            Bag: sys::MQHBAG,
            ItemCount: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddInquiry(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqDeleteItem(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddInteger(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemValue: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddIntegerFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemValue: sys::MQLONG,
            Operator: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddInteger64(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemValue: sys::MQINT64,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddString(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQCHAR,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddStringFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQCHAR,
            Operator: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddByteString(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQBYTE,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddByteStringFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQBYTE,
            Operator: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetInteger(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            ItemValue: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetIntegerFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            ItemValue: sys::MQLONG,
            Operator: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetInteger64(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            ItemValue: sys::MQINT64,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqAddBag(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemValue: sys::MQHBAG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetString(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQCHAR,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetStringFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQCHAR,
            Operator: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetByteString(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQBYTE,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqSetByteStringFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQBYTE,
            Operator: sys::MQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireInteger(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            pItemValue: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireIntegerFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            pItemValue: sys::PMQLONG,
            pOperator: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireInteger64(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            pItemValue: sys::PMQINT64,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireByteString(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQBYTE,
            pByteStringLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireString(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQCHAR,
            pStringLength: sys::PMQLONG,
            pCodedCharSetId: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireStringFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQCHAR,
            pStringLength: sys::PMQLONG,
            pCodedCharSetId: sys::PMQLONG,
            pOperator: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireByteStringFilter(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQBYTE,
            pByteStringLength: sys::PMQLONG,
            pOperator: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqInquireBag(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            pItemValue: sys::PMQHBAG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqCountItems(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            pItemCount: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqExecute(
            &self,
            Hconn: sys::MQHCONN,
            Command: sys::MQLONG,
            OptionsBag: sys::MQHBAG,
            AdminBag: sys::MQHBAG,
            ResponseBag: sys::MQHBAG,
            AdminQ: sys::MQHOBJ,
            ResponseQ: sys::MQHOBJ,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );

        unsafe fn mqBagToBuffer(
            &self,
            OptionsBag: sys::MQHBAG,
            DataBag: sys::MQHBAG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            pDataLength: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );
    
        unsafe fn mqBufferToBag(
            &self,
            OptionsBag: sys::MQHBAG,
            BufferLength: sys::MQLONG,
            pBuffer: sys::PMQVOID,
            DataBag: sys::MQHBAG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );
    
        unsafe fn mqInquireItemInfo(
            &self,
            Bag: sys::MQHBAG,
            Selector: sys::MQLONG,
            ItemIndex: sys::MQLONG,
            pOutSelector: sys::PMQLONG,
            pItemType: sys::PMQLONG,
            pCompCode: sys::PMQLONG,
            pReason: sys::PMQLONG,
        );
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
        self.expect_MQCRTMH()
            .returning(move |_, _, hmsg, comp_code, reason| {
                unsafe {
                    *hmsg = hMsg;
                }
                Self::mqi_outcome_ok(comp_code, reason);
            })
            .times(count)
            .in_sequence(seq);

        self.expect_MQDLTMH()
            .withf(move |_, &msg, _, _, _| unsafe { *msg } == hMsg)
            .returning(|_, _, _, comp_code, reason| {
                Self::mqi_outcome_ok(comp_code, reason);
            });
    }

    pub fn get_error(&mut self, mqrc: sys::MQLONG, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
        self.expect_MQGET()
            .returning(move |_, _, _, _, _, _, _, cc, rc| Self::mqi_outcome(cc, rc, sys::MQCC_FAILED, mqrc))
            .times(count)
            .in_sequence(seq);
    }

    pub fn get_ok(
        &mut self,
        message: &'static (impl PutMessage + ?Sized),
        count: impl Into<mockall::TimesRange>,
        seq: &mut mockall::Sequence,
    ) {
        self.expect_MQGET()
            .returning_st(move |_, _, mqmd, _, buffer_len, buffer, out_length, cc, rc| {
                let md: &mut sys::MQMD = unsafe { &mut *(mqmd.cast()) };
                md.Format = *unsafe { &*std::ptr::from_ref(message.format().fmt.into_ascii().as_ref()).cast() };
                md.Encoding = message.format().encoding.0;

                let buf: &mut [u8] = unsafe {
                    from_raw_parts_mut(
                        buffer.cast(),
                        buffer_len.try_into().expect("buffer length to convert from i32 to usize"),
                    )
                };
                let msg = message.render();
                let len = cmp::min(buf.len(), msg.len());
                buf[..len].copy_from_slice(&msg[..len]);
                let copy_len = len.try_into().expect("buffer length to convert from usize to i32");
                unsafe { *out_length = copy_len };
                Self::mqi_outcome(
                    cc,
                    rc,
                    sys::MQCC_OK,
                    if copy_len < buffer_len {
                        sys::MQRC_TRUNCATED_MSG_ACCEPTED
                    } else {
                        sys::MQRC_NONE
                    },
                );
            })
            .times(count)
            .in_sequence(seq);
    }

    pub fn open_ok(&mut self, hObj: sys::MQHOBJ, count: impl Into<mockall::TimesRange>, seq: &mut mockall::Sequence) {
        self.expect_MQOPEN()
            .returning(move |_, _, _, hobj, comp_code, reason| {
                unsafe {
                    *hobj = hObj;
                }
                Self::mqi_outcome_ok(comp_code, reason);
            })
            .times(count)
            .in_sequence(seq);

        self.close_ok(hObj);
    }

    fn close_ok(&mut self, hObj: sys::MQHOBJ) {
        self.expect_MQCLOSE()
            .withf(move |_, &obj, _, _, _| unsafe { *obj } == hObj)
            .returning(|_, obj, _, comp_code, reason| {
                unsafe { *obj = sys::MQHO_UNUSABLE_HOBJ }
                Self::mqi_outcome_ok(comp_code, reason);
            });
    }

    pub fn subscribe_managed_ok(
        &mut self,
        hObj: sys::MQHOBJ,
        hSub: sys::MQHOBJ,
        count: impl Into<mockall::TimesRange>,
        seq: &mut mockall::Sequence,
    ) {
        self.expect_MQSUB()
            .withf(|_, mqsd, obj, sub, _, _| {
                let sd: &sys::MQSD = unsafe { &*(mqsd.cast()) };
                sd.Options & sys::MQSO_MANAGED != 0 && !obj.is_null() && !sub.is_null()
            })
            .returning(move |_, _, obj, sub, cc, rc| {
                unsafe {
                    *obj = hObj;
                    *sub = hSub;
                };
                Self::mqi_outcome_ok(cc, rc);
            })
            .times(count)
            .in_sequence(seq);

        self.close_ok(hObj);
        self.close_ok(hSub);
    }
}

#[cfg(feature = "mqai")]
mod mqai {
    use libmqm_sys::Mqai;

    use crate::core::Library;

    impl super::MockFunctions {
        pub fn real_bag(&mut self, mqai: impl Library<MQ: Mqai> + Clone + Send + 'static) {
            unsafe {
                // TODO: Add more bag function passthroughs
                let mq = mqai.clone();
                self.expect_mqCreateBag()
                    .returning(move |option, bag, cc, rc| mq.lib().mqCreateBag(option, bag, cc, rc));
                let mq = mqai.clone();
                self.expect_mqDeleteBag()
                    .returning(move |bag, cc, rc| mq.lib().mqDeleteBag(bag, cc, rc));
                let mq = mqai.clone();
                self.expect_mqSetInteger()
                    .returning(move |a, b, c, d, e, f| mq.lib().mqSetInteger(a, b, c, d, e, f));
                let mq = mqai.clone();
                self.expect_mqAddInteger()
                    .returning(move |a, b, c, d, e| mq.lib().mqAddInteger(a, b, c, d, e));
                let mq = mqai.clone();
                self.expect_mqInquireInteger()
                    .returning(move |a, b, c, d, e, f| mq.lib().mqInquireInteger(a, b, c, d, e, f));
                let mq = mqai;
                self.expect_mqAddString()
                    .returning(move |a, b, c, d, e, f| mq.lib().mqAddString(a, b, c, d, e, f));
            }
        }
    }
}

impl Library for MockFunctions {
    type MQ = Self;

    fn lib(&self) -> &Self::MQ {
        self
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
