use crate::okx::protocol::brc20s::num::Num;
use once_cell::sync::Lazy;

pub const PROTOCOL_LITERAL: &str = "brc20-s";
pub const NATIVE_TOKEN: &str = "btc";
pub const NATIVE_TOKEN_DECIMAL: u8 = 8_u8;
pub const MAX_DECIMAL_WIDTH: u8 = 18;
pub const TICK_ID_BYTE_COUNT: usize = 5;
pub const TICK_ID_STR_COUNT: usize = 10;
pub const TICK_BYTE_COUNT: usize = 4;
pub const MAX_STAKED_POOL_NUM: usize = 5;
pub const MAX_STAKED_POOL_NUM_V1: usize = 1024;

pub const TICK_BYTE_MIN_COUNT: usize = 4;
pub const TICK_BYTE_MAX_COUNT: usize = 6;

pub const POOL_TYPE: &str = "pool";
pub const FIXED_TYPE: &str = "fixed";
pub const PID_BYTE_COUNT: usize = 13;

pub static BIGDECIMAL_TEN: Lazy<Num> = Lazy::new(|| Num::from(10u64));
pub static ZERO_NUM: Lazy<Num> = Lazy::new(|| Num::from(0u64));
