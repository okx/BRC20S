use crate::okx::protocol::BRC30::num::Num;
use once_cell::sync::Lazy;

pub const PROTOCOL_LITERAL: &str = "brc-30";
pub const MAX_DECIMAL_WIDTH: u8 = 18;
pub const TICK_ID_BYTE_COUNT: usize = 5;

pub const TICK_BYTE_MIN_COUNT: usize = 4;
pub const TICK_BYTE_MAX_COUNT: usize = 6;
pub const TICK_SPECIAL: &str = "btc";

pub const PID_BYTE_COUNT: usize = 6;

pub static MAXIMUM_SUPPLY: Lazy<Num> = Lazy::new(|| Num::from(u64::MAX));

pub static BIGDECIMAL_TEN: Lazy<Num> = Lazy::new(|| Num::from(10u64));

pub const fn default_decimals() -> u8 {
  MAX_DECIMAL_WIDTH
}
