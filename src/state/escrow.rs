use crate::utils::{impl_len, impl_load};
use bytemuck::{Pod, Zeroable};
// use pinocchio_log::logger::Logger;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Pod, Zeroable)]
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
    pub fn to_bytes(&self) -> &[u8; Self::LEN] {
        bytemuck::bytes_of(self).try_into().unwrap()
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