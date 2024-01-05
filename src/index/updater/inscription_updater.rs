use {
  super::*,
  crate::okx::datastore::ord::operation::{Action, InscriptionOp},
  inscription::Curse,
};

#[derive(Debug, Clone)]
pub(super) struct Flotsam {
  txid: Txid,
  inscription_id: InscriptionId,
  offset: u64,
  old_satpoint: SatPoint,
  origin: Origin,
}

#[derive(Debug, Clone)]
enum Origin {
  New {
    cursed: bool,
    fee: u64,
    parent: Option<InscriptionId>,
    inscription: Inscription,
    unbound: bool,
  },
  Old,
}

pub(super) struct InscriptionUpdater<'a, 'db, 'tx> {
  flotsam: Vec<Flotsam>,
  pub(super) operations: HashMap<Txid, Vec<InscriptionOp>>,
  height: u64,
  id_to_children:
    &'a mut MultimapTable<'db, 'tx, &'static InscriptionIdValue, &'static InscriptionIdValue>,
  id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
  tx_out_receiver: &'a mut Receiver<TxOut>,
  id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
  pub(super) lost_sats: u64,
  pub(super) next_cursed_number: i64,
  pub(super) next_number: i64,
  number_to_id: &'a mut Table<'db, 'tx, i64, &'static InscriptionIdValue>,
  outpoint_to_entry: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
  reward: u64,
  reinscription_id_to_seq_num: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, u64>,
  sat_to_inscription_id: &'a mut MultimapTable<'db, 'tx, u64, &'static InscriptionIdValue>,
  satpoint_to_id:
    &'a mut MultimapTable<'db, 'tx, &'static SatPointValue, &'static InscriptionIdValue>,
  timestamp: u32,
  pub(super) unbound_inscriptions: u64,
  tx_out_cache: &'a mut HashMap<OutPoint, TxOut>,
}

impl<'a, 'db, 'tx> InscriptionUpdater<'a, 'db, 'tx> {
  pub(super) fn new(
    height: u64,
    id_to_children: &'a mut MultimapTable<
      'db,
      'tx,
      &'static InscriptionIdValue,
      &'static InscriptionIdValue,
    >,
    id_to_satpoint: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, &'static SatPointValue>,
    tx_out_receiver: &'a mut Receiver<TxOut>,
    id_to_entry: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, InscriptionEntryValue>,
    lost_sats: u64,
    number_to_id: &'a mut Table<'db, 'tx, i64, &'static InscriptionIdValue>,
    outpoint_to_entry: &'a mut Table<'db, 'tx, &'static OutPointValue, &'static [u8]>,
    reinscription_id_to_seq_num: &'a mut Table<'db, 'tx, &'static InscriptionIdValue, u64>,
    sat_to_inscription_id: &'a mut MultimapTable<'db, 'tx, u64, &'static InscriptionIdValue>,
    satpoint_to_id: &'a mut MultimapTable<
      'db,
      'tx,
      &'static SatPointValue,
      &'static InscriptionIdValue,
    >,
    timestamp: u32,
    unbound_inscriptions: u64,
    tx_out_cache: &'a mut HashMap<OutPoint, TxOut>,
  ) -> Result<Self> {
    let next_cursed_number = number_to_id
      .iter()?
      .next()
      .and_then(|result| result.ok())
      .map(|(number, _id)| number.value() - 1)
      .unwrap_or(-1);

    let next_number = number_to_id
      .iter()?
      .next_back()
      .and_then(|result| result.ok())
      .map(|(number, _id)| number.value() + 1)
      .unwrap_or(0);

    Ok(Self {
      flotsam: Vec::new(),
      operations: HashMap::new(),
      height,
      id_to_children,
      id_to_satpoint,
      tx_out_receiver,
      id_to_entry,
      lost_sats,
      next_cursed_number,
      next_number,
      number_to_id,
      outpoint_to_entry,
      reward: Height(height).subsidy(),
      reinscription_id_to_seq_num,
      sat_to_inscription_id,
      satpoint_to_id,
      timestamp,
      unbound_inscriptions,
      tx_out_cache,
    })
  }

  pub(super) fn index_transaction_inscriptions(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
  ) -> Result {
    let mut new_inscriptions = Inscription::from_transaction(tx).into_iter().peekable();
    let mut floating_inscriptions = Vec::new();
    let mut inscribed_offsets = BTreeMap::new();
    let mut total_input_value = 0;
    let mut id_counter = 0;

    for (input_index, tx_in) in tx.input.iter().enumerate() {
      // skip subsidy since no inscriptions possible
      if tx_in.previous_output.is_null() {
        total_input_value += Height(self.height).subsidy();
        continue;
      }

      // find existing inscriptions on input (transfers of inscriptions)
      for (old_satpoint, inscription_id) in Index::inscriptions_on_output_ordered(
        self.reinscription_id_to_seq_num,
        self.satpoint_to_id,
        tx_in.previous_output,
      )? {
        let offset = total_input_value + old_satpoint.offset;
        floating_inscriptions.push(Flotsam {
          txid,
          offset,
          old_satpoint,
          inscription_id,
          origin: Origin::Old,
        });

        inscribed_offsets
          .entry(offset)
          .and_modify(|(_id, count)| *count += 1)
          .or_insert((inscription_id, 0));
      }

      let offset = total_input_value;

      // multi-level cache for UTXO set to get to the input amount
      let current_input_value = if let Some(tx_out) = self.tx_out_cache.get(&tx_in.previous_output)
      {
        tx_out.value
      } else if let Some(tx_out) =
        Index::transaction_output_by_outpoint(self.outpoint_to_entry, tx_in.previous_output)?
      {
        tx_out.value
      } else {
        let tx_out = self.tx_out_receiver.blocking_recv().ok_or_else(|| {
          anyhow!(
            "failed to get transaction for {}",
            tx_in.previous_output.txid
          )
        })?;
        self
          .tx_out_cache
          .insert(tx_in.previous_output, tx_out.clone());
        tx_out.value
      };

      total_input_value += current_input_value;

      // go through all inscriptions in this input
      while let Some(inscription) = new_inscriptions.peek() {
        if inscription.tx_in_index != u32::try_from(input_index).unwrap() {
          break;
        }

        let inscription_id = InscriptionId {
          txid,
          index: id_counter,
        };

        let curse = if inscription.inscription.unrecognized_even_field {
          Some(Curse::UnrecognizedEvenField)
        } else if inscription.tx_in_index != 0 {
          Some(Curse::NotInFirstInput)
        } else if inscription.tx_in_offset != 0 {
          Some(Curse::NotAtOffsetZero)
        } else if inscribed_offsets.contains_key(&offset) {
          let seq_num = self.reinscription_id_to_seq_num.len()?;

          let sat = Self::calculate_sat(input_sat_ranges, offset);
          log::info!("processing reinscription {inscription_id} on sat {:?}: sequence number {seq_num}, inscribed offsets {:?}", sat, inscribed_offsets);

          // if reinscription track its ordering
          self
            .reinscription_id_to_seq_num
            .insert(&inscription_id.store(), seq_num)?;

          Some(Curse::Reinscription)
        } else {
          None
        };

        if curse.is_some() {
          log::info!("found cursed inscription {inscription_id}: {:?}", curse);
        }

        let cursed = if let Some(Curse::Reinscription) = curse {
          let first_reinscription = inscribed_offsets
            .get(&offset)
            .map(|(_id, count)| count == &0)
            .unwrap_or(false);

          let initial_inscription_is_cursed = inscribed_offsets
            .get(&offset)
            .and_then(|(inscription_id, _count)| {
              match self.id_to_entry.get(&inscription_id.store()) {
                Ok(option) => option.map(|entry| {
                  let loaded_entry = InscriptionEntry::load(entry.value());
                  loaded_entry.number < 0
                }),
                Err(_) => None,
              }
            })
            .unwrap_or(false);

          log::info!("{inscription_id}: is first reinscription: {first_reinscription}, initial inscription is cursed: {initial_inscription_is_cursed}");

          !(initial_inscription_is_cursed && first_reinscription)
        } else {
          curse.is_some()
        };

        let unbound = current_input_value == 0
          || inscription.tx_in_offset != 0
          || curse == Some(Curse::UnrecognizedEvenField);

        if curse.is_some() || unbound {
          log::info!(
            "indexing inscription {inscription_id} with curse {:?} as cursed {} and unbound {}",
            curse,
            cursed,
            unbound
          );
        }

        floating_inscriptions.push(Flotsam {
          txid,
          old_satpoint: SatPoint {
            outpoint: tx_in.previous_output,
            offset: 0,
          },
          inscription_id,
          offset,
          origin: Origin::New {
            cursed,
            fee: 0,
            parent: inscription.inscription.parent(),
            inscription: inscription.inscription.clone(),
            unbound,
          },
        });

        new_inscriptions.next();
        id_counter += 1;
      }
    }

    let potential_parents = floating_inscriptions
      .iter()
      .map(|flotsam| flotsam.inscription_id)
      .collect::<HashSet<InscriptionId>>();

    for flotsam in &mut floating_inscriptions {
      if let Flotsam {
        origin: Origin::New { parent, .. },
        ..
      } = flotsam
      {
        if let Some(purported_parent) = parent {
          if !potential_parents.contains(purported_parent) {
            *parent = None;
          }
        }
      }
    }

    // still have to normalize over inscription size
    let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<u64>();
    let mut floating_inscriptions = floating_inscriptions
      .into_iter()
      .map(|flotsam| {
        if let Flotsam {
          txid,
          old_satpoint,
          inscription_id,
          offset,
          origin:
            Origin::New {
              cursed,
              fee: _,
              parent,
              inscription,
              unbound,
            },
        } = flotsam
        {
          Flotsam {
            txid,
            old_satpoint,
            inscription_id,
            offset,
            origin: Origin::New {
              fee: (total_input_value - total_output_value) / u64::from(id_counter),
              cursed,
              parent,
              inscription,
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
    let mut inscriptions = floating_inscriptions.into_iter().peekable();

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

        self.update_inscription_location(
          input_sat_ranges,
          inscriptions.next().unwrap(),
          new_satpoint,
        )?;
      }

      output_value = end;

      self.tx_out_cache.insert(
        OutPoint {
          vout: vout.try_into().unwrap(),
          txid,
        },
        tx_out.clone(),
      );
    }

    if is_coinbase {
      for flotsam in inscriptions {
        let new_satpoint = SatPoint {
          outpoint: OutPoint::null(),
          offset: self.lost_sats + flotsam.offset - output_value,
        };
        self.update_inscription_location(input_sat_ranges, flotsam, new_satpoint)?;
      }
      self.lost_sats += self.reward - output_value;
      Ok(())
    } else {
      self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
        offset: self.reward + flotsam.offset - output_value,
        ..flotsam
      }));
      self.reward += total_input_value - output_value;
      Ok(())
    }
  }

  // write tx_out to outpoint_to_entry table
  pub(super) fn flush_cache(&mut self) -> Result {
    for (outpoint, tx_out) in self.tx_out_cache.iter() {
      let mut entry = Vec::new();
      tx_out.consensus_encode(&mut entry)?;
      self
        .outpoint_to_entry
        .insert(&outpoint.store(), entry.as_slice())?;
    }
    Ok(())
  }

  fn calculate_sat(
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    input_offset: u64,
  ) -> Option<Sat> {
    let mut sat = None;
    if let Some(input_sat_ranges) = input_sat_ranges {
      let mut offset = 0;
      for (start, end) in input_sat_ranges {
        let size = end - start;
        if offset + size > input_offset {
          let n = start + input_offset - offset;
          sat = Some(Sat(n));
          break;
        }
        offset += size;
      }
    }
    sat
  }

  fn update_inscription_location(
    &mut self,
    input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    flotsam: Flotsam,
    new_satpoint: SatPoint,
  ) -> Result {
    let inscription_id = flotsam.inscription_id.store();
    let unbound = match flotsam.origin {
      Origin::Old => {
        self
          .satpoint_to_id
          .remove_all(&flotsam.old_satpoint.store())?;
        false
      }
      Origin::New {
        cursed,
        fee,
        parent,
        inscription: _,
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
          Self::calculate_sat(input_sat_ranges, flotsam.offset)
        };

        if let Some(Sat(n)) = sat {
          self.sat_to_inscription_id.insert(&n, &inscription_id)?;
        }

        self.id_to_entry.insert(
          &inscription_id,
          &InscriptionEntry {
            fee,
            height: self.height,
            number,
            parent,
            sat,
            timestamp: self.timestamp,
          }
          .store(),
        )?;

        if let Some(parent) = parent {
          self
            .id_to_children
            .insert(&parent.store(), &inscription_id)?;
        }

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
    self
      .operations
      .entry(flotsam.txid)
      .or_default()
      .push(InscriptionOp {
        txid: flotsam.txid,
        inscription_number: self
          .id_to_entry
          .get(&flotsam.inscription_id.store())?
          .map(|entry| InscriptionEntry::load(entry.value()).number),
        inscription_id: flotsam.inscription_id,
        action: match flotsam.origin {
          Origin::Old => Action::Transfer,
          Origin::New {
            fee: _,
            parent: _,
            inscription,
            cursed,
            unbound,
          } => Action::New {
            cursed,
            unbound,
            inscription,
          },
        },
        old_satpoint: flotsam.old_satpoint,
        new_satpoint: Some(Entry::load(satpoint)),
      });

    self.satpoint_to_id.insert(&satpoint, &inscription_id)?;
    self.id_to_satpoint.insert(&inscription_id, &satpoint)?;

    Ok(())
  }
}
