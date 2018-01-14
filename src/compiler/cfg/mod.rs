pub mod block;
pub mod instruction;

use std::collections::HashMap;
use self::block::*;

#[derive(Debug)]
pub struct CFG {
    block_map: HashMap<String, Block>,
    adjacency_list: HashMap<String, Vec<String>>
}
