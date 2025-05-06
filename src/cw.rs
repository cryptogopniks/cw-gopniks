#[cfg(all(feature = "cw-std-v1", not(feature = "cw-std-v2")))]
pub use cosmwasm_std_v1 as cosmwasm_std;
#[cfg(all(feature = "cw-std-v2", not(feature = "cw-std-v1")))]
pub use cosmwasm_std_v2 as cosmwasm_std;

use cosmwasm_std::Env;

pub fn get_block_time(env: &Env) -> u64 {
    env.block.time.seconds()
}

// pub fn get_block_time(env: &cosmwasm_std::Env) -> u64 {
//     #[cfg(feature = "cw-std-v1")]
//     {
//         env.block.time.seconds()
//     }

//     #[cfg(feature = "cw-std-v2")]
//     {
//         env.block.time.seconds()
//     }
// }
