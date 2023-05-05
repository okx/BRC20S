use super::*;
use brc20::{Action, InscriptionData};

#[derive(Debug, Clone, Copy)]
pub(super) struct Flotsam {
  inscription_id: InscriptionId,
  offset: u64,
  origin: Origin,
}

#[derive(Debug, Clone, Copy)]
enum Origin {
  New(u64),
  Old(SatPoint),
}

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  flotsam: Vec<Flotsam>,
  index: &'a Index,
  height: u64,
  id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
  value_receiver: &'a mut Receiver<u64>,
  id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  lost_sats: u64,
  next_number: u64,
  number_to_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
  outpoint_to_value: &'a mut Table<'db, 'tx, &'static OutPointValue, u64>,
  reward: u64,
  sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
  satpoint_to_id: &'a mut Table<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
  timestamp: u32,
  value_cache: &'a mut HashMap<OutPoint, u64>,
  tx_cache: &'a mut HashMap<Txid, Transaction>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    index: &'a Index,
    height: u64,
    id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
    value_receiver: &'a mut Receiver<u64>,
    id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
    lost_sats: u64,
    number_to_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
    outpoint_to_value: &'a mut Table<'db, 'tx, &'static OutPointValue, u64>,
    sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
    satpoint_to_id: &'a mut Table<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
    timestamp: u32,
    value_cache: &'a mut HashMap<OutPoint, u64>,
    tx_cache: &'a mut HashMap<Txid, Transaction>,
  ) -> Result<Self> {
    let next_number = number_to_id
      .iter()?
      .rev()
      .map(|(number, _id)| number.value() + 1)
      .next()
      .unwrap_or(0);

    Ok(Self {
      flotsam: Vec::new(),
      index,
      height,
      id_to_satpoint,
      value_receiver,
      id_to_entry,
      lost_sats,
      next_number,
      number_to_id,
      outpoint_to_value,
      reward: Height(height).subsidy(),
      sat_to_inscription_id,
      satpoint_to_id,
      timestamp,
      value_cache,
      tx_cache,
    })
  }

  pub(super) fn index_transaction_inscriptions(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
  ) -> Result<(u64, Vec<InscriptionData>)> {
    let mut inscriptions = Vec::new();
    let mut inscription_collects: Vec<(u64, InscriptionData)> = Vec::new();

    let mut input_value = 0;
    for tx_in in &tx.input {
      if tx_in.previous_output.is_null() {
        input_value += Height(self.height).subsidy();
      } else {
        for (old_satpoint, inscription_id) in
          Index::inscriptions_on_output(self.satpoint_to_id, tx_in.previous_output)?
        {
          inscriptions.push(Flotsam {
            offset: input_value + old_satpoint.offset,
            inscription_id,
            origin: Origin::Old(old_satpoint),
          });

          // 只存铸造铭文后的第一笔交易
          let inscribe_satpoint = SatPoint {
            outpoint: OutPoint::new(inscription_id.txid, inscription_id.index),
            offset: 0,
          };
          if old_satpoint == inscribe_satpoint {
            // 获取铸造铭文交易
            let inscribe_tx = if let Some(t) = self.tx_cache.remove(&inscription_id.txid) {
              t
            } else {
              self
                .index
                .get_transaction_with_retries(inscription_id.txid)?
                .ok_or(anyhow!(
                  "failed to get inscription transaction for {}",
                  inscription_id.txid
                ))?
            };

            inscription_collects.push((
              input_value + old_satpoint.offset,
              InscriptionData {
                txid,
                inscription_id,
                action: Action::Transfer(
                  inscribe_tx
                    .output
                    .get(0)
                    .ok_or(anyhow!("faild to index output for {}", inscription_id.txid))?
                    .script_pubkey
                    .clone(),
                  Script::new(),
                ),
                inscription: Inscription::from_transaction(&inscribe_tx).unwrap(),
              },
            ));
          }
        }

        input_value += if let Some(value) = self.value_cache.remove(&tx_in.previous_output) {
          value
        } else if let Some(value) = self
          .outpoint_to_value
          .remove(&tx_in.previous_output.store())?
        {
          value.value()
        } else {
          self.value_receiver.blocking_recv().ok_or_else(|| {
            anyhow!(
              "failed to get transaction for {}",
              tx_in.previous_output.txid
            )
          })?
        }
      }
    }

    let inscription = Inscription::from_transaction(tx);
    if inscriptions.iter().all(|flotsam| flotsam.offset != 0) && inscription.is_some() {
      inscriptions.push(Flotsam {
        inscription_id: txid.into(),
        offset: 0,
        origin: Origin::New(input_value - tx.output.iter().map(|txout| txout.value).sum::<u64>()),
      });

      inscription_collects.push((
        0,
        InscriptionData {
          txid,
          inscription_id: txid.into(),
          action: Action::Inscribe(Script::new()),
          inscription: inscription.unwrap(),
        },
      ));
      self.tx_cache.insert(txid, tx.to_owned());
    };

    let is_coinbase = tx
      .input
      .first()
      .map(|tx_in| tx_in.previous_output.is_null())
      .unwrap_or_default();

    if is_coinbase {
      inscriptions.append(&mut self.flotsam);
    }

    inscriptions.sort_by_key(|flotsam| flotsam.offset);
    inscription_collects.sort_by_key(|key| key.0);

    let mut inscriptions = inscriptions.into_iter().peekable();
    let mut inscription_collects_iterator = inscription_collects.iter_mut();
    let mut output_value = 0;
    for (vout, tx_out) in tx.output.iter().enumerate() {
      let end = output_value + tx_out.value;

      while let Some(flotsam) = inscriptions.peek() {
        if flotsam.offset >= end {
          break;
        }

        let new_satpoint = SatPoint {
          outpoint: OutPoint {
            txid,
            vout: vout.try_into().unwrap(),
          },
          offset: flotsam.offset - output_value,
        };

        let flotsam = inscriptions.next().unwrap();
        self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;

        let flotsam_inscription_id = flotsam.clone();
        let value = inscription_collects_iterator
          .find(|key: &&mut (u64, InscriptionData)| {
            key.1.get_inscription_id() == flotsam_inscription_id.inscription_id
          })
          .ok_or(anyhow!(
            "failed to find inscription data for {}",
            flotsam.inscription_id
          ))?;

        let mut action = value.1.get_action();
        action.set_to(tx_out.script_pubkey.clone());
        value.1.set_action(action);
      }

      output_value = end;

      self.value_cache.insert(
        OutPoint {
          vout: vout.try_into().unwrap(),
          txid,
        },
        tx_out.value,
      );
    }
    let (_, inscription_collects): (Vec<u64>, Vec<InscriptionData>) =
      inscription_collects.into_iter().unzip();

    if is_coinbase {
      for flotsam in inscriptions {
        let new_satpoint = SatPoint {
          outpoint: OutPoint::null(),
          offset: self.lost_sats + flotsam.offset - output_value,
        };
        self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
      }

      Ok((self.reward - output_value, inscription_collects))
    } else {
      self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
        offset: self.reward + flotsam.offset - output_value,
        ..flotsam
      }));
      self.reward += input_value - output_value;
      Ok((0, inscription_collects))
    }
  }

  fn update_inscription_location(
    &mut self,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    flotsam: Flotsam,
    new_satpoint: SatPoint,
  ) -> Result {
    let inscription_id = flotsam.inscription_id.store();

    match flotsam.origin {
      Origin::Old(old_satpoint) => {
        self.satpoint_to_id.remove(&old_satpoint.store())?;
      }
      Origin::New(fee) => {
        self
          .number_to_id
          .insert(&self.next_number, &inscription_id)?;

        let mut sat = None;
        if let Some(input_sat_ranges) = input_sat_ranges {
          let mut offset = 0;
          for (start, end) in input_sat_ranges {
            let size = end - start;
            if offset + size > flotsam.offset {
              let n = start + flotsam.offset - offset;
              self.sat_to_inscription_id.insert(&n, &inscription_id)?;
              sat = Some(Sat(n));
              break;
            }
            offset += size;
          }
        }

        self.id_to_entry.insert(
          &inscription_id,
          &InscriptionEntry {
            fee,
            height: self.height,
            number: self.next_number,
            sat,
            timestamp: self.timestamp,
          }
          .store(),
        )?;

        self.next_number += 1;
      }
    }

    let new_satpoint = new_satpoint.store();

    self.satpoint_to_id.insert(&new_satpoint, &inscription_id)?;
    self.id_to_satpoint.insert(&inscription_id, &new_satpoint)?;

    Ok(())
  }
}
