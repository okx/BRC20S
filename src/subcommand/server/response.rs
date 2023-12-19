use {
  super::{info::NodeInfo, *},
  utoipa::ToSchema,
};
#[derive(Default, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[aliases(
  BRC20Tick = ApiResponse<brc20::TickInfo>,
  BRC20AllTick = ApiResponse<brc20::AllTickInfo>,
  BRC20Balance = ApiResponse<brc20::Balance>,
  BRC20AllBalance = ApiResponse<brc20::AllBalance>,
  BRC20TxEvents = ApiResponse<brc20::TxEvents>,
  BRC20BlockEvents = ApiResponse<brc20::BlockEvents>,
  BRC20Transferable = ApiResponse<brc20::TransferableInscriptions>,

  BRC20STick = ApiResponse<brc20s::TickInfo>,
  BRC20SAllTick = ApiResponse<brc20s::AllTickInfo>,
  BRC20SBalance = ApiResponse<brc20s::Balance>,
  BRC20SAllBalance = ApiResponse<brc20s::AllBalance>,
  BRC20SPool = ApiResponse<brc20s::Pool>,
  BRC20SAllPool = ApiResponse<brc20s::AllPoolInfo>,
  BRC20STxReceipts = ApiResponse<brc20s::TxReceipts>,
  BRC20SBlockReceipts = ApiResponse<brc20s::BlockReceipts>,
  BRC20STransferable = ApiResponse<brc20s::Transferable>,
  BRC20SUserInfo = ApiResponse<brc20s::UserInfo>,
  BRC20SStakedInfo = ApiResponse<brc20s::StakedInfo>,

  OrdOrdInscription = ApiResponse<ord::OrdInscription>,
  OrdOutPointData = ApiResponse<ord::OutPointData>,
  OrdOutPointResult = ApiResponse<ord::OutPointResult>,
  OrdTxInscriptions = ApiResponse<ord::TxInscriptions>,
  OrdBlockInscriptions = ApiResponse<ord::BlockInscriptions>,

  Node = ApiResponse<NodeInfo>
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
    Self::new(1, "ok".to_string(), data)
  }
}
