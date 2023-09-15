use fuel_core_chain_config::{ChainConfig, ConsensusConfig};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::any::Any;
use std::fs::File;
use std::io::BufReader;
use std::ops::Add;
use std::str::FromStr;

struct OurVisitor;

impl OurVisitor {
    fn send_to_db(arg: &str) {}
}

impl<'de> Visitor<'de> for OurVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("E nesto smo usrali")
    }

    // fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    // where
    //     A: serde::de::SeqAccess<'de>,
    // {
    //     let _ = seq;
    //     Err(serde::de::Error::invalid_type(serde::de::Unexpected::Seq, &self))
    // }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Ok(Some((key))) = map.next_key::<String>() {
            match key.as_str() {
                "chain_name" => {
                    let value = map.next_value::<String>();
                }
                "block_gas_limit" => {
                    let value = map.next_value::<u64>();
                }
                // "initial_state" => map.next_value(),
                _ => panic!("Ej"),
            };
            eprintln!("found key: '{}'", key);
        }

        Ok(())
        // let _ = map;
        // Err(serde::de::Error::invalid_type(
        //     serde::de::Unexpected::Map,
        //     &self,
        // ))
    }
}

fn main() {
    let file = File::open("snapshot.json").unwrap();
    let reader = BufReader::new(file);
    //
    let contents = r#"
    {
      "ismet_key": "value"
    }
    "#;
    let mut u = serde_json::Deserializer::from_reader(reader).into_iter::<Value>();

    for value in u {
        dbg!(value);
    }

    // let visitor = OurVisitor;
    //
    // u.deserialize_map(visitor).unwrap();
}
