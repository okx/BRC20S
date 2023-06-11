use super::*;
use brc20::deserialize_brc20_operation;
use brc20::{Action, InscriptionData, Operation};

#[derive(Debug, Clone)]
pub(super) struct Flotsam {
  inscription_id: InscriptionId,
  offset: u64,
  origin: Origin,
}

#[derive(Debug, Clone)]
enum Origin {
  New {
    fee: u64,
    cursed: bool,
    unbound: bool,
  },
  Old {
    old_satpoint: SatPoint,
  },
}

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  flotsam: Vec<Flotsam>,
  index: &'a Index,
  height: u64,
  id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
  id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  pub(super) lost_sats: u64,
  next_cursed_number: i64,
  next_number: i64,
  number_to_id: &'a mut Table<'db, 'tx, i64, &'static InscriptionIdValue>,
  outpoint_to_entry: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
  reward: u64,
  sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
  satpoint_to_id: &'a mut Table<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
  timestamp: u32,
  pub(super) unbound_inscriptions: u64,
  tx_cache: &'a mut HashMap<Txid, Transaction>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    index: &'a Index,
    height: u64,
    id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
    id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
    lost_sats: u64,
    number_to_id: &'a mut Table<'db, 'tx, i64, &'static InscriptionIdValue>,
    outpoint_to_entry: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
    sat_to_inscription_id: &'a mut Table<'db, 'tx, u64, &'static InscriptionIdValue>,
    satpoint_to_id: &'a mut Table<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
    timestamp: u32,
    unbound_inscriptions: u64,
    tx_cache: &'a mut HashMap<Txid, Transaction>,
  ) -> Result<Self> {
    let next_cursed_number = number_to_id
      .iter()?
      .map(|(number, _id)| number.value() - 1)
      .next()
      .unwrap_or(-1);

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
      id_to_entry,
      lost_sats,
      next_cursed_number,
      next_number,
      number_to_id,
      outpoint_to_entry,
      reward: Height(height).subsidy(),
      sat_to_inscription_id,
      satpoint_to_id,
      timestamp,
      unbound_inscriptions,
      tx_cache,
    })
  }

  pub(super) fn index_transaction_inscriptions(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
  ) -> Result<Vec<InscriptionData>> {
    let mut inscriptions_collector = Vec::new();
    let mut new_inscriptions = Inscription::from_transaction(tx).into_iter().peekable();
    let mut floating_inscriptions = Vec::new();
    let mut inscribed_offsets = BTreeMap::new();
    let mut input_value = 0;
    let mut id_counter = 0;

    for (input_index, tx_in) in tx.input.iter().enumerate() {
      // skip subsidy since no inscriptions possible
      if tx_in.previous_output.is_null() {
        input_value += Height(self.height).subsidy();
        continue;
      }

      // find existing inscriptions on input aka transfers of inscriptions
      for (old_satpoint, inscription_id) in
        Index::inscriptions_on_output(self.satpoint_to_id, tx_in.previous_output)?
      {
        let offset = input_value + old_satpoint.offset;
        floating_inscriptions.push(Flotsam {
          offset,
          inscription_id,
          origin: Origin::Old { old_satpoint },
        });

        inscribed_offsets.insert(offset, inscription_id);
        let inscribe_satpoint = SatPoint {
          outpoint: OutPoint::new(inscription_id.txid, 0),
          offset: 0,
        };

        if old_satpoint == inscribe_satpoint {
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
          if let Ok(Operation::Transfer(transfer)) = deserialize_brc20_operation(
            Inscription::from_transaction(&inscribe_tx)
              .get(0)
              .unwrap()
              .inscription
              .clone(),
            true,
          ) {
            inscriptions_collector.push((
              input_value + old_satpoint.offset,
              InscriptionData {
                txid,
                inscription_id,
                old_satpoint,
                new_satpoint: None,
                from_script: ScriptKey::from_script(
                  &inscribe_tx
                    .output
                    .get(old_satpoint.outpoint.vout as usize)
                    .ok_or(anyhow!(
                      "failed to find output {} for {}",
                      old_satpoint.outpoint.vout,
                      inscription_id.txid
                    ))?
                    .script_pubkey
                    .clone(),
                  self.index.get_chain_network(),
                ),
                to_script: None,
                action: Action::Transfer(transfer),
              },
            ))
          }
        };
      }

      let offset = input_value;

      input_value +=
        Index::get_transaction_output_by_outpoint(self.outpoint_to_entry, &tx_in.previous_output)
          .map(|txout| txout.value)?;

      // go through all inscriptions in this input
      while let Some(inscription) = new_inscriptions.peek() {
        if inscription.tx_in_index != u32::try_from(input_index).unwrap() {
          break;
        }

        let initial_inscription_is_cursed = inscribed_offsets
          .get(&offset)
          .and_then(
            |inscription_id| match self.id_to_entry.get(&inscription_id.store()) {
              Ok(option) => option.map(|entry| InscriptionEntry::load(entry.value()).number < 0),
              Err(_) => None,
            },
          )
          .unwrap_or(false);

        let cursed = !initial_inscription_is_cursed
          && (inscription.tx_in_index != 0
            || inscription.tx_in_offset != 0
            || inscribed_offsets.contains_key(&offset));

        // In this first part of the cursed inscriptions implementation we ignore reinscriptions.
        // This will change once we implement reinscriptions.
        let unbound = inscribed_offsets.contains_key(&offset)
          || inscription.tx_in_offset != 0
          || input_value == 0;

        let inscription_id = InscriptionId {
          txid,
          index: id_counter,
        };

        floating_inscriptions.push(Flotsam {
          inscription_id,
          offset,
          origin: Origin::New {
            fee: 0,
            cursed,
            unbound,
          },
        });

        if !cursed && !unbound {
          if let Ok(operation) = deserialize_brc20_operation(inscription.inscription.clone(), false)
          {
            let from_script = self.get_previous_output_script(
              tx.input
                .get(0)
                .ok_or(anyhow!("failed to find input {} for {}", 0, txid))?
                .previous_output,
            )?;
            inscriptions_collector.push((
              0,
              InscriptionData {
                txid,
                inscription_id: txid.into(),
                old_satpoint: SatPoint {
                  outpoint: tx.input.get(0).unwrap().previous_output,
                  offset: 0,
                },
                new_satpoint: None,
                from_script: ScriptKey::from_script(&from_script, self.index.get_chain_network()),
                to_script: None,
                action: Action::Inscribe(operation),
              },
            ))
          };
          self.tx_cache.insert(txid, tx.to_owned());
        }

        new_inscriptions.next();
        id_counter += 1;
      }
    }

    // still have to normalize over inscription size
    let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<u64>();
    let mut floating_inscriptions = floating_inscriptions
      .into_iter()
      .map(|flotsam| {
        if let Flotsam {
          inscription_id,
          offset,
          origin:
            Origin::New {
              fee: _,
              cursed,
              unbound,
            },
        } = flotsam
        {
          Flotsam {
            inscription_id,
            offset,
            origin: Origin::New {
              fee: (input_value - total_output_value) / u64::from(id_counter),
              cursed,
              unbound,
            },
          }
        } else {
          flotsam
        }
      })
      .collect::<Vec<Flotsam>>();

    let is_coinbase = tx
      .input
      .first()
      .map(|tx_in| tx_in.previous_output.is_null())
      .unwrap_or_default();

    if is_coinbase {
      floating_inscriptions.append(&mut self.flotsam);
    }

    floating_inscriptions.sort_by_key(|flotsam| flotsam.offset);
    inscriptions_collector.sort_by_key(|key| key.0);

    let mut inscriptions = floating_inscriptions.into_iter().peekable();
    let mut output_value = 0;
    for (vout, tx_out) in tx.output.iter().enumerate() {
      let end = output_value + tx_out.value;

      while let Some(flotsam) = inscriptions.peek().cloned() {
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

        self.update_inscription_location(
          input_sat_ranges,
          inscriptions.next().unwrap(),
          new_satpoint,
        )?;

        if let Some(inscription_data) = inscriptions_collector
          .iter_mut()
          .find(|key: &&mut (u64, InscriptionData)| key.1.inscription_id == flotsam.inscription_id)
          .map(|value| &mut value.1)
        {
          inscription_data.to_script = Some(ScriptKey::from_script(
            &tx_out.script_pubkey,
            self.index.get_chain_network(),
          ));
          inscription_data.new_satpoint = Some(new_satpoint);
        }
      }

      output_value = end;
    }
    let (_, collects): (Vec<u64>, Vec<InscriptionData>) =
      inscriptions_collector.into_iter().unzip();

    if is_coinbase {
      for flotsam in inscriptions {
        let new_satpoint = SatPoint {
          outpoint: OutPoint::null(),
          offset: self.lost_sats + flotsam.offset - output_value,
        };
        self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
      }
      self.lost_sats += self.reward - output_value;
      Ok(Vec::new())
    } else {
      self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
        offset: self.reward + flotsam.offset - output_value,
        ..flotsam
      }));
      self.reward += input_value - output_value;
      Ok(collects)
    }
  }

  fn update_inscription_location(
    &mut self,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    flotsam: Flotsam,
    new_satpoint: SatPoint,
  ) -> Result {
    let inscription_id = flotsam.inscription_id.store();
    let unbound = match flotsam.origin {
      Origin::Old { old_satpoint } => {
        self.satpoint_to_id.remove(&old_satpoint.store())?;

        false
      }
      Origin::New {
        fee,
        cursed,
        unbound,
      } => {
        let number = if cursed {
          let next_cursed_number = self.next_cursed_number;
          self.next_cursed_number -= 1;

          next_cursed_number
        } else {
          let next_number = self.next_number;
          self.next_number += 1;

          next_number
        };

        self.number_to_id.insert(number, &inscription_id)?;

        let sat = if unbound {
          None
        } else {
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
          sat
        };

        self.id_to_entry.insert(
          &inscription_id,
          &InscriptionEntry {
            fee,
            height: self.height,
            number,
            sat,
            timestamp: self.timestamp,
          }
          .store(),
        )?;

        unbound
      }
    };

    let satpoint = if unbound {
      let new_unbound_satpoint = SatPoint {
        outpoint: unbound_outpoint(),
        offset: self.unbound_inscriptions,
      };
      self.unbound_inscriptions += 1;
      new_unbound_satpoint.store()
    } else {
      new_satpoint.store()
    };

    self.satpoint_to_id.insert(&satpoint, &inscription_id)?;
    self.id_to_satpoint.insert(&inscription_id, &satpoint)?;

    Ok(())
  }

  fn get_previous_output_script(&self, outpoint: OutPoint) -> Result<Script> {
    let tx = self
      .index
      .get_transaction_with_retries(outpoint.txid)?
      .ok_or(anyhow!("failed to get transaction for {}", outpoint.txid))?;
    Ok(
      tx.output
        .get(outpoint.vout as usize)
        .ok_or(anyhow!(
          "failed to get output {} for {}",
          outpoint.vout,
          outpoint.txid
        ))?
        .script_pubkey
        .clone(),
    )
  }
}
