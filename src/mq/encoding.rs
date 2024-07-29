/// `(CCSID, encoding, short description)`
pub type CcsidEntry = (i32, u8, &'static str);

// Extracted from "ccsid.tbl" file from IBM MQ
// `grep -v '^[[:space:]]*#' | awk '{  if ($5!="2") print $1, $6, $8}' | sort -n > character_sets.txt``
//
// Note:
// DBCS ccsid excluded
// Encoding => 1=EBCDIC 2=ASCII 3=ISO  4=UCS-2 5=UTF-8 6=euc 7=GB18030

const CCSID: &[CcsidEntry] = &[
    (37, 1, "IBM-037"),
    (256, 1, "IBM-256"),
    (273, 1, "IBM-273"),
    (277, 1, "IBM-277"),
    (278, 1, "IBM-278"),
    (280, 1, "IBM-280"),
    (284, 1, "IBM-284"),
    (285, 1, "IBM-285"),
    (286, 1, "IBM-286"),
    (290, 1, "IBM-290"),
    (297, 1, "IBM-297"),
    (367, 3, "IBM-367"),
    (420, 1, "IBM-420"),
    (421, 1, "IBM-421"),
    (424, 1, "IBM-424"),
    (425, 1, "IBM-425"),
    (437, 2, "IBM-437"),
    (500, 1, "IBM-500"),
    (720, 2, "IBM-720"),
    (737, 2, "IBM-737"),
    (775, 2, "IBM-775"),
    (803, 1, "IBM-803"),
    (806, 2, "IBM-806"),
    (813, 3, "ISO-8859-7"),
    (819, 3, "ANSI_X3.4-1968"),
    (819, 3, "ISO-8859-1"),
    (833, 1, "IBM-833"),
    (836, 1, "IBM-836"),
    (838, 1, "IBM-838"),
    (850, 2, "IBM-850"),
    (852, 2, "IBM-852"),
    (853, 2, "IBM-853"),
    (855, 2, "IBM-855"),
    (856, 2, "IBM-856"),
    (857, 2, "IBM-857"),
    (858, 2, "IBM-858"),
    (860, 2, "IBM-860"),
    (861, 2, "IBM-861"),
    (862, 2, "IBM-860"),
    (863, 2, "IBM-863"),
    (864, 2, "IBM-864"),
    (865, 2, "IBM-865"),
    (866, 2, "IBM-866"),
    (867, 2, "IBM-867"),
    (868, 2, "IBM-868"),
    (869, 2, "IBM-869"),
    (870, 1, "IBM-870"),
    (871, 1, "IBM-871"),
    (874, 2, "TIS-620"),
    (875, 1, "IBM-875"),
    (878, 3, "KOI8-R"),
    (880, 1, "IBM-880"),
    (891, 2, "IBM-891"),
    (895, 3, "IBM-895"),
    (897, 2, "IBM-897"),
    (901, 3, "IBM-901"),
    (902, 3, "IBM-902"),
    (903, 2, "IBM-903"),
    (904, 2, "IBM-904"),
    (912, 3, "ISO-8859-2"),
    (913, 3, "ISO-8859-3"),
    (915, 3, "ISO-8859-5"),
    (916, 3, "ISO-8859-8"),
    (918, 1, "IBM-918"),
    (920, 3, "ISO-8859-9"),
    (921, 3, "ISO-8859-13"),
    (922, 3, "IBM-922"),
    (923, 3, "ISO-8859-15"),
    (924, 1, "IBM-924"),
    (930, 1, "IBM-930"),
    (931, 1, "IBM-931"),
    (932, 2, "IBM-932"),
    (933, 1, "IBM-933"),
    (935, 1, "IBM-935"),
    (936, 2, "IBM-936"),
    (937, 1, "IBM-937"),
    (938, 2, "IBM-938"),
    (939, 1, "IBM-939"),
    (942, 2, "IBM-942"),
    (943, 2, "PCK"),
    (943, 2, "SHIFT_JIS"),
    (948, 2, "IBM-948"),
    (949, 2, "IBM-949"),
    (950, 2, "BIG5"),
    (950, 2, "BIG5-HKSCS"),
    (950, 2, "BIG5HKSCS"),
    (954, 6, "EUC-JP"),
    (964, 6, "EUC-TW"),
    (970, 6, "EUC-KR"),
    (1006, 3, "IBM-1006"),
    (1010, 3, "IBM-1010"),
    (1011, 3, "IBM-1011"),
    (1012, 3, "IBM-1012"),
    (1013, 3, "IBM-1013"),
    (1014, 3, "IBM-1014"),
    (1015, 3, "IBM-1015"),
    (1016, 3, "IBM-1016"),
    (1017, 3, "IBM-1017"),
    (1018, 3, "IBM-1018"),
    (1019, 3, "IBM-1019"),
    (1025, 1, "IBM-1025"),
    (1026, 1, "IBM-1026"),
    (1027, 1, "IBM-1027"),
    (1040, 2, "IBM-1040"),
    (1041, 2, "IBM-1041"),
    (1042, 2, "IBM-1042"),
    (1043, 2, "IBM-1043"),
    (1046, 2, "IBM-1046"),
    (1047, 1, "IBM-1047"),
    (1051, 3, "IBM-1051"),
    (1088, 2, "IBM-1088"),
    (1089, 3, "ISO-8859-6"),
    (1097, 1, "IBM-1097"),
    (1098, 2, "IBM-1098"),
    (1112, 1, "IBM-1112"),
    (1114, 2, "IBM-1114"),
    (1115, 2, "IBM-1115"),
    (1122, 1, "IBM-1122"),
    (1123, 1, "IBM-1123"),
    (1124, 3, "IBM-1124"),
    (1125, 2, "IBM-1125"),
    (1127, 2, "IBM-1127"),
    (1129, 3, "IBM-1129"),
    (1130, 1, "IBM-1130"),
    (1131, 2, "IBM-1131"),
    (1132, 1, "IBM-1132"),
    (1133, 3, "IBM-1133"),
    (1137, 1, "IBM-1137"),
    (1140, 1, "IBM-1140"),
    (1141, 1, "IBM-1141"),
    (1142, 1, "IBM-1142"),
    (1143, 1, "IBM-1143"),
    (1144, 1, "IBM-1144"),
    (1145, 1, "IBM-1145"),
    (1146, 1, "IBM-1146"),
    (1147, 1, "IBM-1147"),
    (1148, 1, "IBM-1148"),
    (1149, 1, "IBM-1149"),
    (1153, 1, "IBM-1153"),
    (1156, 1, "IBM-1156"),
    (1157, 1, "IBM-1157"),
    (1159, 1, "IBM-1159"),
    (1208, 5, "UTF-8"),
    (1250, 2, "IBM-1250"),
    (1251, 2, "CP1251"),
    (1252, 2, "IBM-1252"),
    (1253, 2, "IBM-1253"),
    (1254, 2, "IBM-1254"),
    (1255, 2, "CP1255"),
    (1256, 2, "IBM-1256"),
    (1257, 2, "IBM-1257"),
    (1258, 2, "IBM-1258"),
    (1275, 2, "IBM-1275"),
    (1279, 1, "IBM-1279"),
    (1280, 2, "IBM-1280"),
    (1281, 2, "IBM-1281"),
    (1282, 2, "IBM-1282"),
    (1283, 2, "IBM-1283"),
    (1284, 2, "IBM-1284"),
    (1285, 2, "IBM-1285"),
    (1350, 6, "JISeucJP"),
    (1363, 2, "IBM-1363"),
    (1364, 1, "IBM-1364"),
    (1370, 2, "IBM-1370"),
    (1371, 1, "IBM-1371"),
    (1381, 2, "GB2312"),
    (1383, 6, "EUC-CN"),
    (1386, 2, "GBK"),
    (1388, 1, "IBM-1388"),
    (1390, 1, "IBM-1390"),
    (1392, 7, "GB18030"),
    (1399, 1, "IBM-1399"),
    (4133, 1, "IBM-037"),
    (4325, 1, "IBM-256"),
    (4369, 1, "IBM-273"),
    (4370, 1, "IBM-4370"),
    (4371, 1, "IBM-4371"),
    (4372, 1, "IBM-4372"),
    (4373, 1, "IBM-277"),
    (4374, 1, "IBM-278"),
    (4376, 1, "IBM-280"),
    (4378, 1, "IBM-4378"),
    (4380, 1, "IBM-284"),
    (4381, 1, "IBM-285"),
    (4386, 1, "IBM-290"),
    (4393, 1, "IBM-297"),
    (4516, 1, "IBM-420"),
    (4520, 1, "IBM-424"),
    (4533, 2, "IBM-437"),
    (4596, 1, "IBM-500"),
    (4899, 1, "IBM-4899"),
    (4909, 3, "ISO-8859-7@euro"),
    (4929, 1, "IBM-833"),
    (4932, 1, "IBM-836"),
    (4934, 1, "IBM-838"),
    (4946, 2, "IBM-850"),
    (4948, 2, "IBM-852"),
    (4949, 2, "IBM-853"),
    (4951, 2, "IBM-855"),
    (4952, 2, "IBM-856"),
    (4953, 2, "IBM-857"),
    (4960, 2, "IBM-864"),
    (4964, 2, "IBM-868"),
    (4965, 2, "IBM-869"),
    (4966, 1, "IBM-870"),
    (4967, 1, "IBM-871"),
    (4970, 2, "TIS-620"),
    (4971, 1, "IBM-4971"),
    (4976, 1, "IBM-880"),
    (4993, 2, "IBM-897"),
    (5014, 1, "IBM-918"),
    (5026, 1, "IBM-930"),
    (5028, 2, "IBM-932"),
    (5029, 1, "IBM-933"),
    (5031, 1, "IBM-935"),
    (5033, 1, "IBM-937"),
    (5035, 1, "IBM-939"),
    (5039, 2, "IBM-5039"),
    (5039, 2, "IBM-5039"),
    (5045, 2, "IBM-949"),
    (5046, 2, "BIG5"),
    (5050, 6, "EUC-JP"),
    (5060, 6, "EUC-TW"),
    (5066, 6, "EUC-KR"),
    (5123, 1, "IBM-5123"),
    (5137, 2, "IBM-1041"),
    (5142, 2, "IBM-1046"),
    (5143, 1, "IBM-1047"),
    (5210, 2, "IBM-5210"),
    (5211, 2, "IBM-1115"),
    (5346, 2, "IBM-5346"),
    (5347, 2, "IBM-5347"),
    (5348, 2, "IBM-5348"),
    (5349, 2, "IBM-5349"),
    (5350, 2, "IBM-5350"),
    (5351, 2, "IBM-5351"),
    (5352, 2, "IBM-5352"),
    (5353, 2, "IBM-5353"),
    (5354, 2, "IBM-5354"),
    (5477, 2, "GB2312"),
    (5479, 6, "EUC-CN"),
    (5482, 2, "GBK"),
    (5484, 1, "IBM-1388"),
    (5488, 7, "GB18030"),
    (8229, 1, "IBM-037"),
    (8448, 1, "IBM-256"),
    (8478, 1, "IBM-284"),
    (8482, 1, "IBM-8482"),
    (8489, 1, "IBM-297"),
    (8612, 1, "IBM-8612"),
    (8629, 2, "IBM-437"),
    (8692, 1, "IBM-500"),
    (9025, 1, "IBM-833"),
    (9028, 1, "IBM-836"),
    (9030, 1, "IBM-838"),
    (9044, 2, "IBM-9044"),
    (9047, 2, "IBM-855"),
    (9048, 2, "IBM-9048"),
    (9056, 2, "IBM-864"),
    (9060, 2, "IBM-868"),
    (9061, 2, "IBM-9061"),
    (9066, 2, "TIS-620"),
    (9089, 2, "IBM-897"),
    (9122, 1, "IBM-930"),
    (9124, 2, "IBM-932"),
    (9125, 1, "IBM-933"),
    (9127, 1, "IBM-935"),
    (9131, 1, "IBM-939"),
    (9142, 2, "BIG5"),
    (9146, 6, "EUC-JP"),
    (9449, 2, "IBM-9449"),
    (12325, 1, "IBM-037"),
    (12544, 1, "IBM-256"),
    (12712, 1, "IBM-12712"),
    (12725, 2, "IBM-437"),
    (12788, 1, "IBM-500"),
    (13152, 2, "IBM-864"),
    (13218, 1, "IBM-930"),
    (13219, 1, "IBM-931"),
    (13221, 1, "IBM-933"),
    (13223, 1, "IBM-935"),
    (13238, 2, "BIG5"),
    (13242, 6, "EUC-JP"),
    (16421, 1, "IBM-037"),
    (16821, 2, "IBM-437"),
    (16884, 1, "IBM-500"),
    (17314, 1, "IBM-930"),
    (20517, 1, "IBM-037"),
    (20917, 2, "IBM-437"),
    (20980, 1, "IBM-500"),
    (24613, 1, "IBM-037"),
    (25076, 1, "IBM-500"),
    (29172, 1, "IBM-500"),
    (32805, 1, "IBM-037"),
    (33058, 1, "IBM-290"),
    (33268, 1, "IBM-500"),
    (33618, 2, "IBM-850"),
    (33698, 1, "IBM-930"),
    (33699, 1, "IBM-931"),
    (33722, 6, "EUC-JP"),
    (37364, 1, "IBM-500"),
    (41460, 1, "IBM-500"),
    (45556, 1, "IBM-500"),
    (49652, 1, "IBM-500"),
    (53748, 1, "IBM-500"),
    (61696, 1, "IBM-500"),
    (61697, 2, "IBM-850"),
    (61698, 2, "IBM-850"),
    (61699, 3, "ISO-8859-1"),
    (61710, 3, "ISO-8859-1"),
    (61711, 1, "IBM-500"),
    (61712, 1, "IBM-500"),
];

// Refer to https://www.ibm.com/docs/en/iis/latest?topic=tables-ebcdic-ascii

const ASCII7_EBCDIC: [u8; 256] = [
    0x00, // NUL
    0x01, // SOH
    0x02, // STX
    0x03, // ETX
    0x1A, // SEL
    0x09, // HT
    0x1A, // RNL
    0x7F, // DEL
    0x1A, // GE
    0x1A, // SPS
    0x1A, // RPT
    0x0B, // VT
    0x0C, // FF
    0x0D, // CR
    0x0E, // SO
    0x0F, // SI
    0x10, // DLE
    0x11, // DC1
    0x12, // DC2
    0x13, // DC3
    0x3C, // DC4
    0x3D, // NAK
    0x32, // SYN
    0x26, // ETB
    0x18, // CAN
    0x19, // EM
    0x3F, // SUB
    0x27, // ESC
    0x1C, // FS
    0x1D, // GS
    0x1E, // RS
    0x1F, // US
    0x40, // (space)
    0x4F, // !
    0x7F, // “
    0x7B, // #
    0x5B, // $
    0x6C, // %
    0x50, // &
    0x7D, // ‘
    0x4D, // (
    0x5D, // )
    0x5C, // *
    0x4E, // +
    0x6B, // ,
    0x60, // –
    0x4B, // .
    0x61, // /
    0xF0, // 0
    0xF1, // 1
    0xF2, // 2
    0xF3, // 3
    0xF4, // 4
    0xF5, // 5
    0xF6, // 6
    0xF7, // 7
    0xF8, // 8
    0xF9, // 9
    0x7A, // :
    0x5E, // ;
    0x4C, // >
    0x7E, // =
    0x6E, // >
    0x6F, // ?
    0x7C, // @
    0xC1, // A
    0xC2, // B
    0xC3, // C
    0xC4, // D
    0xC5, // E
    0xC6, // F
    0xC7, // G
    0xC8, // H
    0xC9, // I
    0xD1, // J
    0xD2, // K
    0xD3, // L
    0xD4, // M
    0xD5, // N
    0xD6, // O
    0xD7, // P
    0xD8, // Q
    0xD9, // R
    0xE2, // S
    0xE3, // T
    0xE4, // U
    0xE5, // V
    0xE6, // W
    0xE7, // X
    0xE8, // Y
    0xE9, // Z
    0x4A, // [
    0xE0, // \
    0x5A, // ]
    0x5F, // ^
    0x6D, // _
    0x79, // ′
    0x81, // a
    0x82, // b
    0x83, // c
    0x84, // d
    0x85, // e
    0x86, // f
    0x87, // g
    0x88, // h
    0x89, // i
    0x91, // j
    0x92, // k
    0x93, // l
    0x94, // m
    0x95, // n
    0x96, // o
    0x97, // p
    0x98, // q
    0x99, // r
    0xA2, // s
    0xA3, // t
    0xA4, // u
    0xA5, // v
    0xA6, // w
    0xA7, // x
    0xA8, // y
    0xA9, // z
    0xC0, // {
    0x6A, // ¦
    0xD0, // }
    0xA1, // ~
    0x07, // DEL
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
    0x3F, // (no mapping)
];

const EBCDIC_ASCII7: [u8; 256] = [
    0x00, // NUL
    0x01, // SOH
    0x02, // STX
    0x03, // ETX
    0x1A, // SEL (no mapping)
    0x09, // HT
    0x1A, // RNL (no mapping)
    0x7F, // DEL
    0x1A, // GE (no mapping)
    0x1A, // SPS (no mapping)
    0x1A, // RPT (no mapping)
    0x0B, // VT
    0x0C, // FF
    0x0D, // CR
    0x0E, // SO
    0x0F, // SI
    0x10, // DLE
    0x11, // DC1
    0x12, // DC2
    0x13, // DC3
    0x1A, // RES/ENP (no mapping)
    0x1A, // NL (no mapping)
    0x08, // BS
    0x1A, // POC (no mapping)
    0x18, // CAN
    0x19, // EM
    0x1A, // UBS (no mapping)
    0x1A, // CU1 (no mapping)
    0x1C, // IFS
    0x1D, // IGS
    0x1E, // IRS
    0x1F, // ITB/IUS
    0x1A, // DS (no mapping)
    0x1A, // SOS (no mapping)
    0x1A, // FS (no mapping)
    0x1A, // WUS (no mapping)
    0x1A, // BYP/INP (no mapping)
    0x0A, // LF
    0x17, // ETB
    0x1B, // ESC
    0x1A, // SA (no mapping)
    0x1A, // SFE (no mapping)
    0x1A, // SM/SW (no mapping)
    0x1A, // CSP (no mapping)
    0x1A, // MFA (no mapping)
    0x05, // ENQ
    0x06, // ACK
    0x07, // BEL
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x16, // SYN
    0x1A, // IR (no mapping)
    0x1A, // PP (no mapping)
    0x1A, // TRN (no mapping)
    0x1A, // NBS (no mapping)
    0x04, // EOT
    0x1A, // SBS (no mapping)
    0x1A, // IT (no mapping)
    0x1A, // RFF (no mapping)
    0x1A, // CU3 (no mapping)
    0x14, // DC4
    0x15, // NAK
    0x1A, // (no mapping)
    0x1A, // SUB (no mapping)
    0x20, // (space)
    0x1A, // RSP (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x5B, // ¢
    0x2E, // .
    0x3C, // <
    0x28, // (
    0x2B, // +
    0x21, // |
    0x26, // &
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x5D, // !
    0x24, // $
    0x2A, // *
    0x29, // )
    0x3B, // ;
    0x5E, // ‥
    0x2D, // -
    0x1A, // / (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x7C, //
    0x2C, // ‘
    0x25, //
    0x5F, // _
    0x3E, // >
    0x3F, // ?
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x60, //
    0x3A, // :
    0x23, // #
    0x40, // @
    0x27, // '
    0x3D, // =
    0x22, // "
    0x1A, // (no mapping)
    0x61, // a
    0x62, // b
    0x63, // c
    0x64, // d
    0x65, // e
    0x66, // f
    0x67, // g
    0x68, // h
    0x69, // i
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x6A, // j
    0x6B, // k
    0x6C, // l
    0x6D, // m
    0x6E, // n
    0x6F, // o
    0x70, // p
    0x71, // q
    0x72, // r
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x7E, // ∞
    0x73, // s
    0x74, // t
    0x75, // u
    0x76, // v
    0x77, // w
    0x78, // x
    0x79, // y
    0x7A, // z
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x7B, //
    0x41, // A
    0x42, // B
    0x43, // C
    0x44, // D
    0x45, // E
    0x46, // F
    0x47, // G
    0x48, // H
    0x49, // I
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x7D, //
    0x4A, // J
    0x4B, // K
    0x4C, // L
    0x4D, // M
    0x4E, // N
    0x4F, // O
    0x50, // P
    0x51, // Q
    0x52, // R
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x5C, // \
    0x1A, // (no mapping)
    0x53, // S
    0x54, // T
    0x55, // U
    0x56, // V
    0x57, // W
    0x58, // X
    0x59, // Y
    0x5A, // Z
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x30, // 0
    0x31, // 1
    0x32, // 2
    0x33, // 3
    0x34, // 4
    0x35, // 5
    0x36, // 6
    0x37, // 7
    0x38, // 8
    0x39, // 9
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
    0x1A, // (no mapping)
];

const CCSID_2K: [CcsidEntry; 2048] = ccsid_lookup_init(); // Efficient lookup for the first 2k

const fn convert<const N: usize>(input: &[u8; N], table: &[u8; 256]) -> [u8; N] {
    let mut result = [0; N];
    let mut i = 0;
    while i < N {
        result[i] = table[input[i] as usize];
        i += 1;
    }
    result
}

#[must_use]
pub const fn ebcdic_ascii7<const N: usize>(ebcdic: &[u8; N]) -> [u8; N] {
    convert(ebcdic, &EBCDIC_ASCII7)
}

#[must_use]
pub const fn ascii7_ebcdic<const N: usize>(ascii: &[u8; N]) -> [u8; N] {
    convert(ascii, &ASCII7_EBCDIC)
}

const fn ccsid_lookup_init<const N: usize>() -> [(i32, u8, &'static str); N] {
    let mut i = 0;
    let mut result = [(0, 0, ""); N];
    while i < CCSID.len() {
        let entry @ (ccsid, ..) = CCSID[i];
        #[allow(clippy::cast_sign_loss)]
        let uccsid = ccsid as usize;
        if uccsid < N {
            result[uccsid] = entry;
        }
        i += 1;
    }
    result
}

#[must_use]
pub fn ccsid_lookup(ccsid: i32) -> Option<&'static CcsidEntry> {
    if ccsid < 2048 {
        #[allow(clippy::cast_sign_loss)]
        Some(&CCSID_2K[ccsid as usize]).filter(|(ccsid, ..)| *ccsid != 0)
    } else {
        CCSID
            .binary_search_by_key(&ccsid, |(ccsid_entry, ..)| *ccsid_entry)
            .map(|index| &CCSID[index])
            .ok()
    }
}

#[must_use]
pub fn is_ebcdic(ccsid: i32) -> Option<bool> {
    ccsid_lookup(ccsid).map(|&(_, encoding, _)| encoding == 1)
}

#[cfg(test)]
mod tests {
    use crate::encoding::{ccsid_lookup, is_ebcdic};

    #[test]
    fn ccsid_lookup_all() {
        assert!(ccsid_lookup(1).is_none());
        assert!(ccsid_lookup(3000).is_none());
        assert!(ccsid_lookup(37).is_some_and(|(.., name)| *name == "IBM-037"));
        assert!(ccsid_lookup(8612).is_some_and(|(.., name)| *name == "IBM-8612"));
    }

    #[test]
    fn is_ebcdic_all() {
        assert!(is_ebcdic(1).is_none());
        assert!(is_ebcdic(37).is_some_and(|ebcdic| ebcdic));
        assert!(is_ebcdic(1208).is_some_and(|ebcdic| !ebcdic));
    }
}
