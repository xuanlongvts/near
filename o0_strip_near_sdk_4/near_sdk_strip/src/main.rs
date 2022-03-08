use near_sdk::{
    borsh::{self, BorshSerialize},
    serde::Deserialize,
    serde_json, AccountId,
};
use std::convert::TryFrom;

#[derive(Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
struct User {
    fingerprint: String,
    location: String,
}

#[derive(BorshSerialize, Debug)]
struct MyBorshSerializableStruct {
    value: String,
}

fn main() {
    let alice: AccountId = "alice.near".parse().unwrap();

    println!("alice 1: {:?}", alice);

    let alice_string = "alice".to_string();
    let alice = AccountId::try_from(alice_string.clone()).unwrap();
    // let alice: AccountId = alice_string.try_into().unwrap();
    println!("alice 2: {:?}", alice);

    let alice_into: AccountId = alice.into();
    println!("alice_into: {:?}", alice_into);

    // -------------
    let j = b"
        {
            \"fingerprint\": \"0xF9BA143B95FF6D82\",
            \"location\": \"Menlo Park, CA\"
        }";
    let user: User = serde_json::from_slice(j).unwrap();
    println!("user: ===> {:#?}", user);

    // -------------
    let x_1 = MyBorshSerializableStruct {
        value: "hello".to_owned(),
    };
    let mut buffer: Vec<u8> = Vec::new();
    x_1.serialize(&mut buffer).unwrap();
    let single_serialized_buffer_len = buffer.len();
    println!("buffer: {:?}", buffer);
}
