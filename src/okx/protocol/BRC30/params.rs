use crate::okx::protocol::BRC30::num::Num;
use once_cell::sync::Lazy;

pub const PROTOCOL_LITERAL: &str = "brc-30";
pub const NATIVE_TOKEN: &str = "btc";
pub const NATIVE_TOKEN_DECIMAL: u8 = 8_u8;
pub const MAX_DECIMAL_WIDTH: u8 = 18;
pub const MAX_SUPPLY_WIDTH: u128 = 18;
pub const TICK_ID_BYTE_COUNT: usize = 5;

pub const TICK_BYTE_MIN_COUNT: usize = 4;
pub const TICK_BYTE_MAX_COUNT: usize = 6;

pub const POOL_TYPE: &str = "pool";
pub const FIXED_TYPE: &str = "fixed";

pub const PID_BYTE_COUNT: usize = 13;

pub static MAXIMUM_SUPPLY: Lazy<Num> = Lazy::new(|| Num::from(u64::MAX));

pub static BIGDECIMAL_TEN: Lazy<Num> = Lazy::new(|| Num::from(10u64));

pub const fn default_decimals() -> u8 {
  MAX_DECIMAL_WIDTH
}
