use std::collections::BTreeMap;

use hc_zome_transaction_requests_integrity::*;
use hc_zome_transactions_integrity::*;
use hdk::prelude::holo_hash::*;

use holochain::test_utils::consistency_10s;
use holochain::{conductor::config::ConductorConfig, sweettest::*};

#[tokio::test(flavor = "multi_thread")]
async fn simple_transaction() {
    // Use prebuilt DNA file
    let dna_path = std::env::current_dir()
        .unwrap()
        .join("./workdir/mutual_credit.dna");
    let dna = SweetDnaFile::from_bundle(&dna_path).await.unwrap();

    // Set up conductors
    let mut conductors = SweetConductorBatch::from_config(2, ConductorConfig::default()).await;
    let apps = conductors.setup_app("mutual_credit", &[dna]).await.unwrap();
    conductors.exchange_peer_info().await;

    let ((alice,), (bobbo,)) = apps.into_tuples();

    let alice_transactions = alice.zome("transactions");
    let bob_transactions = bobbo.zome("transactions");
    let alice_transaction_requests = alice.zome("transaction_requests");
    let bob_transaction_requests = bobbo.zome("transaction_requests");

    println!("Alice {}", alice.agent_pubkey());
    println!("Bob {}", bobbo.agent_pubkey());

    consistency_10s(&[&alice, &bobbo]).await;

    let map: BTreeMap<HeaderHashB64, Transaction> = conductors[0]
        .call(&alice_transactions, "query_my_transactions", ())
        .await;
    assert_eq!(map.len(), 0);

    let map: BTreeMap<HeaderHashB64, Transaction> = conductors[1]
        .call(&bob_transactions, "query_my_transactions", ())
        .await;
    assert_eq!(map.len(), 0);

    let transaction_request_input = CreateTransactionRequestInput {
        transaction_request_type: TransactionRequestType::Send,
        counterparty_pub_key: AgentPubKeyB64::from(bobbo.agent_pubkey().clone()),
        amount: 10.0,
    };

    let (transaction_request_hash, _): (HeaderHashB64, TransactionRequest) = conductors[0]
        .call(
            &alice_transaction_requests,
            "create_transaction_request",
            transaction_request_input,
        )
        .await;

    consistency_10s(&[&alice, &bobbo]).await;

    let transaction_requests: BTreeMap<HeaderHashB64, TransactionRequest> = conductors[0]
        .call(
            &alice_transaction_requests,
            "get_my_transaction_requests",
            (),
        )
        .await;

    assert_eq!(transaction_requests.len(), 1);

    let transaction_requests: BTreeMap<HeaderHashB64, TransactionRequest> = conductors[1]
        .call(&bob_transaction_requests, "get_my_transaction_requests", ())
        .await;

    assert_eq!(transaction_requests.len(), 1);

    let _txn: (HeaderHashB64, Transaction) = conductors[1]
        .call(
            &bob_transaction_requests,
            "accept_transaction_request",
            transaction_request_hash,
        )
        .await;

    consistency_10s(&[&alice, &bobbo]).await;
    consistency_10s(&[&alice, &bobbo]).await;
    consistency_10s(&[&alice, &bobbo]).await;

    let transactions: BTreeMap<HeaderHashB64, Transaction> = conductors[0]
        .call(&alice_transactions, "query_my_transactions", ())
        .await;
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions.into_iter().next().unwrap().1.amount, 10.0);

    let transactions: BTreeMap<HeaderHashB64, Transaction> = conductors[1]
        .call(&bob_transactions, "query_my_transactions", ())
        .await;
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions.into_iter().next().unwrap().1.amount, 10.0);

    let transactions: BTreeMap<HeaderHashB64, Transaction> = conductors[0]
        .call(
            &alice_transactions,
            "get_transactions_for_agent",
            AgentPubKeyB64::from(bobbo.agent_pubkey().clone()),
        )
        .await;
    assert_eq!(transactions.len(), 1);

    let transactions: BTreeMap<HeaderHashB64, Transaction> = conductors[1]
        .call(
            &bob_transactions,
            "get_transactions_for_agent",
            AgentPubKeyB64::from(alice.agent_pubkey().clone()),
        )
        .await;
    assert_eq!(transactions.len(), 1);

    let transaction_requests: BTreeMap<HeaderHashB64, TransactionRequest> = conductors[0]
        .call(
            &alice_transaction_requests,
            "get_my_transaction_requests",
            (),
        )
        .await;

    assert_eq!(transaction_requests.len(), 1);

    let transaction_requests: BTreeMap<HeaderHashB64, TransactionRequest> = conductors[1]
        .call(&bob_transaction_requests, "get_my_transaction_requests", ())
        .await;

    assert_eq!(transaction_requests.len(), 1);
}
