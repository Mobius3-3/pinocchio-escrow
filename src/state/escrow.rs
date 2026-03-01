use crate::utils::{impl_len, impl_load};
use wincode::SchemaRead;
use bytemuck::{Pod, Zeroable};
// use pinocchio_log::logger::Logger;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, SchemaRead)]
pub struct Escrow {
    maker: [u8; 32],
    mint_a: [u8; 32],
    mint_b: [u8; 32],
    amount_to_receive: [u8; 8],
    amount_to_give: [u8; 8],
    pub bump: u8,
}

impl_len!(Escrow);
impl_load!(Escrow);

impl Escrow {
    // pub fn to_bytes(&self) -> &[u8; Self::LEN] {
    //     bytemuck::bytes_of(self).try_into().unwrap()
    // }

    pub fn maker(&self) -> [u8; 32] {
        self.maker
    }

    pub fn mint_b(&self) -> [u8; 32] {
        self.mint_b
    }

    pub fn amount_to_receive(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_receive)
    }

    pub fn amount_to_give(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_give)
    }

    pub fn initialize(
        &mut self,
        maker: [u8; 32],
        mint_a: [u8; 32],
        mint_b: [u8; 32],
        amount_to_receive: [u8; 8],
        amount_to_give: [u8; 8],
        bump: u8,
    ) {
        self.maker = maker;
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self.amount_to_receive = amount_to_receive;
        self.amount_to_give = amount_to_give;
        self.bump = bump;
    }
}