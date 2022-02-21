use std::rc::Rc;

use crate::{
    block_chain::{Block, BlockChain},
    chain_store::HeadRef,
    garden::GardenPlot,
    reducers, Action, ChainStore, Hash,
};

#[derive(Debug)]
pub struct StateStore {
    pub chain_store: Box<dyn ChainStore<Action>>,
    pub state: State,
}

impl StateStore {
    pub fn new(chain_store: Box<dyn ChainStore<Action>>) -> Self {
        let mut store = Self {
            chain_store,
            state: State::new(),
        };

        store
            .load_untrusted_chain_store()
            .expect("The block chain failed to reduce");

        store
    }

    pub fn create_garden_plot(&mut self, name: String) -> (Hash, GardenPlot) {
        let plot = GardenPlot::new(name);
        let block = self.chain_store.add(Action::CreatePlot(plot.clone()));
        (block.hash.clone(), plot)
    }

    pub fn load_untrusted_chain_store(&mut self) -> Result<(), ()> {
        let mut prev_hash = Hash::empty();

        for block in self.chain_store.iter().rev() {
            if prev_hash != block.hash {
                return Err(());
            }
            if block.payload.hash() != block.hash {
                return Err(());
            }
            prev_hash = block.hash.clone();

            self.state = self.state.reduce(&block.payload.data);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct State {
    my_garden: Option<Rc<GardenPlot>>,
}

impl State {
    pub fn new() -> Self {
        Self { my_garden: None }
    }

    pub fn reduce(&self, event: &Action) -> State {
        State {
            my_garden: reducers::garden(self.my_garden.clone(), event),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::chain_store::FsChainStore;
    use std::path::PathBuf;
    use tempdir::TempDir;

    struct StateStoreTest {
        #[allow(dead_code)] // RAII
        pub tmp_dir: TempDir,
        pub path: PathBuf,
        pub state_store: StateStore,
    }

    impl StateStoreTest {
        pub fn new() -> Self {
            let tmp_dir =
                TempDir::new("example").expect("Failed to create a temp directory");
            let path: PathBuf = tmp_dir.path().into();
            let head_ref =
                HeadRef::try_from("my-garden").expect("Failed to create HeadRef");
            let chain_store = Box::new(
                FsChainStore::<Action>::try_new(path.clone(), head_ref)
                    .expect("Failed to create ChainStore"),
            );
            let state_store = StateStore::new(chain_store);

            Self {
                tmp_dir,
                path,
                state_store,
            }
        }
    }

    #[test]
    fn test_garden() {
        let mut test = StateStoreTest::new();
        let StateStoreTest {
            ref mut state_store,
            ref path,
            ..
        } = test;
        println!("{:#?}", state_store);
    }
}
