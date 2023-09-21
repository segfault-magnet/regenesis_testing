use fuel_core_chain_config::{CoinConfig, ConsensusConfig, ContractConfig, MessageConfig};
use fuel_core_types::fuel_tx::ConsensusParameters;
use fuel_core_types::fuel_types::BlockHeight;
use fuel_core_types::fuel_vm::GasCosts;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserializer;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

trait ProcessEntry {
    fn process_chain_name(&self, arg: String);
    fn process_block_gas_limit(&self, arg: u64);
    fn process_transaction_parameters(&self, arg: ConsensusParameters);
    fn process_gas_costs(&self, arg: GasCosts);
    fn process_consensus(&self, arg: ConsensusConfig);

    fn process_coin(&self, coin: CoinConfig);
    fn process_contract_config(&self, coin: ContractConfig);
    fn process_message_config(&self, coin: MessageConfig);
    fn process_block_height(&self, block_height: BlockHeight);

    fn clone_me(&self) -> Box<dyn ProcessEntry>;
}

struct ChainConfigVisitor {
    callback: Box<dyn ProcessEntry>,
}

struct StateConfigVisitor {
    callback: Box<dyn ProcessEntry>,
}

struct CoinDeser {
    callback: Box<dyn ProcessEntry>,
}

impl<'de> DeserializeSeed<'de> for CoinDeser {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(CoinVisitor {
            callback: self.callback,
        })
    }
}

struct CoinVisitor {
    callback: Box<dyn ProcessEntry>,
}
impl<'de> Visitor<'de> for CoinVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "CoinVisitor")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(coin) = seq.next_element::<CoinConfig>()? {
            self.callback.process_coin(coin);
        }

        Ok(())
    }
}

struct MessageDeser {
    callback: Box<dyn ProcessEntry>,
}

impl<'de> DeserializeSeed<'de> for MessageDeser {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(MessageVisitor {
            callback: self.callback,
        })
    }
}

struct MessageVisitor {
    callback: Box<dyn ProcessEntry>,
}
impl<'de> Visitor<'de> for MessageVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "MessageVisitor")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(message) = seq.next_element::<MessageConfig>()? {
            self.callback.process_message_config(message);
        }

        Ok(())
    }
}

impl<'de> Visitor<'de> for StateConfigVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "StateConfigVisitor")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<String>().unwrap() {
            match key.as_str() {
                "coins" => {
                    map.next_value_seed(CoinDeser {
                        callback: self.callback.clone_me(),
                    })?;
                }
                "contracts" => {
                    map.next_value_seed(ContractDeser {
                        callback: self.callback.clone_me(),
                    })?;
                }
                "messages" => {
                    map.next_value_seed(MessageDeser {
                        callback: self.callback.clone_me(),
                    })?;
                }
                "height" => {
                    let height_str = map.next_value::<String>()?;
                    let height = BlockHeight::from_str(&height_str).unwrap();
                    self.callback.process_block_height(height);
                }
                _ => {
                    todo!("See about unexpected keys")
                }
            }
        }
        Ok(())
    }
}

struct ContractDeser {
    callback: Box<dyn ProcessEntry>,
}

impl<'de> DeserializeSeed<'de> for ContractDeser {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ContractVisitor {
            callback: self.callback,
        })
    }
}

struct ContractVisitor {
    callback: Box<dyn ProcessEntry>,
}
impl<'de> Visitor<'de> for ContractVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "ContractVisitor")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(contract) = seq.next_element::<ContractConfig>()? {
            self.callback.process_contract_config(contract);
        }

        Ok(())
    }
}

struct InitialStateDeser {
    callback: Box<dyn ProcessEntry>,
}

impl<'de> DeserializeSeed<'de> for InitialStateDeser {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StateConfigVisitor {
            callback: self.callback,
        })
    }
}

impl<'de> Visitor<'de> for ChainConfigVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "ChainConfigVisitor")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = MapAccess::next_key::<String>(&mut map)? {
            match key.as_str() {
                "chain_name" => {
                    let chain_name = map.next_value().unwrap();
                    self.callback.process_chain_name(chain_name);
                }
                "block_gas_limit" => {
                    let block_gas_limit = map.next_value().unwrap();
                    self.callback.process_block_gas_limit(block_gas_limit);
                }
                "initial_state" => {
                    let initial_state = map.next_value_seed(InitialStateDeser {
                        callback: self.callback.clone_me(),
                    })?;
                }
                "transaction_parameters" => {
                    let transaction_parameters = map.next_value().unwrap();
                    self.callback
                        .process_transaction_parameters(transaction_parameters);
                }
                "gas_costs" => {
                    let gas_costs = map.next_value().unwrap();
                    self.callback.process_gas_costs(gas_costs);
                }
                "consensus" => {
                    let consensus = map.next_value().unwrap();
                    self.callback.process_consensus(consensus);
                }
                unexpected => {
                    eprintln!("Didn't expect key: {unexpected}");
                    todo!("We should either error here or read the type using the Any deserializator that will ignore the read value");
                }
            };
        }
        Ok(())
    }
}

#[derive(Clone)]
struct PrintingCallback;

impl ProcessEntry for PrintingCallback {
    fn process_chain_name(&self, arg: String) {
        // eprintln!("fn process_chain_name(&self, arg:{}) ", arg);
    }

    fn process_block_gas_limit(&self, arg: u64) {
        // eprintln!("fn process_block_gas_limit(&self, arg:{}) ", arg);
    }

    fn process_transaction_parameters(&self, arg: ConsensusParameters) {
        // eprintln!("fn process_transaction_parameters(&self, arg:{:?}) ", arg);
    }

    fn process_gas_costs(&self, arg: GasCosts) {
        // eprintln!("fn process_gas_costs(&self, arg:{:?}) ", arg);
    }

    fn process_consensus(&self, arg: ConsensusConfig) {
        // eprintln!("fn process_consensus(&self, arg:{:?}) ", arg);
    }

    fn process_coin(&self, coin: CoinConfig) {
        // eprintln!("fn process_coin(&self, coin:{:?}) ", coin);
    }

    fn process_contract_config(&self, contract: ContractConfig) {
        // eprintln!("fn process_contract_config(&self, coin:{:?}) ", contract);
    }

    fn process_message_config(&self, arg: MessageConfig) {
        // eprintln!("fn process_message_config(&self, coin:{:?}) ", arg);
    }

    fn process_block_height(&self, block_height: BlockHeight) {
        // eprintln!(
        // "fn process_block_height(&self, block_height:{}) ",
        // block_height
        // );
    }

    fn clone_me(&self) -> Box<dyn ProcessEntry> {
        Box::new(self.clone())
    }
}

fn main() {
    let file = File::open("snapshot.json").unwrap();
    let reader = BufReader::new(file);
    let callback = Box::new(PrintingCallback {});
    let visitor = ChainConfigVisitor { callback };
    serde_json::Deserializer::from_reader(reader)
        .deserialize_map(visitor)
        .unwrap();
}
