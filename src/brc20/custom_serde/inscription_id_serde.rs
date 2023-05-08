use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub struct InscriptionIDSerde;

impl InscriptionIDSerde {
  pub fn serialize<S>(val: &[u8; 36], serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut vec = Vec::<u8>::with_capacity(val.len());
    vec.extend(val);
    vec.serialize(serializer)
  }

  pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 36], D::Error>
  where
    D: Deserializer<'de>,
  {
    let vec = Vec::<u8>::deserialize(deserializer)
      .map_err(|e| de::Error::custom(format!("deseralize u8 array error: {}", e)))?;
    Ok(
      vec
        .try_into()
        .map_err(|e| de::Error::custom(format!("invalid inscription id: {:?}", e)))?,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Serialize, Deserialize)]
  struct Test {
    #[serde(with = "InscriptionIDSerde")]
    t: [u8; 36],
  }

  #[test]
  fn test_inscription_id_serialize() {
    assert_eq!(
      serde_json::to_string(&Test { t: [b'a'; 36] }).unwrap(),
      r##"{"t":[97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97,97]}"##
    );
  }
}
