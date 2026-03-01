use pinocchio::{cpi::{Seed, Signer}, error::ProgramError, AccountView, ProgramResult};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{instructions::{CloseAccount, Transfer}, state::TokenAccount};

use crate::{state::Escrow, ID};

pub fn process_take_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [taker, maker, escrow_account, taker_ata_a, taker_ata_b, maker_ata_b, escrow_ata, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // validation
    let (bump, amount_to_receive, amount_to_give, maker_addr) = {
        if !taker.is_signer() {
            return Err(ProgramError::IncorrectAuthority);
        }

        let escrow_data = escrow_account.try_borrow()?;
        let escrow_state = Escrow::load(&escrow_data)?;
        if escrow_state.maker() != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        let bump = escrow_state.bump;
        let seeds: [&[u8]; 3] = [b"escrow", maker.address().as_ref(), &[bump]];
        let expected_escrow_addr = derive_address(&seeds, None, ID.as_array());
        if escrow_account.address().as_array() != &expected_escrow_addr {
            return Err(ProgramError::InvalidAccountData);
        }

        let mint_b = escrow_state.mint_b();
        let maker_ata_b_state = TokenAccount::from_account_view(maker_ata_b)?;
        if maker_ata_b_state.owner() != maker.address() {
            return Err(ProgramError::IllegalOwner);
        } else if maker_ata_b_state.mint().as_array() != &mint_b {
            return Err(ProgramError::InvalidAccountData);
        }

        let amount_to_receive = escrow_state.amount_to_receive();
        let amount_to_give = escrow_state.amount_to_give();

        (bump, amount_to_receive, amount_to_give, maker.address())
    };

    // cpis
    let bump_seed = [bump];
    let signer_seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker_addr.as_array()),
        Seed::from(&bump_seed),
    ];
    let escrow_signer = Signer::from(&signer_seeds[..]);

    Transfer {
        from: taker_ata_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_to_receive,
    }
    .invoke()?;

    Transfer {
        from: escrow_ata,
        to: taker_ata_a,
        authority: escrow_account,
        amount: amount_to_give,
    }
    .invoke_signed(&[escrow_signer.clone()])?;

    CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[escrow_signer])?;

    // close escrow account and refund rent to maker
    let escrow_lamports = escrow_account.lamports();
    maker.set_lamports(maker.lamports() + escrow_lamports);
    escrow_account.set_lamports(0);

    Ok(())
}
