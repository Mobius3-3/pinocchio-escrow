use std::path::PathBuf;

use litesvm::LiteSVM;
use litesvm_token::{spl_token::{self}, CreateAssociatedTokenAccount, CreateMint, MintTo};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub fn program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

#[allow(clippy::too_many_arguments)]
pub fn setup() -> (
    Pubkey,
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    (Pubkey, u8),
    u8,
    (u64, u64),
) {
    let (mut svm, payer) = load_svm();

    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    println!("Mint A: {}", mint_a);

    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    println!("Mint B: {}", mint_b);

    // Create the maker's associated token account for Mint A
    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
        .owner(&payer.pubkey())
        .send()
        .unwrap();
    println!("Maker ATA A: {}\n", maker_ata_a);

    // Derive the PDA for the escrow account using the maker's public key and a seed value
    let escrow = Pubkey::find_program_address(
        &[b"escrow".as_ref(), payer.pubkey().as_ref()],
        &PROGRAM_ID.parse().unwrap(),
    );
    println!("Escrow PDA: {}\n", escrow.0);

    // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
    let vault = spl_associated_token_account::get_associated_token_address(
        &escrow.0, // owner will be the escrow PDA
        &mint_a,   // mint
    );
    println!("Vault PDA: {}\n", vault);

    // Define program IDs for associated token program, token program, and system program
    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM_ID;
    let system_program = solana_sdk_ids::system_program::ID;

    // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
    MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1_000_000_000)
        .send()
        .unwrap();

    let amounts = (100_000_000u64, 500_000_000u64); // 100 and 500 tokens (6 decimals)
    let bump = escrow.1;

    (
        program_id(),
        svm,
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
        amounts,
    )
}

pub struct MakeSetup {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub maker_ata_a: Pubkey,
    pub escrow_pda: Pubkey,
    pub escrow_ata: Pubkey,
    pub make_cu: u64,
}

pub fn load_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm
        .airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed");

    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/sbpf-solana-solana/release/escrow.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    let program_id = program_id();

    svm.add_program(program_id, &program_data)
        .expect("Failed to add program");

    assert_eq!(program_id.to_string(), PROGRAM_ID);

    (svm, payer)
}

pub fn setup_make(amount_to_receive: u64, amount_to_give: u64) -> MakeSetup {
    let (mut svm, maker) = load_svm();

    let mint_a = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
        .owner(&maker.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, 1_000_000_000)
        .send()
        .unwrap();

    let (escrow_pda, bump) =
        Pubkey::find_program_address(&[b"escrow", maker.pubkey().as_ref()], &program_id());
    let escrow_ata =
        spl_associated_token_account::get_associated_token_address(&escrow_pda, &mint_a);

    let data = [
        vec![0u8],
        bump.to_le_bytes().to_vec(),
        amount_to_receive.to_le_bytes().to_vec(),
        amount_to_give.to_le_bytes().to_vec(),
    ]
    .concat();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new(maker_ata_a, false),
            AccountMeta::new(escrow_ata, false),
            AccountMeta::new(solana_sdk_ids::system_program::ID, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(
                ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap(),
                false,
            ),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let blockhash = svm.latest_blockhash();
    let make_cu = svm
        .send_transaction(Transaction::new(&[&maker], msg, blockhash))
        .expect("send make v1")
        .compute_units_consumed;

    MakeSetup {
        svm,
        maker,
        mint_a,
        mint_b,
        maker_ata_a,
        escrow_pda,
        escrow_ata,
        make_cu,
    }
}

pub fn setup_make_v2(amount_to_receive: u64, amount_to_give: u64) -> MakeSetup {
    let (mut svm, maker) = load_svm();

    let mint_a = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &maker)
        .decimals(6)
        .authority(&maker.pubkey())
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &maker, &mint_a)
        .owner(&maker.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut svm, &maker, &mint_a, &maker_ata_a, 1_000_000_000)
        .send()
        .unwrap();

    let (escrow_pda, bump) =
        Pubkey::find_program_address(&[b"escrow", maker.pubkey().as_ref()], &program_id());
    let escrow_ata =
        spl_associated_token_account::get_associated_token_address(&escrow_pda, &mint_a);

    let data = [
        vec![3u8],
        amount_to_receive.to_le_bytes().to_vec(),
        amount_to_give.to_le_bytes().to_vec(),
        vec![bump],
    ]
    .concat();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new(maker_ata_a, false),
            AccountMeta::new(escrow_ata, false),
            AccountMeta::new(solana_sdk_ids::system_program::ID, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(
                ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap(),
                false,
            ),
        ],
        data,
    };

    let msg = Message::new(&[ix], Some(&maker.pubkey()));
    let blockhash = svm.latest_blockhash();
    let make_cu = svm
        .send_transaction(Transaction::new(&[&maker], msg, blockhash))
        .expect("send make v2")
        .compute_units_consumed;

    MakeSetup {
        svm,
        maker,
        mint_a,
        mint_b,
        maker_ata_a,
        escrow_pda,
        escrow_ata,
        make_cu,
    }
}
