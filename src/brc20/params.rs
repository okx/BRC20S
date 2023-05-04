use once_cell::sync::Lazy;
use crate::brc20::num::Num;

pub const PROTOCOL_LITERAL: &str = "brc-20";
pub const MAX_DECIMAL_WIDTH: u32 = 18;
pub const TICK_CHAR_COUNT: usize = 4;

pub static MAXIMUM_SUPPLY : Lazy<Num> = Lazy::new(||Num::from_str_radix("FFFFFFFFFFFFFFFF", 16).unwrap());

pub const fn default_decimals() -> u32 {
  MAX_DECIMAL_WIDTH
}
