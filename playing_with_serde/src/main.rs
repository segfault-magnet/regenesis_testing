use fuel_core_chain_config::{CoinConfig, ConsensusConfig, ContractConfig, MessageConfig};
use fuel_core_types::fuel_tx::ConsensusParameters;
use fuel_core_types::fuel_types::BlockHeight;
use fuel_core_types::fuel_vm::GasCosts;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Deserializer;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::mpsc::SyncSender;

trait ProcessEntry {
    fn process(&self, event: Event);
    fn clone_me(&self) -> Box<dyn ProcessEntry>;
}

struct ChainConfigVisitor {
    callback: EventHandler,
}

struct StateConfigVisitor {
    callback: EventHandler,
}

struct CoinDeser {
    callback: EventHandler,
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
    callback: EventHandler,
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
            self.callback.process(Event::CoinConfig(coin))
        }

        Ok(())
    }
}

struct MessageDeser {
    callback: EventHandler,
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
    callback: EventHandler,
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
            self.callback.process(Event::MessageConfig(message))
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
                        callback: self.callback.clone(),
                    })?;
                }
                "contracts" => {
                    map.next_value_seed(ContractDeser {
                        callback: self.callback.clone(),
                    })?;
                }
                "messages" => {
                    map.next_value_seed(MessageDeser {
                        callback: self.callback.clone(),
                    })?;
                }
                "height" => {
                    let height_str = map.next_value::<String>()?;
                    let height = BlockHeight::from_str(&height_str).unwrap();
                    self.callback.process(Event::BlockHeight(height))
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
    callback: EventHandler,
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
    callback: EventHandler,
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
            self.callback.process(Event::ContractConfig(contract))
        }

        Ok(())
    }
}

struct InitialStateDeser {
    callback: EventHandler,
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
                    self.callback.process(Event::ChainName(chain_name))
                }
                "block_gas_limit" => {
                    let block_gas_limit = map.next_value().unwrap();
                    self.callback.process(Event::BlockGasLimit(block_gas_limit))
                }
                "initial_state" => {
                    map.next_value_seed(InitialStateDeser {
                        callback: self.callback.clone(),
                    })?;
                }
                "transaction_parameters" => {
                    let transaction_parameters = map.next_value().unwrap();
                    self.callback
                        .process(Event::ConsensusParameters(transaction_parameters))
                }
                "gas_costs" => {
                    let gas_costs = map.next_value().unwrap();
                    self.callback.process(Event::GasCosts(gas_costs))
                }
                "consensus" => {
                    let consensus = map.next_value().unwrap();
                    self.callback.process(Event::ConsensusConfig(consensus))
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
struct EventHandler {
    sender: SyncSender<Event>,
}

#[derive(Debug)]
enum Event {
    ChainName(String),
    BlockGasLimit(u64),
    ConsensusParameters(ConsensusParameters),
    GasCosts(GasCosts),
    ConsensusConfig(ConsensusConfig),
    CoinConfig(CoinConfig),
    ContractConfig(ContractConfig),
    MessageConfig(MessageConfig),
    BlockHeight(BlockHeight),
}

impl EventHandler {
    fn process(&self, event: Event) {
        self.sender.send(event).unwrap()
    }
}

fn main() {
    let file = File::open("snapshot.json").unwrap();
    let reader = BufReader::new(file);
    let (tx, rx) = std::sync::mpsc::sync_channel(100000);
    let callback = EventHandler { sender: tx };
    let visitor = ChainConfigVisitor { callback };

    let handle = std::thread::spawn(|| {
        serde_json::Deserializer::from_reader(reader)
            .deserialize_map(visitor)
            .unwrap();
    });

    handle.join().unwrap();

    while let Ok(el) = rx.recv() {
        eprintln!("{el:?}")
    }
}
