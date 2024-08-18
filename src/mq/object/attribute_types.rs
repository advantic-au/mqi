use crate::{sys, MqValue};

use super::attribute::AttributeType;

// Create a string based InqReqType
const fn inqreq_str(mqca: sys::MQLONG, length: usize) -> AttributeType {
    AttributeType {
        attribute: MqValue::from(mqca),
        #[allow(clippy::cast_possible_truncation)]
        text_len: length as u32,
    }
}

// Create a MQLONG based InqReqType
const fn inqreq_long(mqca: sys::MQLONG) -> AttributeType {
    AttributeType {
        attribute: MqValue::from(mqca),
        text_len: 0,
    }
}

// All MQIA_* and MQCA_* constants (excluding MQCA_NAMES) supported by MQINQ as documented at
// https://www.ibm.com/docs/en/ibm-mq/9.4?topic=calls-mqinq-inquire-object-attributes

// MQIA constants
pub const MQIA_ACCOUNTING_CONN_OVERRIDE: AttributeType = inqreq_long(sys::MQIA_ACCOUNTING_CONN_OVERRIDE);
pub const MQIA_ACCOUNTING_INTERVAL: AttributeType = inqreq_long(sys::MQIA_ACCOUNTING_INTERVAL);
pub const MQIA_ACCOUNTING_MQI: AttributeType = inqreq_long(sys::MQIA_ACCOUNTING_MQI);
pub const MQIA_ACCOUNTING_Q: AttributeType = inqreq_long(sys::MQIA_ACCOUNTING_Q);
pub const MQIA_ACTIVE_CHANNELS: AttributeType = inqreq_long(sys::MQIA_ACTIVE_CHANNELS);
pub const MQIA_ADOPTNEWMCA_CHECK: AttributeType = inqreq_long(sys::MQIA_ADOPTNEWMCA_CHECK);
pub const MQIA_ADOPTNEWMCA_INTERVAL: AttributeType = inqreq_long(sys::MQIA_ADOPTNEWMCA_INTERVAL);
pub const MQIA_ADOPTNEWMCA_TYPE: AttributeType = inqreq_long(sys::MQIA_ADOPTNEWMCA_TYPE);
pub const MQIA_APPL_TYPE: AttributeType = inqreq_long(sys::MQIA_APPL_TYPE);
pub const MQIA_AUTHORITY_EVENT: AttributeType = inqreq_long(sys::MQIA_AUTHORITY_EVENT);
pub const MQIA_BACKOUT_THRESHOLD: AttributeType = inqreq_long(sys::MQIA_BACKOUT_THRESHOLD);
pub const MQIA_BRIDGE_EVENT: AttributeType = inqreq_long(sys::MQIA_BRIDGE_EVENT);
pub const MQIA_CHANNEL_AUTO_DEF: AttributeType = inqreq_long(sys::MQIA_CHANNEL_AUTO_DEF);
pub const MQIA_CHANNEL_AUTO_DEF_EVENT: AttributeType = inqreq_long(sys::MQIA_CHANNEL_AUTO_DEF_EVENT);
pub const MQIA_CHANNEL_EVENT: AttributeType = inqreq_long(sys::MQIA_CHANNEL_EVENT);
pub const MQIA_CHINIT_ADAPTERS: AttributeType = inqreq_long(sys::MQIA_CHINIT_ADAPTERS);
pub const MQIA_CHINIT_DISPATCHERS: AttributeType = inqreq_long(sys::MQIA_CHINIT_DISPATCHERS);
pub const MQIA_CHINIT_TRACE_AUTO_START: AttributeType = inqreq_long(sys::MQIA_CHINIT_TRACE_AUTO_START);
pub const MQIA_CHINIT_TRACE_TABLE_SIZE: AttributeType = inqreq_long(sys::MQIA_CHINIT_TRACE_TABLE_SIZE);
pub const MQIA_CLUSTER_WORKLOAD_LENGTH: AttributeType = inqreq_long(sys::MQIA_CLUSTER_WORKLOAD_LENGTH);
pub const MQIA_CLWL_MRU_CHANNELS: AttributeType = inqreq_long(sys::MQIA_CLWL_MRU_CHANNELS);
pub const MQIA_CLWL_Q_PRIORITY: AttributeType = inqreq_long(sys::MQIA_CLWL_Q_PRIORITY);
pub const MQIA_CLWL_Q_RANK: AttributeType = inqreq_long(sys::MQIA_CLWL_Q_RANK);
pub const MQIA_CLWL_USEQ: AttributeType = inqreq_long(sys::MQIA_CLWL_USEQ);
pub const MQIA_CODED_CHAR_SET_ID: AttributeType = inqreq_long(sys::MQIA_CODED_CHAR_SET_ID);
pub const MQIA_COMMAND_EVENT: AttributeType = inqreq_long(sys::MQIA_COMMAND_EVENT);
pub const MQIA_COMMAND_LEVEL: AttributeType = inqreq_long(sys::MQIA_COMMAND_LEVEL);
pub const MQIA_CONFIGURATION_EVENT: AttributeType = inqreq_long(sys::MQIA_CONFIGURATION_EVENT);
pub const MQIA_CURRENT_Q_DEPTH: AttributeType = inqreq_long(sys::MQIA_CURRENT_Q_DEPTH);
pub const MQIA_DEF_BIND: AttributeType = inqreq_long(sys::MQIA_DEF_BIND);
pub const MQIA_DEF_CLUSTER_XMIT_Q_TYPE: AttributeType = inqreq_long(sys::MQIA_DEF_CLUSTER_XMIT_Q_TYPE);
pub const MQIA_DEF_INPUT_OPEN_OPTION: AttributeType = inqreq_long(sys::MQIA_DEF_INPUT_OPEN_OPTION);
pub const MQIA_DEF_PERSISTENCE: AttributeType = inqreq_long(sys::MQIA_DEF_PERSISTENCE);
pub const MQIA_DEF_PRIORITY: AttributeType = inqreq_long(sys::MQIA_DEF_PRIORITY);
pub const MQIA_DEFINITION_TYPE: AttributeType = inqreq_long(sys::MQIA_DEFINITION_TYPE);
pub const MQIA_DIST_LISTS: AttributeType = inqreq_long(sys::MQIA_DIST_LISTS);
pub const MQIA_DNS_WLM: AttributeType = inqreq_long(sys::MQIA_DNS_WLM);
pub const MQIA_EXPIRY_INTERVAL: AttributeType = inqreq_long(sys::MQIA_EXPIRY_INTERVAL);
pub const MQIA_GROUP_UR: AttributeType = inqreq_long(sys::MQIA_GROUP_UR);
pub const MQIA_HARDEN_GET_BACKOUT: AttributeType = inqreq_long(sys::MQIA_HARDEN_GET_BACKOUT);
pub const MQIA_IGQ_PUT_AUTHORITY: AttributeType = inqreq_long(sys::MQIA_IGQ_PUT_AUTHORITY);
pub const MQIA_INDEX_TYPE: AttributeType = inqreq_long(sys::MQIA_INDEX_TYPE);
pub const MQIA_INHIBIT_EVENT: AttributeType = inqreq_long(sys::MQIA_INHIBIT_EVENT);
pub const MQIA_INHIBIT_GET: AttributeType = inqreq_long(sys::MQIA_INHIBIT_GET);
pub const MQIA_INHIBIT_PUT: AttributeType = inqreq_long(sys::MQIA_INHIBIT_PUT);
pub const MQIA_INTRA_GROUP_QUEUING: AttributeType = inqreq_long(sys::MQIA_INTRA_GROUP_QUEUING);
pub const MQIA_LISTENER_TIMER: AttributeType = inqreq_long(sys::MQIA_LISTENER_TIMER);
pub const MQIA_LOCAL_EVENT: AttributeType = inqreq_long(sys::MQIA_LOCAL_EVENT);
pub const MQIA_LOGGER_EVENT: AttributeType = inqreq_long(sys::MQIA_LOGGER_EVENT);
pub const MQIA_LU62_CHANNELS: AttributeType = inqreq_long(sys::MQIA_LU62_CHANNELS);
pub const MQIA_MAX_CHANNELS: AttributeType = inqreq_long(sys::MQIA_MAX_CHANNELS);
pub const MQIA_MAX_HANDLES: AttributeType = inqreq_long(sys::MQIA_MAX_HANDLES);
pub const MQIA_MAX_MSG_LENGTH: AttributeType = inqreq_long(sys::MQIA_MAX_MSG_LENGTH);
pub const MQIA_MAX_PRIORITY: AttributeType = inqreq_long(sys::MQIA_MAX_PRIORITY);
pub const MQIA_MAX_Q_DEPTH: AttributeType = inqreq_long(sys::MQIA_MAX_Q_DEPTH);
pub const MQIA_MAX_UNCOMMITTED_MSGS: AttributeType = inqreq_long(sys::MQIA_MAX_UNCOMMITTED_MSGS);
pub const MQIA_MSG_DELIVERY_SEQUENCE: AttributeType = inqreq_long(sys::MQIA_MSG_DELIVERY_SEQUENCE);
pub const MQIA_MSG_MARK_BROWSE_INTERVAL: AttributeType = inqreq_long(sys::MQIA_MSG_MARK_BROWSE_INTERVAL);
pub const MQIA_NAME_COUNT: AttributeType = inqreq_long(sys::MQIA_NAME_COUNT);
pub const MQIA_NAMELIST_TYPE: AttributeType = inqreq_long(sys::MQIA_NAMELIST_TYPE);
pub const MQIA_NPM_CLASS: AttributeType = inqreq_long(sys::MQIA_NPM_CLASS);
pub const MQIA_OPEN_INPUT_COUNT: AttributeType = inqreq_long(sys::MQIA_OPEN_INPUT_COUNT);
pub const MQIA_OPEN_OUTPUT_COUNT: AttributeType = inqreq_long(sys::MQIA_OPEN_OUTPUT_COUNT);
pub const MQIA_OUTBOUND_PORT_MAX: AttributeType = inqreq_long(sys::MQIA_OUTBOUND_PORT_MAX);
pub const MQIA_OUTBOUND_PORT_MIN: AttributeType = inqreq_long(sys::MQIA_OUTBOUND_PORT_MIN);
pub const MQIA_PERFORMANCE_EVENT: AttributeType = inqreq_long(sys::MQIA_PERFORMANCE_EVENT);
pub const MQIA_PLATFORM: AttributeType = inqreq_long(sys::MQIA_PLATFORM);
pub const MQIA_PROPERTY_CONTROL: AttributeType = inqreq_long(sys::MQIA_PROPERTY_CONTROL);
pub const MQIA_PROT_POLICY_CAPABILITY: AttributeType = inqreq_long(sys::MQIA_PROT_POLICY_CAPABILITY);
pub const MQIA_PUBSUB_MAXMSG_RETRY_COUNT: AttributeType = inqreq_long(sys::MQIA_PUBSUB_MAXMSG_RETRY_COUNT);
pub const MQIA_PUBSUB_MODE: AttributeType = inqreq_long(sys::MQIA_PUBSUB_MODE);
pub const MQIA_PUBSUB_NP_MSG: AttributeType = inqreq_long(sys::MQIA_PUBSUB_NP_MSG);
pub const MQIA_PUBSUB_NP_RESP: AttributeType = inqreq_long(sys::MQIA_PUBSUB_NP_RESP);
pub const MQIA_PUBSUB_SYNC_PT: AttributeType = inqreq_long(sys::MQIA_PUBSUB_SYNC_PT);
pub const MQIA_Q_DEPTH_HIGH_EVENT: AttributeType = inqreq_long(sys::MQIA_Q_DEPTH_HIGH_EVENT);
pub const MQIA_Q_DEPTH_HIGH_LIMIT: AttributeType = inqreq_long(sys::MQIA_Q_DEPTH_HIGH_LIMIT);
pub const MQIA_Q_DEPTH_LOW_EVENT: AttributeType = inqreq_long(sys::MQIA_Q_DEPTH_LOW_EVENT);
pub const MQIA_Q_DEPTH_LOW_LIMIT: AttributeType = inqreq_long(sys::MQIA_Q_DEPTH_LOW_LIMIT);
pub const MQIA_Q_DEPTH_MAX_EVENT: AttributeType = inqreq_long(sys::MQIA_Q_DEPTH_MAX_EVENT);
pub const MQIA_Q_SERVICE_INTERVAL: AttributeType = inqreq_long(sys::MQIA_Q_SERVICE_INTERVAL);
pub const MQIA_Q_SERVICE_INTERVAL_EVENT: AttributeType = inqreq_long(sys::MQIA_Q_SERVICE_INTERVAL_EVENT);
pub const MQIA_Q_TYPE: AttributeType = inqreq_long(sys::MQIA_Q_TYPE);
pub const MQIA_QMGR_CFCONLOS: AttributeType = inqreq_long(sys::MQIA_QMGR_CFCONLOS);
pub const MQIA_QSG_DISP: AttributeType = inqreq_long(sys::MQIA_QSG_DISP);
pub const MQIA_RECEIVE_TIMEOUT: AttributeType = inqreq_long(sys::MQIA_RECEIVE_TIMEOUT);
pub const MQIA_RECEIVE_TIMEOUT_MIN: AttributeType = inqreq_long(sys::MQIA_RECEIVE_TIMEOUT_MIN);
pub const MQIA_RECEIVE_TIMEOUT_TYPE: AttributeType = inqreq_long(sys::MQIA_RECEIVE_TIMEOUT_TYPE);
pub const MQIA_REMOTE_EVENT: AttributeType = inqreq_long(sys::MQIA_REMOTE_EVENT);
pub const MQIA_RETENTION_INTERVAL: AttributeType = inqreq_long(sys::MQIA_RETENTION_INTERVAL);
pub const MQIA_SCOPE: AttributeType = inqreq_long(sys::MQIA_SCOPE);
pub const MQIA_SECURITY_CASE: AttributeType = inqreq_long(sys::MQIA_SECURITY_CASE);
pub const MQIA_SHAREABILITY: AttributeType = inqreq_long(sys::MQIA_SHAREABILITY);
pub const MQIA_SSL_EVENT: AttributeType = inqreq_long(sys::MQIA_SSL_EVENT);
pub const MQIA_SSL_FIPS_REQUIRED: AttributeType = inqreq_long(sys::MQIA_SSL_FIPS_REQUIRED);
pub const MQIA_SSL_RESET_COUNT: AttributeType = inqreq_long(sys::MQIA_SSL_RESET_COUNT);
pub const MQIA_START_STOP_EVENT: AttributeType = inqreq_long(sys::MQIA_START_STOP_EVENT);
pub const MQIA_STATISTICS_AUTO_CLUSSDR: AttributeType = inqreq_long(sys::MQIA_STATISTICS_AUTO_CLUSSDR);
pub const MQIA_STATISTICS_CHANNEL: AttributeType = inqreq_long(sys::MQIA_STATISTICS_CHANNEL);
pub const MQIA_STATISTICS_INTERVAL: AttributeType = inqreq_long(sys::MQIA_STATISTICS_INTERVAL);
pub const MQIA_STATISTICS_MQI: AttributeType = inqreq_long(sys::MQIA_STATISTICS_MQI);
pub const MQIA_STATISTICS_Q: AttributeType = inqreq_long(sys::MQIA_STATISTICS_Q);
pub const MQIA_SYNCPOINT: AttributeType = inqreq_long(sys::MQIA_SYNCPOINT);
pub const MQIA_TCP_CHANNELS: AttributeType = inqreq_long(sys::MQIA_TCP_CHANNELS);
pub const MQIA_TCP_KEEP_ALIVE: AttributeType = inqreq_long(sys::MQIA_TCP_KEEP_ALIVE);
pub const MQIA_TCP_STACK_TYPE: AttributeType = inqreq_long(sys::MQIA_TCP_STACK_TYPE);
pub const MQIA_TRACE_ROUTE_RECORDING: AttributeType = inqreq_long(sys::MQIA_TRACE_ROUTE_RECORDING);
pub const MQIA_TREE_LIFE_TIME: AttributeType = inqreq_long(sys::MQIA_TREE_LIFE_TIME);
pub const MQIA_TRIGGER_CONTROL: AttributeType = inqreq_long(sys::MQIA_TRIGGER_CONTROL);
pub const MQIA_TRIGGER_DEPTH: AttributeType = inqreq_long(sys::MQIA_TRIGGER_DEPTH);
pub const MQIA_TRIGGER_INTERVAL: AttributeType = inqreq_long(sys::MQIA_TRIGGER_INTERVAL);
pub const MQIA_TRIGGER_MSG_PRIORITY: AttributeType = inqreq_long(sys::MQIA_TRIGGER_MSG_PRIORITY);
pub const MQIA_TRIGGER_TYPE: AttributeType = inqreq_long(sys::MQIA_TRIGGER_TYPE);
pub const MQIA_USAGE: AttributeType = inqreq_long(sys::MQIA_USAGE);

// MQCA constants
pub const MQCA_ALTERATION_DATE: AttributeType = inqreq_str(sys::MQCA_ALTERATION_DATE, sys::MQ_DATE_LENGTH);
pub const MQCA_ALTERATION_TIME: AttributeType = inqreq_str(sys::MQCA_ALTERATION_TIME, sys::MQ_TIME_LENGTH);
pub const MQCA_APPL_ID: AttributeType = inqreq_str(sys::MQCA_APPL_ID, sys::MQ_PROCESS_APPL_ID_LENGTH);
pub const MQCA_BACKOUT_REQ_Q_NAME: AttributeType = inqreq_str(sys::MQCA_BACKOUT_REQ_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_BASE_Q_NAME: AttributeType = inqreq_str(sys::MQCA_BASE_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_CF_STRUC_NAME: AttributeType = inqreq_str(sys::MQCA_CF_STRUC_NAME, sys::MQ_CF_STRUC_NAME_LENGTH);
pub const MQCA_CHANNEL_AUTO_DEF_EXIT: AttributeType = inqreq_str(sys::MQCA_CHANNEL_AUTO_DEF_EXIT, sys::MQ_EXIT_NAME_LENGTH);
pub const MQCA_CLUS_CHL_NAME: AttributeType = inqreq_str(sys::MQCA_CLUS_CHL_NAME, sys::MQ_CHANNEL_NAME_LENGTH);
pub const MQCA_CLUSTER_NAME: AttributeType = inqreq_str(sys::MQCA_CLUSTER_NAME, sys::MQ_CLUSTER_NAME_LENGTH);
pub const MQCA_CLUSTER_NAMELIST: AttributeType = inqreq_str(sys::MQCA_CLUSTER_NAMELIST, sys::MQ_NAMELIST_NAME_LENGTH);
pub const MQCA_CLUSTER_WORKLOAD_DATA: AttributeType = inqreq_str(sys::MQCA_CLUSTER_WORKLOAD_DATA, sys::MQ_EXIT_DATA_LENGTH);
pub const MQCA_CLUSTER_WORKLOAD_EXIT: AttributeType = inqreq_str(sys::MQCA_CLUSTER_WORKLOAD_EXIT, sys::MQ_EXIT_NAME_LENGTH);
pub const MQCA_COMMAND_INPUT_Q_NAME: AttributeType = inqreq_str(sys::MQCA_COMMAND_INPUT_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_CREATION_DATE: AttributeType = inqreq_str(sys::MQCA_CREATION_DATE, sys::MQ_CREATION_DATE_LENGTH);
pub const MQCA_CREATION_TIME: AttributeType = inqreq_str(sys::MQCA_CREATION_TIME, sys::MQ_CREATION_TIME_LENGTH);
pub const MQCA_CUSTOM: AttributeType = inqreq_str(sys::MQCA_CUSTOM, sys::MQ_CUSTOM_LENGTH);
pub const MQCA_DEAD_LETTER_Q_NAME: AttributeType = inqreq_str(sys::MQCA_DEAD_LETTER_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_DEF_XMIT_Q_NAME: AttributeType = inqreq_str(sys::MQCA_DEF_XMIT_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_DNS_GROUP: AttributeType = inqreq_str(sys::MQCA_DNS_GROUP, sys::MQ_DNS_GROUP_NAME_LENGTH);
pub const MQCA_ENV_DATA: AttributeType = inqreq_str(sys::MQCA_ENV_DATA, sys::MQ_PROCESS_ENV_DATA_LENGTH);
pub const MQCA_IGQ_USER_ID: AttributeType = inqreq_str(sys::MQCA_IGQ_USER_ID, sys::MQ_USER_ID_LENGTH);
pub const MQCA_INITIAL_KEY: AttributeType = inqreq_str(sys::MQCA_INITIAL_KEY, sys::MQ_INITIAL_KEY_LENGTH);
pub const MQCA_INITIATION_Q_NAME: AttributeType = inqreq_str(sys::MQCA_INITIATION_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_INSTALLATION_DESC: AttributeType = inqreq_str(sys::MQCA_INSTALLATION_DESC, sys::MQ_INSTALLATION_DESC_LENGTH);
pub const MQCA_INSTALLATION_NAME: AttributeType = inqreq_str(sys::MQCA_INSTALLATION_NAME, sys::MQ_INSTALLATION_NAME_LENGTH);
pub const MQCA_INSTALLATION_PATH: AttributeType = inqreq_str(sys::MQCA_INSTALLATION_PATH, sys::MQ_INSTALLATION_PATH_LENGTH);
pub const MQCA_LU_GROUP_NAME: AttributeType = inqreq_str(sys::MQCA_LU_GROUP_NAME, sys::MQ_LU_NAME_LENGTH);
pub const MQCA_LU_NAME: AttributeType = inqreq_str(sys::MQCA_LU_NAME, sys::MQ_LU_NAME_LENGTH);
pub const MQCA_LU62_ARM_SUFFIX: AttributeType = inqreq_str(sys::MQCA_LU62_ARM_SUFFIX, sys::MQ_ARM_SUFFIX_LENGTH);
pub const MQCA_NAMELIST_DESC: AttributeType = inqreq_str(sys::MQCA_NAMELIST_DESC, sys::MQ_NAMELIST_DESC_LENGTH);
pub const MQCA_NAMELIST_NAME: AttributeType = inqreq_str(sys::MQCA_NAMELIST_NAME, sys::MQ_NAMELIST_NAME_LENGTH);
pub const MQCA_PARENT: AttributeType = inqreq_str(sys::MQCA_PARENT, sys::MQ_Q_MGR_NAME_LENGTH);
pub const MQCA_PROCESS_DESC: AttributeType = inqreq_str(sys::MQCA_PROCESS_DESC, sys::MQ_PROCESS_DESC_LENGTH);
pub const MQCA_PROCESS_NAME: AttributeType = inqreq_str(sys::MQCA_PROCESS_NAME, sys::MQ_PROCESS_NAME_LENGTH);
pub const MQCA_Q_DESC: AttributeType = inqreq_str(sys::MQCA_Q_DESC, sys::MQ_Q_DESC_LENGTH);
pub const MQCA_Q_MGR_DESC: AttributeType = inqreq_str(sys::MQCA_Q_MGR_DESC, sys::MQ_Q_MGR_DESC_LENGTH);
pub const MQCA_Q_MGR_IDENTIFIER: AttributeType = inqreq_str(sys::MQCA_Q_MGR_IDENTIFIER, sys::MQ_Q_MGR_IDENTIFIER_LENGTH);
pub const MQCA_Q_MGR_NAME: AttributeType = inqreq_str(sys::MQCA_Q_MGR_NAME, sys::MQ_Q_MGR_NAME_LENGTH);
pub const MQCA_Q_NAME: AttributeType = inqreq_str(sys::MQCA_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_QSG_NAME: AttributeType = inqreq_str(sys::MQCA_QSG_NAME, sys::MQ_QSG_NAME_LENGTH);
pub const MQCA_REMOTE_Q_MGR_NAME: AttributeType = inqreq_str(sys::MQCA_REMOTE_Q_MGR_NAME, sys::MQ_Q_MGR_NAME_LENGTH);
pub const MQCA_REMOTE_Q_NAME: AttributeType = inqreq_str(sys::MQCA_REMOTE_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_REPOSITORY_NAME: AttributeType = inqreq_str(sys::MQCA_REPOSITORY_NAME, sys::MQ_CLUSTER_NAME_LENGTH);
pub const MQCA_REPOSITORY_NAMELIST: AttributeType = inqreq_str(sys::MQCA_REPOSITORY_NAMELIST, sys::MQ_NAMELIST_NAME_LENGTH);
pub const MQCA_SSL_KEY_REPO_PASSWORD: AttributeType =
    inqreq_str(sys::MQCA_SSL_KEY_REPO_PASSWORD, sys::MQ_SSL_ENCRYP_KEY_REPO_PWD_LEN);
pub const MQCA_STORAGE_CLASS: AttributeType = inqreq_str(sys::MQCA_STORAGE_CLASS, sys::MQ_STORAGE_CLASS_LENGTH);
pub const MQCA_TCP_NAME: AttributeType = inqreq_str(sys::MQCA_TCP_NAME, sys::MQ_TCP_NAME_LENGTH);
pub const MQCA_TRIGGER_DATA: AttributeType = inqreq_str(sys::MQCA_TRIGGER_DATA, sys::MQ_TRIGGER_DATA_LENGTH);
pub const MQCA_USER_DATA: AttributeType = inqreq_str(sys::MQCA_USER_DATA, sys::MQ_PROCESS_USER_DATA_LENGTH);
pub const MQCA_XMIT_Q_NAME: AttributeType = inqreq_str(sys::MQCA_XMIT_Q_NAME, sys::MQ_Q_NAME_LENGTH);

// TODO: Add some further constants supported as per
// https://www.ibm.com/docs/en/ibm-mq/9.4?topic=formats-mqcmd-inquire-q-inquire-queue
// https://www.ibm.com/docs/en/ibm-mq/9.4?topic=formats-mqcmd-inquire-q-mgr-inquire-queue-manager
// ..etc
