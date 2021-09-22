use zoon::{*, println, eprintln};

use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction,
    message::Message,
    signer::{Signer, keypair::Keypair},
};
use voting_program::instruction as voting_instruction;
use shared::UpMsg;
use crate::{connection::connection, app};

pub fn create_and_send_transaction(voting_owner_keypair: Keypair, voter_pubkey: Pubkey) {
    let voting_owner_pubkey = voting_owner_keypair.pubkey();

    let seeds = &[b"voter_votes", voter_pubkey.as_ref(), voting_owner_pubkey.as_ref()];
    let voter_votes_pubkey = Pubkey::find_program_address(seeds, &voting_program::id()).0;
    println!("voter_votes_pubkey: {}", voter_votes_pubkey);

    let add_voter_ix = voting_instruction::add_voter(
        &voting_owner_pubkey, 
        &voter_votes_pubkey,
        &voter_pubkey
    );

    Task::start(async move {
        // @TODO refactor
        let mut blockhash_stream = app::recent_blockhash().signal().to_stream();
        let _ = blockhash_stream.next().await;
        if let Err(error) = connection().send_up_msg(UpMsg::RecentBlockhash).await {
            let error = error.to_string();
            eprintln!("recent_blockhash request failed: {}", error);
            super::set_status(error);
        }
        let recent_blockhash = blockhash_stream.next().await.unwrap_throw().unwrap_throw();
        println!("recent_blockhash: {:#?}", recent_blockhash);

        let message = Message::new(
            &[add_voter_ix], 
            None
        );
        let transaction = Transaction::new(
            &[&voting_owner_keypair], 
            message, 
            recent_blockhash
        );

        let up_msg = UpMsg::AddVoter {
            voter_pubkey,
            transaction,
        };
        if let Err(error) = connection().send_up_msg(up_msg).await {
            let error = error.to_string();
            eprintln!("add_voter request failed: {}", error);
            super::set_status(error);
        }
    
        println!("add_voter transaction sent.");
    });
}
