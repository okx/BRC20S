use super::super::error::BRC20Error;
use serde::{Deserialize, Serialize};
use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick(String);
// 此处提供一个lowercase 的方法。
impl FromStr for Tick {
  type Err = BRC20Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    todo!("通过字符串转成tick, 只能有4个字符")
  }
}
impl Display for Tick {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Tick {
  pub fn to_lowercase(&self) -> Tick {
    todo!("内部字符转换成小写")
  }
}
