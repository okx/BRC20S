use {super::*, crate::okx::datastore::brc20s, axum::Json};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Transferable {
  pub inscriptions: Vec<Inscription>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Inscription {
  pub tick: Tick,
  pub inscription_id: String,
  pub inscription_number: i64,
  pub amount: String,
  pub owner: String,
}

impl Inscription {
  pub fn set_tick_name(&mut self, name: String) {
    self.tick.name = name;
  }

  pub fn set_inscription_number(&mut self, inscription_number: i64) {
    self.inscription_number = inscription_number;
  }
}

impl From<&brc20s::TransferableAsset> for Inscription {
  fn from(asset: &brc20s::TransferableAsset) -> Self {
    let tick = Tick {
      id: asset.tick_id.hex(),
      name: "".to_string(),
    };

    Self {
      tick,
      inscription_id: asset.inscription_id.to_string(),
      inscription_number: 0,
      amount: asset.amount.to_string(),
      owner: asset.owner.to_string(),
    }
  }
}

// brc20s/tick/:tickId/address/:address/transferable
pub(crate) async fn brc20s_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path((tick_id, address)): Path<(String, String)>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc20s_transferable: {},{}", tick_id, address);

  let tick_id = brc20s::TickId::from_str(&tick_id)
    .map_err(|_| ApiError::bad_request(BRC20SError::IncorrectTickIdFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;
  let all_transfer = &index.brc20s_tickid_transferable(&tick_id, &address)?;

  log::debug!(
    "rpc: get brc20s_transferable: {:?} {:?}",
    tick_id.hex(),
    all_transfer
  );

  Ok(Json(ApiResponse::ok(Transferable {
    inscriptions: all_transfer
      .iter()
      .map(|asset| {
        let mut inscription = Inscription::from(asset);

        let tick_info = &index.brc20s_tick_info(&asset.tick_id).unwrap().unwrap();

        let inscription_number = &index
          .get_inscription_entry(asset.inscription_id)
          .unwrap()
          .unwrap();

        inscription.set_tick_name(tick_info.name.as_str().to_string());
        inscription.set_inscription_number(inscription_number.number);
        inscription
      })
      .collect(),
  })))
}

// brc20s/address/:address/transferable
pub(crate) async fn brc20s_all_transferable(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<Transferable> {
  log::debug!("rpc: get brc20s_all_transferable: {}", address);

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let all = index.brc20s_all_transferable(&address)?;

  log::debug!("rpc: get brc20s_all_transferable: {} {:?}", address, all);

  Ok(Json(ApiResponse::ok(Transferable {
    inscriptions: all
      .iter()
      .map(|asset| {
        let mut inscription = Inscription::from(asset);

        let tick_info = &index.brc20s_tick_info(&asset.tick_id).unwrap().unwrap();

        let inscription_number = &index
          .get_inscription_entry(asset.inscription_id)
          .unwrap()
          .unwrap();

        inscription.set_tick_name(tick_info.name.as_str().to_string());
        inscription.set_inscription_number(inscription_number.number);
        inscription
      })
      .collect(),
  })))
}
