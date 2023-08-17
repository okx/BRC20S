use {super::*, crate::okx::datastore::brc20s, axum::Json};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::TransferableInscription)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransferableInscription {
  /// Ticker.
  #[schema(value_type = brc20s::Tick)]
  pub tick: Tick,
  /// The inscription id.
  pub inscription_id: String,
  /// The inscription number.
  pub inscription_number: i64,
  /// The amount.
  #[schema(format = "uint64")]
  pub amount: String,
  /// The owner.
  pub owner: String,
}

impl TransferableInscription {
  pub fn set_tick_name(&mut self, name: String) {
    self.tick.name = name;
  }

  pub fn set_inscription_number(&mut self, inscription_number: i64) {
    self.inscription_number = inscription_number;
  }
}

impl From<&brc20s::TransferableAsset> for TransferableInscription {
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
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/tick/{tick_id}/address/{address}/transferable",
  operation_id = "get transferable inscriptions",
  params(
      ("tick_id" = String, Path, description = "Ticker ID", min_length = 10, max_length = 10, example = "a12345678f"),
      ("address" = String, Path, description = "Address")
),
  responses(
    (status = 200, description = "Obtain account transferable inscriptions of ticker ID.", body = BRC20STransferable),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
        let mut inscription = TransferableInscription::from(asset);

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20s::Transferable)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Transferable {
  #[schema(value_type = Vec<brc20s::TransferableInscription>)]
  pub inscriptions: Vec<TransferableInscription>,
}

// brc20s/address/:address/transferable
#[utoipa::path(
  get,
  path = "/api/v1/brc20s/address/{address}/transferable",
  operation_id = "get address ticker balance",
  params(
      ("address" = String, Path, description = "Address")
),
  responses(
    (status = 200, description = "Obtain account all transferable inscriptions.", body = BRC20STransferable),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
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
        let mut inscription = TransferableInscription::from(asset);

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
