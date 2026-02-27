use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError
};
use crate::state::Escrow;

pub fn process_take_instruction(
    accounts: &[AccountView],
    _data: &[u8],
) -> ProgramResult {
    let [
        taker,
        maker,
        mint_a,
        mint_b,
        escrow,
        taker_ata_a, 
        taker_ata_b,
        maker_ata_b,
        escrow_ata_a, 
        _system_program,
        _token_program,
        _associated_token_program@ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !taker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let (amount_to_receive, amount_to_give, bump) = {
        let escrow_state = Escrow::from_account_info(&escrow)?;
        let taker_ata_a_state = pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_a)?;
        let taker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_view(&taker_ata_b)?;
        let maker_ata_b_state = pinocchio_token::state::TokenAccount::from_account_view(&maker_ata_b)?;

        if maker_ata_b_state.owner() != maker.address() || taker_ata_a_state.owner() != taker.address() || taker_ata_b_state.owner() != taker.address() {
            return Err(ProgramError::IllegalOwner);
        }
        
        if escrow_state.maker() != *maker.address()|| escrow_state.mint_a() != *mint_a.address() || escrow_state.mint_b() != *mint_b.address() ||
           taker_ata_a_state.mint() != mint_a.address() || taker_ata_b_state.mint() != mint_b.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount_to_receive = escrow_state.amount_to_receive();
        let amount_to_give = escrow_state.amount_to_give();
        let bump = escrow_state.bump;

        (amount_to_receive, amount_to_give, bump)

    };

    let bump = [bump.to_le()];
    let seed = [Seed::from(b"escrow"), Seed::from(maker.address().as_array()), Seed::from(&bump)];
    let seeds = Signer::from(&seed);

    pinocchio_token::instructions::Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }.invoke()?;

    pinocchio_token::instructions::Transfer {
        from: escrow_ata_a,
        to: taker_ata_a,
        authority: escrow,
        amount: amount_to_give,
    }.invoke()?;

    pinocchio_token::instructions::CloseAccount {
        account: escrow_ata_a,
        destination: maker,
        authority: escrow
    }.invoke_signed(&[seeds.clone()])?;

    maker.set_lamports(maker.lamports() + escrow.lamports());
    escrow.set_lamports(0);

    Ok(())
}