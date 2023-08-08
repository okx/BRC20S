use {
  super::{
    brc20::{
      AllBalance, AllTickInfo, Balance, BlockEvents, TickInfo, TransferableInscription,
      TransferableInscriptions, TxEvents,
    },
    info::NodeInfo,
    *,
  },
  utoipa::{ToResponse, ToSchema},
};
#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
#[aliases(
  BRC20TickResponse = ApiResponse<TickInfo>,
  BRC20AllTickResponse = ApiResponse<AllTickInfo>,
  BRC20BalanceResponse = ApiResponse<Balance>,
  BRC20AllBalanceResponse = ApiResponse<AllBalance>,
  BRC20TxEventsResponse = ApiResponse<TxEvents>,
  BRC20BlockEventsResponse = ApiResponse<BlockEvents>,
  BRC20TransferableResponse = ApiResponse<TransferableInscription>,
  BRC20AllTransferableResponse = ApiResponse<TransferableInscriptions>,
  NodeResponse = ApiResponse<NodeInfo>
)]
pub(crate) struct ApiResponse<T: Serialize> {
  pub code: i32,
  /// ok
  #[schema(example = "ok")]
  pub msg: String,
  pub data: T,
}

impl<T> ApiResponse<T>
where
  T: Serialize,
{
  fn new(code: i32, msg: String, data: T) -> Self {
    Self { code, msg, data }
  }

  pub fn ok(data: T) -> Self {
    Self::new(0, "ok".to_string(), data)
  }
}
