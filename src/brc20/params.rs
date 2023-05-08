use crate::brc20::num::Num;
use once_cell::sync::Lazy;

pub const PROTOCOL_LITERAL: &str = "brc-20";
pub const MAX_DECIMAL_WIDTH: u8 = 18;
pub const TICK_BYTE_COUNT: usize = 4;

pub static MAXIMUM_SUPPLY: Lazy<Num> =
  Lazy::new(|| Num::from_str_radix("FFFFFFFFFFFFFFFF", 16).unwrap());

pub const fn default_decimals() -> u8 {
  MAX_DECIMAL_WIDTH
}
