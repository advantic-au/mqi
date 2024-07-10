use super::{InqReqItem, InqReqType};
use crate::{sys, MqValue};

// Create a string based InqReqType
const fn inqreq_str(mqca: sys::MQLONG, length: usize) -> InqReqType {
    (MqValue::from(mqca), InqReqItem::Str(length))
}

// Create a MQLONG based InqReqType
const fn inqreq_long(mqca: sys::MQLONG) -> InqReqType {
    (MqValue::from(mqca), InqReqItem::Long)
}

// All MQIA_* and MQCA_* constants (excluding MQCA_NAMES) supported by MQINQ as documented at
// https://www.ibm.com/docs/en/ibm-mq/9.4?topic=calls-mqinq-inquire-object-attributes

// MQIA constants
pub const MQIA_ACCOUNTING_CONN_OVERRIDE: InqReqType = inqreq_long(sys::MQIA_ACCOUNTING_CONN_OVERRIDE);
pub const MQIA_ACCOUNTING_INTERVAL: InqReqType = inqreq_long(sys::MQIA_ACCOUNTING_INTERVAL);
pub const MQIA_ACCOUNTING_MQI: InqReqType = inqreq_long(sys::MQIA_ACCOUNTING_MQI);
pub const MQIA_ACCOUNTING_Q: InqReqType = inqreq_long(sys::MQIA_ACCOUNTING_Q);
pub const MQIA_ACTIVE_CHANNELS: InqReqType = inqreq_long(sys::MQIA_ACTIVE_CHANNELS);
pub const MQIA_ADOPTNEWMCA_CHECK: InqReqType = inqreq_long(sys::MQIA_ADOPTNEWMCA_CHECK);
pub const MQIA_ADOPTNEWMCA_INTERVAL: InqReqType = inqreq_long(sys::MQIA_ADOPTNEWMCA_INTERVAL);
pub const MQIA_ADOPTNEWMCA_TYPE: InqReqType = inqreq_long(sys::MQIA_ADOPTNEWMCA_TYPE);
pub const MQIA_APPL_TYPE: InqReqType = inqreq_long(sys::MQIA_APPL_TYPE);
pub const MQIA_AUTHORITY_EVENT: InqReqType = inqreq_long(sys::MQIA_AUTHORITY_EVENT);
pub const MQIA_BACKOUT_THRESHOLD: InqReqType = inqreq_long(sys::MQIA_BACKOUT_THRESHOLD);
pub const MQIA_BRIDGE_EVENT: InqReqType = inqreq_long(sys::MQIA_BRIDGE_EVENT);
pub const MQIA_CHANNEL_AUTO_DEF: InqReqType = inqreq_long(sys::MQIA_CHANNEL_AUTO_DEF);
pub const MQIA_CHANNEL_AUTO_DEF_EVENT: InqReqType = inqreq_long(sys::MQIA_CHANNEL_AUTO_DEF_EVENT);
pub const MQIA_CHANNEL_EVENT: InqReqType = inqreq_long(sys::MQIA_CHANNEL_EVENT);
pub const MQIA_CHINIT_ADAPTERS: InqReqType = inqreq_long(sys::MQIA_CHINIT_ADAPTERS);
pub const MQIA_CHINIT_DISPATCHERS: InqReqType = inqreq_long(sys::MQIA_CHINIT_DISPATCHERS);
pub const MQIA_CHINIT_TRACE_AUTO_START: InqReqType = inqreq_long(sys::MQIA_CHINIT_TRACE_AUTO_START);
pub const MQIA_CHINIT_TRACE_TABLE_SIZE: InqReqType = inqreq_long(sys::MQIA_CHINIT_TRACE_TABLE_SIZE);
pub const MQIA_CLUSTER_WORKLOAD_LENGTH: InqReqType = inqreq_long(sys::MQIA_CLUSTER_WORKLOAD_LENGTH);
pub const MQIA_CLWL_MRU_CHANNELS: InqReqType = inqreq_long(sys::MQIA_CLWL_MRU_CHANNELS);
pub const MQIA_CLWL_Q_PRIORITY: InqReqType = inqreq_long(sys::MQIA_CLWL_Q_PRIORITY);
pub const MQIA_CLWL_Q_RANK: InqReqType = inqreq_long(sys::MQIA_CLWL_Q_RANK);
pub const MQIA_CLWL_USEQ: InqReqType = inqreq_long(sys::MQIA_CLWL_USEQ);
pub const MQIA_CODED_CHAR_SET_ID: InqReqType = inqreq_long(sys::MQIA_CODED_CHAR_SET_ID);
pub const MQIA_COMMAND_EVENT: InqReqType = inqreq_long(sys::MQIA_COMMAND_EVENT);
pub const MQIA_COMMAND_LEVEL: InqReqType = inqreq_long(sys::MQIA_COMMAND_LEVEL);
pub const MQIA_CONFIGURATION_EVENT: InqReqType = inqreq_long(sys::MQIA_CONFIGURATION_EVENT);
pub const MQIA_CURRENT_Q_DEPTH: InqReqType = inqreq_long(sys::MQIA_CURRENT_Q_DEPTH);
pub const MQIA_DEF_BIND: InqReqType = inqreq_long(sys::MQIA_DEF_BIND);
pub const MQIA_DEF_CLUSTER_XMIT_Q_TYPE: InqReqType = inqreq_long(sys::MQIA_DEF_CLUSTER_XMIT_Q_TYPE);
pub const MQIA_DEF_INPUT_OPEN_OPTION: InqReqType = inqreq_long(sys::MQIA_DEF_INPUT_OPEN_OPTION);
pub const MQIA_DEF_PERSISTENCE: InqReqType = inqreq_long(sys::MQIA_DEF_PERSISTENCE);
pub const MQIA_DEF_PRIORITY: InqReqType = inqreq_long(sys::MQIA_DEF_PRIORITY);
pub const MQIA_DEFINITION_TYPE: InqReqType = inqreq_long(sys::MQIA_DEFINITION_TYPE);
pub const MQIA_DIST_LISTS: InqReqType = inqreq_long(sys::MQIA_DIST_LISTS);
pub const MQIA_DNS_WLM: InqReqType = inqreq_long(sys::MQIA_DNS_WLM);
pub const MQIA_EXPIRY_INTERVAL: InqReqType = inqreq_long(sys::MQIA_EXPIRY_INTERVAL);
pub const MQIA_GROUP_UR: InqReqType = inqreq_long(sys::MQIA_GROUP_UR);
pub const MQIA_HARDEN_GET_BACKOUT: InqReqType = inqreq_long(sys::MQIA_HARDEN_GET_BACKOUT);
pub const MQIA_IGQ_PUT_AUTHORITY: InqReqType = inqreq_long(sys::MQIA_IGQ_PUT_AUTHORITY);
pub const MQIA_INDEX_TYPE: InqReqType = inqreq_long(sys::MQIA_INDEX_TYPE);
pub const MQIA_INHIBIT_EVENT: InqReqType = inqreq_long(sys::MQIA_INHIBIT_EVENT);
pub const MQIA_INHIBIT_GET: InqReqType = inqreq_long(sys::MQIA_INHIBIT_GET);
pub const MQIA_INHIBIT_PUT: InqReqType = inqreq_long(sys::MQIA_INHIBIT_PUT);
pub const MQIA_INTRA_GROUP_QUEUING: InqReqType = inqreq_long(sys::MQIA_INTRA_GROUP_QUEUING);
pub const MQIA_LISTENER_TIMER: InqReqType = inqreq_long(sys::MQIA_LISTENER_TIMER);
pub const MQIA_LOCAL_EVENT: InqReqType = inqreq_long(sys::MQIA_LOCAL_EVENT);
pub const MQIA_LOGGER_EVENT: InqReqType = inqreq_long(sys::MQIA_LOGGER_EVENT);
pub const MQIA_LU62_CHANNELS: InqReqType = inqreq_long(sys::MQIA_LU62_CHANNELS);
pub const MQIA_MAX_CHANNELS: InqReqType = inqreq_long(sys::MQIA_MAX_CHANNELS);
pub const MQIA_MAX_HANDLES: InqReqType = inqreq_long(sys::MQIA_MAX_HANDLES);
pub const MQIA_MAX_MSG_LENGTH: InqReqType = inqreq_long(sys::MQIA_MAX_MSG_LENGTH);
pub const MQIA_MAX_PRIORITY: InqReqType = inqreq_long(sys::MQIA_MAX_PRIORITY);
pub const MQIA_MAX_Q_DEPTH: InqReqType = inqreq_long(sys::MQIA_MAX_Q_DEPTH);
pub const MQIA_MAX_UNCOMMITTED_MSGS: InqReqType = inqreq_long(sys::MQIA_MAX_UNCOMMITTED_MSGS);
pub const MQIA_MSG_DELIVERY_SEQUENCE: InqReqType = inqreq_long(sys::MQIA_MSG_DELIVERY_SEQUENCE);
pub const MQIA_MSG_MARK_BROWSE_INTERVAL: InqReqType = inqreq_long(sys::MQIA_MSG_MARK_BROWSE_INTERVAL);
pub const MQIA_NAME_COUNT: InqReqType = inqreq_long(sys::MQIA_NAME_COUNT);
pub const MQIA_NAMELIST_TYPE: InqReqType = inqreq_long(sys::MQIA_NAMELIST_TYPE);
pub const MQIA_NPM_CLASS: InqReqType = inqreq_long(sys::MQIA_NPM_CLASS);
pub const MQIA_OPEN_INPUT_COUNT: InqReqType = inqreq_long(sys::MQIA_OPEN_INPUT_COUNT);
pub const MQIA_OPEN_OUTPUT_COUNT: InqReqType = inqreq_long(sys::MQIA_OPEN_OUTPUT_COUNT);
pub const MQIA_OUTBOUND_PORT_MAX: InqReqType = inqreq_long(sys::MQIA_OUTBOUND_PORT_MAX);
pub const MQIA_OUTBOUND_PORT_MIN: InqReqType = inqreq_long(sys::MQIA_OUTBOUND_PORT_MIN);
pub const MQIA_PERFORMANCE_EVENT: InqReqType = inqreq_long(sys::MQIA_PERFORMANCE_EVENT);
pub const MQIA_PLATFORM: InqReqType = inqreq_long(sys::MQIA_PLATFORM);
pub const MQIA_PROPERTY_CONTROL: InqReqType = inqreq_long(sys::MQIA_PROPERTY_CONTROL);
pub const MQIA_PROT_POLICY_CAPABILITY: InqReqType = inqreq_long(sys::MQIA_PROT_POLICY_CAPABILITY);
pub const MQIA_PUBSUB_MAXMSG_RETRY_COUNT: InqReqType = inqreq_long(sys::MQIA_PUBSUB_MAXMSG_RETRY_COUNT);
pub const MQIA_PUBSUB_MODE: InqReqType = inqreq_long(sys::MQIA_PUBSUB_MODE);
pub const MQIA_PUBSUB_NP_MSG: InqReqType = inqreq_long(sys::MQIA_PUBSUB_NP_MSG);
pub const MQIA_PUBSUB_NP_RESP: InqReqType = inqreq_long(sys::MQIA_PUBSUB_NP_RESP);
pub const MQIA_PUBSUB_SYNC_PT: InqReqType = inqreq_long(sys::MQIA_PUBSUB_SYNC_PT);
pub const MQIA_Q_DEPTH_HIGH_EVENT: InqReqType = inqreq_long(sys::MQIA_Q_DEPTH_HIGH_EVENT);
pub const MQIA_Q_DEPTH_HIGH_LIMIT: InqReqType = inqreq_long(sys::MQIA_Q_DEPTH_HIGH_LIMIT);
pub const MQIA_Q_DEPTH_LOW_EVENT: InqReqType = inqreq_long(sys::MQIA_Q_DEPTH_LOW_EVENT);
pub const MQIA_Q_DEPTH_LOW_LIMIT: InqReqType = inqreq_long(sys::MQIA_Q_DEPTH_LOW_LIMIT);
pub const MQIA_Q_DEPTH_MAX_EVENT: InqReqType = inqreq_long(sys::MQIA_Q_DEPTH_MAX_EVENT);
pub const MQIA_Q_SERVICE_INTERVAL: InqReqType = inqreq_long(sys::MQIA_Q_SERVICE_INTERVAL);
pub const MQIA_Q_SERVICE_INTERVAL_EVENT: InqReqType = inqreq_long(sys::MQIA_Q_SERVICE_INTERVAL_EVENT);
pub const MQIA_Q_TYPE: InqReqType = inqreq_long(sys::MQIA_Q_TYPE);
pub const MQIA_QMGR_CFCONLOS: InqReqType = inqreq_long(sys::MQIA_QMGR_CFCONLOS);
pub const MQIA_QSG_DISP: InqReqType = inqreq_long(sys::MQIA_QSG_DISP);
pub const MQIA_RECEIVE_TIMEOUT: InqReqType = inqreq_long(sys::MQIA_RECEIVE_TIMEOUT);
pub const MQIA_RECEIVE_TIMEOUT_MIN: InqReqType = inqreq_long(sys::MQIA_RECEIVE_TIMEOUT_MIN);
pub const MQIA_RECEIVE_TIMEOUT_TYPE: InqReqType = inqreq_long(sys::MQIA_RECEIVE_TIMEOUT_TYPE);
pub const MQIA_REMOTE_EVENT: InqReqType = inqreq_long(sys::MQIA_REMOTE_EVENT);
pub const MQIA_RETENTION_INTERVAL: InqReqType = inqreq_long(sys::MQIA_RETENTION_INTERVAL);
pub const MQIA_SCOPE: InqReqType = inqreq_long(sys::MQIA_SCOPE);
pub const MQIA_SECURITY_CASE: InqReqType = inqreq_long(sys::MQIA_SECURITY_CASE);
pub const MQIA_SHAREABILITY: InqReqType = inqreq_long(sys::MQIA_SHAREABILITY);
pub const MQIA_SSL_EVENT: InqReqType = inqreq_long(sys::MQIA_SSL_EVENT);
pub const MQIA_SSL_FIPS_REQUIRED: InqReqType = inqreq_long(sys::MQIA_SSL_FIPS_REQUIRED);
pub const MQIA_SSL_RESET_COUNT: InqReqType = inqreq_long(sys::MQIA_SSL_RESET_COUNT);
pub const MQIA_START_STOP_EVENT: InqReqType = inqreq_long(sys::MQIA_START_STOP_EVENT);
pub const MQIA_STATISTICS_AUTO_CLUSSDR: InqReqType = inqreq_long(sys::MQIA_STATISTICS_AUTO_CLUSSDR);
pub const MQIA_STATISTICS_CHANNEL: InqReqType = inqreq_long(sys::MQIA_STATISTICS_CHANNEL);
pub const MQIA_STATISTICS_INTERVAL: InqReqType = inqreq_long(sys::MQIA_STATISTICS_INTERVAL);
pub const MQIA_STATISTICS_MQI: InqReqType = inqreq_long(sys::MQIA_STATISTICS_MQI);
pub const MQIA_STATISTICS_Q: InqReqType = inqreq_long(sys::MQIA_STATISTICS_Q);
pub const MQIA_SYNCPOINT: InqReqType = inqreq_long(sys::MQIA_SYNCPOINT);
pub const MQIA_TCP_CHANNELS: InqReqType = inqreq_long(sys::MQIA_TCP_CHANNELS);
pub const MQIA_TCP_KEEP_ALIVE: InqReqType = inqreq_long(sys::MQIA_TCP_KEEP_ALIVE);
pub const MQIA_TCP_STACK_TYPE: InqReqType = inqreq_long(sys::MQIA_TCP_STACK_TYPE);
pub const MQIA_TRACE_ROUTE_RECORDING: InqReqType = inqreq_long(sys::MQIA_TRACE_ROUTE_RECORDING);
pub const MQIA_TREE_LIFE_TIME: InqReqType = inqreq_long(sys::MQIA_TREE_LIFE_TIME);
pub const MQIA_TRIGGER_CONTROL: InqReqType = inqreq_long(sys::MQIA_TRIGGER_CONTROL);
pub const MQIA_TRIGGER_DEPTH: InqReqType = inqreq_long(sys::MQIA_TRIGGER_DEPTH);
pub const MQIA_TRIGGER_INTERVAL: InqReqType = inqreq_long(sys::MQIA_TRIGGER_INTERVAL);
pub const MQIA_TRIGGER_MSG_PRIORITY: InqReqType = inqreq_long(sys::MQIA_TRIGGER_MSG_PRIORITY);
pub const MQIA_TRIGGER_TYPE: InqReqType = inqreq_long(sys::MQIA_TRIGGER_TYPE);
pub const MQIA_USAGE: InqReqType = inqreq_long(sys::MQIA_USAGE);

// MQCA constants
pub const MQCA_ALTERATION_DATE: InqReqType = inqreq_str(sys::MQCA_ALTERATION_DATE, sys::MQ_DATE_LENGTH);
pub const MQCA_ALTERATION_TIME: InqReqType = inqreq_str(sys::MQCA_ALTERATION_TIME, sys::MQ_TIME_LENGTH);
pub const MQCA_APPL_ID: InqReqType = inqreq_str(sys::MQCA_APPL_ID, sys::MQ_PROCESS_APPL_ID_LENGTH);
pub const MQCA_BACKOUT_REQ_Q_NAME: InqReqType = inqreq_str(sys::MQCA_BACKOUT_REQ_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_BASE_Q_NAME: InqReqType = inqreq_str(sys::MQCA_BASE_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_CF_STRUC_NAME: InqReqType = inqreq_str(sys::MQCA_CF_STRUC_NAME, sys::MQ_CF_STRUC_NAME_LENGTH);
pub const MQCA_CHANNEL_AUTO_DEF_EXIT: InqReqType = inqreq_str(sys::MQCA_CHANNEL_AUTO_DEF_EXIT, sys::MQ_EXIT_NAME_LENGTH);
pub const MQCA_CLUS_CHL_NAME: InqReqType = inqreq_str(sys::MQCA_CLUS_CHL_NAME, sys::MQ_CHANNEL_NAME_LENGTH);
pub const MQCA_CLUSTER_NAME: InqReqType = inqreq_str(sys::MQCA_CLUSTER_NAME, sys::MQ_CLUSTER_NAME_LENGTH);
pub const MQCA_CLUSTER_NAMELIST: InqReqType = inqreq_str(sys::MQCA_CLUSTER_NAMELIST, sys::MQ_NAMELIST_NAME_LENGTH);
pub const MQCA_CLUSTER_WORKLOAD_DATA: InqReqType = inqreq_str(sys::MQCA_CLUSTER_WORKLOAD_DATA, sys::MQ_EXIT_DATA_LENGTH);
pub const MQCA_CLUSTER_WORKLOAD_EXIT: InqReqType = inqreq_str(sys::MQCA_CLUSTER_WORKLOAD_EXIT, sys::MQ_EXIT_NAME_LENGTH);
pub const MQCA_COMMAND_INPUT_Q_NAME: InqReqType = inqreq_str(sys::MQCA_COMMAND_INPUT_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_CREATION_DATE: InqReqType = inqreq_str(sys::MQCA_CREATION_DATE, sys::MQ_CREATION_DATE_LENGTH);
pub const MQCA_CREATION_TIME: InqReqType = inqreq_str(sys::MQCA_CREATION_TIME, sys::MQ_CREATION_TIME_LENGTH);
pub const MQCA_CUSTOM: InqReqType = inqreq_str(sys::MQCA_CUSTOM, sys::MQ_CUSTOM_LENGTH);
pub const MQCA_DEAD_LETTER_Q_NAME: InqReqType = inqreq_str(sys::MQCA_DEAD_LETTER_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_DEF_XMIT_Q_NAME: InqReqType = inqreq_str(sys::MQCA_DEF_XMIT_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_DNS_GROUP: InqReqType = inqreq_str(sys::MQCA_DNS_GROUP, sys::MQ_DNS_GROUP_NAME_LENGTH);
pub const MQCA_ENV_DATA: InqReqType = inqreq_str(sys::MQCA_ENV_DATA, sys::MQ_PROCESS_ENV_DATA_LENGTH);
pub const MQCA_IGQ_USER_ID: InqReqType = inqreq_str(sys::MQCA_IGQ_USER_ID, sys::MQ_USER_ID_LENGTH);
pub const MQCA_INITIAL_KEY: InqReqType = inqreq_str(sys::MQCA_INITIAL_KEY, sys::MQ_INITIAL_KEY_LENGTH);
pub const MQCA_INITIATION_Q_NAME: InqReqType = inqreq_str(sys::MQCA_INITIATION_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_INSTALLATION_DESC: InqReqType = inqreq_str(sys::MQCA_INSTALLATION_DESC, sys::MQ_INSTALLATION_DESC_LENGTH);
pub const MQCA_INSTALLATION_NAME: InqReqType = inqreq_str(sys::MQCA_INSTALLATION_NAME, sys::MQ_INSTALLATION_NAME_LENGTH);
pub const MQCA_INSTALLATION_PATH: InqReqType = inqreq_str(sys::MQCA_INSTALLATION_PATH, sys::MQ_INSTALLATION_PATH_LENGTH);
pub const MQCA_LU_GROUP_NAME: InqReqType = inqreq_str(sys::MQCA_LU_GROUP_NAME, sys::MQ_LU_NAME_LENGTH);
pub const MQCA_LU_NAME: InqReqType = inqreq_str(sys::MQCA_LU_NAME, sys::MQ_LU_NAME_LENGTH);
pub const MQCA_LU62_ARM_SUFFIX: InqReqType = inqreq_str(sys::MQCA_LU62_ARM_SUFFIX, sys::MQ_ARM_SUFFIX_LENGTH);
pub const MQCA_NAMELIST_DESC: InqReqType = inqreq_str(sys::MQCA_NAMELIST_DESC, sys::MQ_NAMELIST_DESC_LENGTH);
pub const MQCA_NAMELIST_NAME: InqReqType = inqreq_str(sys::MQCA_NAMELIST_NAME, sys::MQ_NAMELIST_NAME_LENGTH);
pub const MQCA_PARENT: InqReqType = inqreq_str(sys::MQCA_PARENT, sys::MQ_Q_MGR_NAME_LENGTH);
pub const MQCA_PROCESS_DESC: InqReqType = inqreq_str(sys::MQCA_PROCESS_DESC, sys::MQ_PROCESS_DESC_LENGTH);
pub const MQCA_PROCESS_NAME: InqReqType = inqreq_str(sys::MQCA_PROCESS_NAME, sys::MQ_PROCESS_NAME_LENGTH);
pub const MQCA_Q_DESC: InqReqType = inqreq_str(sys::MQCA_Q_DESC, sys::MQ_Q_DESC_LENGTH);
pub const MQCA_Q_MGR_DESC: InqReqType = inqreq_str(sys::MQCA_Q_MGR_DESC, sys::MQ_Q_MGR_DESC_LENGTH);
pub const MQCA_Q_MGR_IDENTIFIER: InqReqType = inqreq_str(sys::MQCA_Q_MGR_IDENTIFIER, sys::MQ_Q_MGR_IDENTIFIER_LENGTH);
pub const MQCA_Q_MGR_NAME: InqReqType = inqreq_str(sys::MQCA_Q_MGR_NAME, sys::MQ_Q_MGR_NAME_LENGTH);
pub const MQCA_Q_NAME: InqReqType = inqreq_str(sys::MQCA_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_QSG_NAME: InqReqType = inqreq_str(sys::MQCA_QSG_NAME, sys::MQ_QSG_NAME_LENGTH);
pub const MQCA_REMOTE_Q_MGR_NAME: InqReqType = inqreq_str(sys::MQCA_REMOTE_Q_MGR_NAME, sys::MQ_Q_MGR_NAME_LENGTH);
pub const MQCA_REMOTE_Q_NAME: InqReqType = inqreq_str(sys::MQCA_REMOTE_Q_NAME, sys::MQ_Q_NAME_LENGTH);
pub const MQCA_REPOSITORY_NAME: InqReqType = inqreq_str(sys::MQCA_REPOSITORY_NAME, sys::MQ_CLUSTER_NAME_LENGTH);
pub const MQCA_REPOSITORY_NAMELIST: InqReqType = inqreq_str(sys::MQCA_REPOSITORY_NAMELIST, sys::MQ_NAMELIST_NAME_LENGTH);
pub const MQCA_SSL_KEY_REPO_PASSWORD: InqReqType =
    inqreq_str(sys::MQCA_SSL_KEY_REPO_PASSWORD, sys::MQ_SSL_ENCRYP_KEY_REPO_PWD_LEN);
pub const MQCA_STORAGE_CLASS: InqReqType = inqreq_str(sys::MQCA_STORAGE_CLASS, sys::MQ_STORAGE_CLASS_LENGTH);
pub const MQCA_TCP_NAME: InqReqType = inqreq_str(sys::MQCA_TCP_NAME, sys::MQ_TCP_NAME_LENGTH);
pub const MQCA_TRIGGER_DATA: InqReqType = inqreq_str(sys::MQCA_TRIGGER_DATA, sys::MQ_TRIGGER_DATA_LENGTH);
pub const MQCA_USER_DATA: InqReqType = inqreq_str(sys::MQCA_USER_DATA, sys::MQ_PROCESS_USER_DATA_LENGTH);
pub const MQCA_XMIT_Q_NAME: InqReqType = inqreq_str(sys::MQCA_XMIT_Q_NAME, sys::MQ_Q_NAME_LENGTH);

// TODO: Add some further constants supported as per
// https://www.ibm.com/docs/en/ibm-mq/9.4?topic=formats-mqcmd-inquire-q-inquire-queue
// https://www.ibm.com/docs/en/ibm-mq/9.4?topic=formats-mqcmd-inquire-q-mgr-inquire-queue-manager
// ..etc
