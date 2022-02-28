use crate::{
    block_chain::SerializedBytes,
    game::{
        game_state::{GAME_H, GAME_W},
        primitives::{BBox, Position, Size},
    },
};
use bincode;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use uuid::Uuid;

/// Create a garden plot.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Hash)]
pub struct GardenPlot {
    pub uuid: Uuid,
    pub name: String,
}

impl SerializedBytes for GardenPlot {
    fn serialized_bytes(&self) -> Cow<[u8]> {
        Cow::from(bincode::serialize(self).expect("Unable to serialize GardenPlot."))
    }
}

impl GardenPlot {
    pub fn new(name: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
        }
    }

    pub fn get_dimensions() -> (u16, u16) {
        (10, 10)
    }

    pub fn get_default_bbox() -> BBox<i32> {
        let margin = 10;
        BBox {
            top_left: Position::new(margin, margin),
            size: Size::new(GAME_W - margin * 2, GAME_H - margin * 2),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        block_chain::{BlockChain, BlockData},
        Action, ChainAction,
    };
    use insta::assert_display_snapshot;
    use serde_json::Value;
    use std::collections::HashMap;

    fn serialize_for_test<T: BlockData + Serialize>(
        block_chain: &BlockChain<T>,
    ) -> String {
        let mut value = serde_json::to_value(&block_chain.blocks)
            .expect("Unable to convert blockchain to value.");
        make_test_safe(&mut value);
        serde_json::to_string_pretty(&value)
            .expect("Failed to run serde_json::to_string_pretty")
    }

    struct InternedString {
        i: usize,
        map: HashMap<String, String>,
        tag: &'static str,
    }

    impl InternedString {
        pub fn new(tag: &'static str) -> Self {
            Self {
                i: 0,
                map: HashMap::new(),
                tag,
            }
        }

        pub fn get(&mut self, value: &str) -> String {
            if let Some(v) = self.map.get(value) {
                return v.into();
            }
            self.i += 1;
            self.map
                .insert(value.into(), format!("({}:{})", self.tag, self.i));
            self.map.get(value).unwrap().into()
        }
    }

    /// Removes all arbitrary data from a blockchain.
    fn make_test_safe(value: &mut Value) {
        let mut hashes = InternedString::new("Hash");
        let mut uuids = InternedString::new("UUID");

        if let Value::Array(ref mut blocks) = value {
            for block in blocks {
                if let Value::Object(ref mut block) = block {
                    if let Some(Value::String(ref mut hash)) = block.get_mut("hash") {
                        *hash = hashes.get(hash);
                    }

                    // Strip out the payload.
                    if let Some(Value::Object(ref mut payload)) = block.get_mut("payload")
                    {
                        if let Some(Value::String(ref mut parent)) =
                            payload.get_mut("parent")
                        {
                            *parent = hashes.get(parent);
                        }
                        payload.remove("timestamp");

                        // Anonymize the payload.
                        if let Some(Value::Object(ref mut data)) = payload.get_mut("data")
                        {
                            if let Some(Value::Object(ref mut create_plot)) =
                                data.get_mut("CreatePlot")
                            {
                                if let Some(Value::String(ref mut uuid)) =
                                    create_plot.get_mut("uuid")
                                {
                                    *uuid = uuids.get(uuid);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_create_garden_plot() {
        let mut block_chain = BlockChain::<ChainAction>::new();
        block_chain.add_data(ChainAction::CreatePlot(GardenPlot::new(
            "Greg's plot".into(),
        )));
        assert_display_snapshot!(serialize_for_test(&block_chain), @r###"
        [
          {
            "hash": "(Hash:1)",
            "payload": {
              "data": {
                "CreatePlot": {
                  "name": "Greg's plot",
                  "uuid": "(UUID:1)"
                }
              },
              "parent": "(Hash:2)"
            }
          }
        ]
        "###);
    }
}
