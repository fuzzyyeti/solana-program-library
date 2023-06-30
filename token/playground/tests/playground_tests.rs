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
    solana_sdk::{instruction::AccountMeta},
    spl_transfer_hook_example::state::example_data,
    spl_transfer_hook_interface::get_extra_account_metas_address,
    spl_token_2022::extension::transfer_hook::instruction::initialize,
    spl_token_2022::instruction::initialize_mint2,
    spl_token_2022::instruction::mint_to,
};
use solana_sdk::signature::{Keypair, Signer};
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::state::Mint;
use spl_associated_token_account::{instruction::create_associated_token_account, get_associated_token_address, get_associated_token_address_with_program_id};
use spl_token_2022::extension::ExtensionType::ImmutableOwner;


#[tokio::test]
async fn create_transfer_hook_ata() {
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
        solana_sdk::account::Account {
            lamports: 1_000_000_000, // a lot, just to be safe
            data,
            owner: transfer_hook_program_id,
            ..solana_sdk::account::Account::default()
        },
    );


    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    let create_account_instruction = create_account(
        &payer.pubkey(), // Account to be initialized
        &mint.pubkey(), // Program ID
        1_000_000_000, // Initial balance
        ExtensionType::try_get_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap() as u64,
        //  ExtensionType::try_get_account_len::<Mint>(&[]).unwrap() as u64,
        &spl_token_2022::ID, // Allocate to the same program ID
    );

    let ix = initialize_mint2(
        &spl_token_2022::ID,
        &mint.pubkey(),
        &payer.pubkey(),
        Some(&payer.pubkey()),
        9,
    ).unwrap();

    let transfer_hook_init_ix = initialize(
        &spl_token_2022::ID,
        &mint.pubkey(),
        Some(payer.pubkey()),
        Some(transfer_hook_program_id)
    ).unwrap();


    let mut tx = Transaction::new_signed_with_payer(
        &[create_account_instruction, transfer_hook_init_ix],
        Some(&payer.pubkey()),
        &[&mint, &payer],
        recent_blockhash,
    );
    let result = banks_client.process_transaction(tx).await;
    match result {
        Ok(()) => println!("Transaction succeeded"),
        Err(e) => eprintln!("Transaction failed: {:?}", e),

    }

    let mut tx2 = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    println!("Trying to init transfer hook");
    let result = banks_client.process_transaction(tx2).await;
    match result {
        Ok(()) => println!("Transfer Hook Init succeeded"),
        Err(e) => eprintln!("Transfer Hook Init failed: {:?}", e),
    }

    let alice = Keypair::new();

    let alice_create_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &alice.pubkey(),
        &mint.pubkey(),
        &spl_token_2022::ID,
    );

    let mut create_ata_tx = Transaction::new_signed_with_payer(
        &[alice_create_ata_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );



    let result = banks_client.process_transaction(create_ata_tx).await;
    match result {
        Ok(()) => println!("Create ATA Transaction succeeded"),
        Err(e) => eprintln!("Create ATA Transaction failed: {:?}", e),
    };

}
#[tokio::test]
async fn create_non_transfer_hook_ata() {
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
        solana_sdk::account::Account {
            lamports: 1_000_000_000, // a lot, just to be safe
            data,
            owner: transfer_hook_program_id,
            ..solana_sdk::account::Account::default()
        },
    );


    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let create_account_instruction = create_account(
        &payer.pubkey(), // Account to be initialized
        &mint.pubkey(), // Program ID
        1_000_000_000, // Initial balance
        ExtensionType::try_get_account_len::<Mint>(&[]).unwrap() as u64,
        //  ExtensionType::try_get_account_len::<Mint>(&[]).unwrap() as u64,
        &spl_token_2022::ID, // Allocate to the same program ID
    );

    let ix = initialize_mint2(
        &spl_token_2022::ID,
        &mint.pubkey(),
        &payer.pubkey(),
        None,
        9,
    ).unwrap();

    let mut tx = Transaction::new_signed_with_payer(
        &[create_account_instruction, ix],
        Some(&payer.pubkey()),
        &[&mint, &payer],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(tx).await;
    match result {
        Ok(()) => println!("Transaction succeeded"),
        Err(e) => eprintln!("Transaction failed: {:?}", e),
    }

    let alice = Keypair::new();

    let alice_create_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &alice.pubkey(),
        &mint.pubkey(),
        &spl_token_2022::ID,
    );

    let mut create_ata_tx = Transaction::new_signed_with_payer(
        &[alice_create_ata_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(create_ata_tx).await;
    match result {
        Ok(()) => println!("Create ATA Transaction succeeded"),
        Err(e) => eprintln!("Create ATA Transaction failed: {:?}", e),
    };
}
