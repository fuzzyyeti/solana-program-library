use std::alloc::System;
use std::cmp::min;
use solana_program::bpf_loader_upgradeable::UpgradeableLoaderState::Buffer;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;
use solana_program::program_pack::Pack;
use solana_sdk::system_instruction::create_account;
use solana_sdk::transaction::Transaction;
use {
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{account::Account, instruction::AccountMeta },
    spl_transfer_hook_example::state::example_data,
    spl_transfer_hook_interface::get_extra_account_metas_address,
    spl_token_2022::extension::transfer_hook::instruction::initialize,
    spl_token_2022::instruction::mint_to
};
use solana_sdk::signature::{Keypair, Signer};
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::Mint;
use spl_associated_token_account::{instruction::create_associated_token_account, get_associated_token_address};


#[tokio::test]
async fn playground_test() {
    println!("Getting started now.");
    let transfer_hook_program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "spl_transfer_hook_example",
        transfer_hook_program_id,
        processor!(spl_transfer_hook_example::processor::process),
    );
    program_test.prefer_bpf(false);


    let mint = Keypair::new();

    let extra_accounts_address = get_extra_account_metas_address(&mint.pubkey(), &transfer_hook_program_id);

    let account_metas = vec![
        AccountMeta {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: Pubkey::new_unique(),
            is_signer: false,
            is_writable: false,
        },
    ];
    let data = example_data(&account_metas).unwrap();
    program_test.add_program("spl-token-2022",
    spl_token_2022::ID,
    processor!(spl_token_2022::processor::Processor::process));

    program_test.add_program("spl-associated-token-account",
    spl_associated_token_account::ID,
    processor!(spl_associated_token_account::processor::process_instruction));

    program_test.add_account(
        extra_accounts_address,
        Account {
            lamports: 1_000_000_000, // a lot, just to be safe
            data,
            owner: transfer_hook_program_id,
            ..Account::default()
        },
    );



    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let create_account_instruction = create_account(
        &payer.pubkey(), // Account to be initialized
        &mint.pubkey(), // Program ID
        1_000_000_000, // Initial balance
        ExtensionType::get_account_len::<Mint>(&[ExtensionType::TransferHook]) as u64,
        &spl_token_2022::ID, // Allocate to the same program ID
    );
    let ix = initialize(
        &spl_token_2022::ID,
        &mint.pubkey(),
        Some(payer.pubkey()),
        Some(transfer_hook_program_id)
    ).unwrap();

    let mut tx = Transaction::new_with_payer(
        &[create_account_instruction, ix],
        Some(&payer.pubkey()),
    );

    tx.sign(&[&mint,&payer], recent_blockhash);

    let result = banks_client.process_transaction(tx).await;
   match result {
       Ok(()) => println!("Transaction succeeded"),
       Err(e) => eprintln!("Transaction failed: {:?}", e),
    }
    println!("transfer hook program {}",transfer_hook_program_id.to_string());
    println!("derived PDA {}",extra_accounts_address.to_string());

    //Try to mint some tokens
    //
    let alice = Keypair::new();
    let bob = Pubkey::new_unique();
    let alice_ata =  get_associated_token_address(
       &alice.pubkey(),
        &mint.pubkey(),
    );
    println!("ata {}", alice_ata.to_string());

    let alice_create_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &alice.pubkey(),
        &mint.pubkey(),
        &spl_token_2022::ID,
    );

    let mint_tokens_ix = mint_to(
        &spl_token_2022::ID,
        &mint.pubkey(),
        &alice_ata,
        &alice.pubkey(),
        &[&payer.pubkey()],
        100).unwrap();
    //
    //
        let mut mint_tokens_tx = Transaction::new_with_payer(
            &[alice_create_ata_ix],
            Some(&payer.pubkey()),
        );

        mint_tokens_tx.sign(&[&payer], recent_blockhash);

        let result = banks_client.process_transaction(mint_tokens_tx).await;
        match result {
            Ok(()) => println!("Token Mint Transaction succeeded"),
            Err(e) => eprintln!("Token Mint Transaction failed: {:?}", e),
        };
}
