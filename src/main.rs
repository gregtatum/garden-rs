mod block_chain;
mod hash;

use block_chain::BlockChain;

fn main() {
    let mut block_chain = BlockChain::new(3);

    let start = std::time::Instant::now();
    block_chain.add_data("First block".into());
    println!("Timing: {:?}", start.elapsed());
    block_chain.add_data("Second block".into());
    println!("Timing: {:?}", start.elapsed());
    block_chain.add_data("Third block".into());
    println!("Timing: {:?}", start.elapsed());

    println!("{:#?}", block_chain);

    for (block_index, block) in block_chain.blocks.iter().enumerate() {
        println!(
            "Block {} {:?}",
            block_index,
            match std::str::from_utf8(&block.payload.data) {
                Ok(string) => string,
                Err(_) => "invalid utf8",
            }
        )
    }
}
