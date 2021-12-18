use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::{
    env, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, PanicOnDefault,
};

const PRIZE_AMOUNT: u128 = 5_000_000_000_000_000_000_000_000; // 5 Ⓝ in yoctoNEAR

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum AnswerDirection {
    Across,
    Down,
}

/// The origin (0, 0) starts at the top left side of the square
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CoordinatePair {
    x: u8,
    y: u8,
}

// {"num": 1, "start": {"x": 19, "y": 31}, "direction": "Across", "length": 8, "clue": "not far but"}
// We'll have the clue stored on-chain for now for simplicity.
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Answer {
    num: u8,
    start: CoordinatePair,
    direction: AnswerDirection,
    length: u8,
    clue: String,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PuzzleStatus {
    Unsolved,
    Soved { memo: String },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPuzzle {
    solution_hash: String, // The human-readable (not in bytes) hash of the solution
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UnsolvedPuzzles {
    puzzles: Vec<JsonPuzzle>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Puzzle {
    status: PuzzleStatus,
    answer: Vec<Answer>,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Crossword {
    owner_id: AccountId,
    puzzles: LookupMap<String, Puzzle>,
    unsolved_puzzles: UnorderedSet<String>,
}

#[near_bindgen]
impl Crossword {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            puzzles: LookupMap::new(b"c"),
            unsolved_puzzles: UnorderedSet::new(b"u"),
        }
    }

    pub fn new_puzzle(&mut self, solution_hash: String, answers: Vec<Answer>) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the onwer may call his method"
        );
        let existing = self.puzzles.insert(
            &solution_hash,
            &Puzzle {
                status: PuzzleStatus::Unsolved,
                answer: answers,
            },
        );

        assert!(existing.is_none(), "Puzzle with that key already exists");
        self.unsolved_puzzles.insert(&solution_hash);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::{
//         test_utils::{get_logs, VMContextBuilder},
//         testing_env, AccountId,
//     };

//     #[test]
//     fn debug_get_hash() {
//         testing_env!(VMContextBuilder::new().build());
//         let debug_solution = "near nomicon ref finance";
//         let debug_hash_bytes = env::sha256(debug_solution.as_bytes());
//         let debug_hash_string = hex::encode(debug_hash_bytes);
//         println!("========> Let's debug: {:?}", debug_hash_string);
//     }

//     fn get_context(predecessor: AccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder.predecessor_account_id(predecessor);
//         builder
//     }

//     #[test]
//     fn check_guess_solution() {
//         let alice = AccountId::new_unchecked("alice.testnet".to_string());

//         let context = get_context(alice);
//         testing_env!(context.build());

//         let mut contract = Contract::new(
//             "69c2feb084439956193f4c21936025f14a5a5a78979d67ae34762e18a7206a0f".to_string(),
//         );
//         let mut guess_result = contract.guess_solution("wrong answer here".to_string());
//         assert!(!guess_result, "Expected a fairlure from the wrong guess");
//         assert_eq!(get_logs(), ["Try again."], "Expected a failure log.");

//         let debug_solution = "near nomicon ref finance";
//         guess_result = contract.guess_solution(debug_solution.to_string());
//         assert!(guess_result, "Expected the correct answer to return true.");
//         assert_eq!(
//             get_logs(),
//             ["Try again.", "You guessed right!"],
//             "Expected a successful log after the previous failed log."
//         );
//     }
// }
