use pinocchio::{
    AccountView, Address, ProgramResult, cpi::{Seed, Signer}, error::ProgramError, sysvars::{Sysvar, rent::Rent}
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use bytemuck::{Pod, Zeroable};
use pinocchio_log::logger::Logger;

use crate::state::Escrow;

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct InitData {
    amount_to_receive: u64,
    amount_to_give: u64,
}

impl InitData {
    pub const LEN: usize = core::mem::size_of::<InitData>();
}

pub fn process_make_instruction(
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {

    let [maker, mint_a, mint_b, escrow_acc, maker_ata, escrow_ata, system_program, token_program, _associated_token_program @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // validation
    let (escrow_bump, parsed_data) = {
        let maker_addr = maker.address();
        let mint_a_addr = mint_a.address();

        if data.len() < 1 + InitData::LEN {
            return Err(ProgramError::InvalidInstructionData);
        } else if !escrow_acc.is_data_empty() {
            return Err(ProgramError::InvalidAccountData);
        } else if !maker.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata)?;
        if maker_ata_state.owner() != maker_addr {
            return Err(ProgramError::IllegalOwner);
        } else if maker_ata_state.mint() != mint_a_addr {
            return Err(ProgramError::InvalidAccountData);
        }

        let bump = data[0];
        let seeds = [b"escrow".as_ref(), maker.address().as_ref(), &[bump]];

        // must supply the bump exactly once—either embedded in the seed bytes or passed separately, but not both and not zero times.
        let expected_escrow_addr = Address::from(derive_address(&seeds, None, &crate::ID.to_bytes()));
        if &expected_escrow_addr != escrow_acc.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let parsed_data = bytemuck::pod_read_unaligned::<InitData>(
        data[1..1 + InitData::LEN]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        (bump, parsed_data)
    };

    // init account
    {   
        let escrow_bump = [escrow_bump.to_le()];
        let escrow_seeds = [Seed::from(b"escrow"), Seed::from(maker.address().as_array()), Seed::from(&escrow_bump)];
        let escrow_as_signer = Signer::from(&escrow_seeds);

        CreateAccount {
            from: maker,
            to: escrow_acc,
            lamports: Rent::get()?.try_minimum_balance(Escrow::LEN)?,
            space: Escrow::LEN as u64,
            owner: &crate::ID,
        }.invoke_signed(&[escrow_as_signer.clone()])?;

        let mut escrow_data = escrow_acc.try_borrow_mut()?;
        let escrow = Escrow::load_mut(&mut *escrow_data)?;
        
        escrow.initialize(
            *maker.address().as_array(),
            *mint_a.address().as_array(),
            *mint_b.address().as_array(),
            parsed_data.amount_to_receive.to_le_bytes(),
            parsed_data.amount_to_give.to_le_bytes(),
            escrow_bump[0],
        );
    }

    // cpis
    pinocchio_associated_token_account::instructions::Create {
        funding_account: maker,
        account: escrow_ata,
        wallet: escrow_acc, // should be dropped
        mint: mint_a,
        token_program: token_program,
        system_program: system_program,
    }.invoke()?;

    pinocchio_token::instructions::Transfer {
        from: maker_ata,
        to: escrow_ata,
        authority: maker,
        amount: parsed_data.amount_to_give,
    }.invoke()?;

    Ok(())
}