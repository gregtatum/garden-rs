use std::rc::Rc;

use crate::{
    block_chain::{Block, BlockChain},
    chain_store::HeadRef,
    garden::GardenPlot,
    reducers, Action, ChainStore, Hash,
};

pub struct Store {
    pub block_chain: BlockChain<Action>,
    pub state: State,
}

impl Store {
    pub fn new(chain_store: &mut dyn ChainStore<Action>) -> Self {
        let mut store = Self {
            block_chain: BlockChain::<Action>::new(),
            state: State::new(),
        };

        store
            .load_untrusted_chain_store(chain_store)
            .expect("The block chain failed to reduce");

        store
    }

    pub fn create_garden_plot(&mut self, name: String) -> (Hash, GardenPlot) {
        let plot = GardenPlot::new(name);
        let block = self.block_chain.add_data(Action::CreatePlot(plot.clone()));
        (block.hash.clone(), plot)
    }

    pub fn load_untrusted_chain_store(
        &mut self,
        chain_store: &mut dyn ChainStore<Action>,
    ) -> Result<(), ()> {
        let mut prev_hash = Hash::empty();
        // TODO

        // for block in self.store.block_chain.root_iter() {
        //     if prev_hash != block.hash {
        //         return Err(());
        //     }
        //     if block.payload.hash() != block.hash {
        //         return Err(());
        //     }
        //     self.state = store.reduce(block.payload);
        //     prev_hash = block.hash;
        // }
        Ok(())
    }
}

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
