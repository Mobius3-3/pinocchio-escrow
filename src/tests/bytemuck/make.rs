use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::super::helpers::setup;

#[test]
pub fn test_make_instruction() {
    let (
        program_id,
        mut svm,
        payer,
        mint_a,
        mint_b,
        associated_token_program,
        token_program,
        system_program,
        maker_ata_a,
        vault,
        escrow,
        bump,
        (amount_to_receive, amount_to_give),
    ) = setup();

    let make_data = [
        vec![0u8], // Discriminator for "Make" instruction
        bump.to_le_bytes().to_vec(),
        amount_to_receive.to_le_bytes().to_vec(),
        amount_to_give.to_le_bytes().to_vec(),
    ]
    .concat();
    let make_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(escrow.0, false),
            AccountMeta::new(maker_ata_a, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(token_program, false),
            AccountMeta::new(associated_token_program, false),
        ],
        data: make_data,
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();

    println!("\n\nMake transaction sucessfull");
    println!("CUs Consumed: {}", tx.compute_units_consumed);
    println!("Program logs:\n{}", tx.logs.join("\n"));
}
