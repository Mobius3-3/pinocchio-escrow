use pinocchio::{cpi::{Seed, Signer}, error::ProgramError, AccountView, ProgramResult};
use pinocchio_pubkey::derive_address;
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{state::Escrow, ID};

pub fn process_cancel_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [maker, escrow_acc, maker_ata_a, escrow_ata, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let (bump, amount_to_give, maker_addr) = {
        if !maker.is_signer() {
            return Err(ProgramError::IncorrectAuthority);
        }

        let maker_addr = maker.address();
        let escrow_data = escrow_acc.try_borrow()?;
        let escrow_state = Escrow::load(&escrow_data)?;

        if escrow_state.maker() != *maker_addr.as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        let bump = escrow_state.bump;
        let amount_to_give = escrow_state.amount_to_give();

        let seeds: [&[u8]; 3] = [b"escrow", maker_addr.as_ref(), &[bump]];
        let expected_escrow = derive_address(&seeds, None, ID.as_array());
        if escrow_acc.address().as_array() != &expected_escrow {
            return Err(ProgramError::InvalidAccountData);
        }

        (bump, amount_to_give, maker_addr)
    };

    let bump_seed = [bump];
    let signer_seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker_addr.as_array()),
        Seed::from(&bump_seed),
    ];
    let escrow_signer = Signer::from(&signer_seeds[..]);

    Transfer {
        from: escrow_ata,
        to: maker_ata_a,
        authority: escrow_acc,
        amount: amount_to_give,
    }
    .invoke_signed(&[escrow_signer.clone()])?;

    CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_acc,
    }
    .invoke_signed(&[escrow_signer])?;

    let escrow_lamports = escrow_acc.lamports();
    maker.set_lamports(maker.lamports() + escrow_lamports);
    escrow_acc.set_lamports(0);

    Ok(())
}
