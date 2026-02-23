use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    PolicyNotFound = 5,
    PolicyExpired = 6,
    PolicyNotActive = 7,
    ListingNotFound = 8,
    ListingNotActive = 9,
    InsufficientBalance = 10,
    InsufficientLiquidity = 11,
    TriggerNotFound = 12,
    TriggerAlreadyFired = 13,
    ClaimNotFound = 14,
    ClaimAlreadyProcessed = 15,
    InvalidPremium = 16,
    ContractPaused = 17,
    SlippageExceeded = 18,
    Overflow = 19,
    DivisionByZero = 20,
}
