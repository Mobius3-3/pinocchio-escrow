use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_transaction::Transaction;

use super::super::helpers::setup;

#[test]
pub fn test_take_instruction() {
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

    // Execute make first to set up escrow state and deposit mint A
    let make_data = [
        vec![0u8],
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
    let make_msg = Message::new(&[make_ix], Some(&payer.pubkey()));
    let make_tx = Transaction::new(&[&payer], make_msg, svm.latest_blockhash());
    svm.send_transaction(make_tx).unwrap();

    // Prepare taker accounts and funds
    let taker = Keypair::new();
    svm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &taker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_b)
        .owner(&payer.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut svm, &payer, &mint_b, &taker_ata_b, amount_to_receive)
        .send()
        .unwrap();

    // Take instruction
    let take_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(taker.pubkey(), true),
            AccountMeta::new(payer.pubkey(), false),
            AccountMeta::new(escrow.0, false),
            AccountMeta::new(taker_ata_a, false),
            AccountMeta::new(taker_ata_b, false),
            AccountMeta::new(maker_ata_b, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(token_program, false),
            AccountMeta::new(associated_token_program, false),
        ],
        data: vec![1u8],
    };

    let take_msg = Message::new(&[take_ix], Some(&taker.pubkey()));
    let take_tx = Transaction::new(&[&taker], take_msg, svm.latest_blockhash());
    let tx = svm.send_transaction(take_tx).unwrap();

    println!("\n\nTake transaction successful");
    println!("CUs Consumed: {}", tx.compute_units_consumed);
    println!("Program logs:\n{}", tx.logs.join("\n"));
}
