use std::rc::Rc;

use crate::{
    block_chain::{Block, BlockChain},
    chain_store::HeadRef,
    garden::GardenPlot,
    reducers, Action, Hash,
};

pub struct Store {
    pub block_chain: BlockChain<Action>,
    pub head_ref: HeadRef,
    pub state: State,
}

impl Store {
    pub fn new() -> Self {
        let mut store = Self {
            block_chain: BlockChain::<Action>::new(),
            head_ref: HeadRef::try_from("my-garden").expect("Failed to create HeadRef"),
            state: State::new(),
        };

        // let mut prev_hash = Hash::empty();
        // for block in store.block_chain.root_iter() {
        //     if prev_hash !==
        //     store.reduce(prev_hash, block);
        //     prev_hash = block.hash;
        // }

        store
    }

    pub fn create_garden_plot(&mut self, name: String) -> (Hash, GardenPlot) {
        let plot = GardenPlot::new(name);
        let block = self.block_chain.add_data(Action::CreatePlot(plot.clone()));
        (block.hash.clone(), plot)
    }

    pub fn reduce_untrusted(&mut self, prev_hash: Hash, block: &Block<Action>) {
        // block.hash()
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
            my_garden: reducers::garden(self.my_garden, event),
        }
    }
}
