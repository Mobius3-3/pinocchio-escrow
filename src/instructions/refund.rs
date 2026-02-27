use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError
};

use crate::state::Escrow;

pub fn process_refund_instruction(
    accounts: &[AccountView],
    _data: &[u8],
) -> ProgramResult {
    let [
        maker,
        mint_a,
        escrow,
        maker_ata,
        escrow_ata,
        _system_program,
        _token_program,
        _associated_token_program@ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let (amount_to_give, bump) = {
        let escrow_state = Escrow::from_account_info(&escrow)?;
        let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata)?;

        if maker_ata_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        
        if escrow_state.maker() != *maker.address()|| escrow_state.mint_a() != *mint_a.address() || maker_ata_state.mint() != mint_a.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount_to_give = escrow_state.amount_to_give();
        let bump = escrow_state.bump;

        (amount_to_give, bump)

    };

    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.address().as_array()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);


    pinocchio_token::instructions::Transfer {
        from: escrow_ata,
        to: maker_ata,
        authority: escrow,
        amount: amount_to_give,
    }.invoke()?;

    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow
    }.invoke_signed(&[seeds.clone()])?;

    maker.set_lamports(maker.lamports() + escrow.lamports());
    escrow.set_lamports(0);

    Ok(())
}