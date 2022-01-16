use garden::block_chain::BlockChain;

// This example shows how to add blocks to the blockchain using proof of work.

fn main() {
    // The number represents the proof of work. The work load increases according to:
    // 2^(n*8), so it increases quite dramatically.
    let mut block_chain = BlockChain::<String>::new(4);

    // Time how long it takes.
    let start = std::time::Instant::now();

    block_chain.add_data("First block".into());
    println!("Timing: {:?}", start.elapsed());

    block_chain.add_data("Second block".into());
    println!("Timing: {:?}", start.elapsed());

    block_chain.add_data("Third block".into());
    println!("Timing: {:?}", start.elapsed());

    println!("{:#?}", block_chain);

    for (block_index, block) in block_chain.blocks.iter().enumerate() {
        println!("Block {} {:?}", block_index, block.payload.data)
    }
}
