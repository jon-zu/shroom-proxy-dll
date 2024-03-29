use num_enum::{IntoPrimitive, TryFromPrimitive};


#[allow(non_camel_case_types)]
#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ClientErrorCode {
    EC_FAILED_PROTOCOL_WITH_GAME = 553648131,
    EC_INVALID_GAME_DATA = 570425350,
    EC_PATCH = 536870912,
    EC_DISCONNECT_BEGIN = 553648128,
    EC_DISCONNECT_END = 553648134,
    EC_TERMINATE_BEGIN = 570425344,
    EC_CONNECT_TO_LOGIN_FAILED = 570425345,
    EC_NOT_ENOUGH_MEMORY = 570425347,
    EC_NO_DATA_PACKAGE = 570425348,
    EC_INVALID_GAME_DATA_VERSION = 570425349,
    EC_WEB_LOGIN_NEEDED = 570425353,
    EC_AUTH_SETLOCALE_FAILED = 570425356,
    EC_AUTH_COINIT_FAILED = 570425357,
    EC_TERMINATE_END = 570425358,
    EC_CONNECT_TO_GAME_FAILED = 553648129,
    EC_CONNECTION_FROM_GAME_CLOSED = 553648130,
    EC_FORCE_DISCONNECT = 553648132,
    EC_DISCONNECT_BY_MALICIOUS_PROCESS = 553648133,
    EC_CONNECTION_FROM_LOGIN_CLOSED = 570425346,
    EC_INVALID_CLIENT_VERSION = 570425351,
    EC_FAILED_CRITICAL_PROTOCOL_WITH_GAME = 570425352,
    EC_CLIENTCRC_FAILED = 570425354,
    EC_DOWNLOAD_FULL_CLIENT = 570425355,
}
