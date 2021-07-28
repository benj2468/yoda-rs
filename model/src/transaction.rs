use super::*;

#[derive(Default, Clone, Debug, Api)]
#[auth(
    mutate = ["admin", "service"],
    query = ["admin", "service", "user"]
)]
struct Transaction {
    #[construct]
    identifier: Vec<atoms::Identifier>,
    #[searchable]
    amount: Option<u32>,
    #[construct]
    payment_method: Vec<Reference>,
    completed: Option<bool>,
}
#[derive(Default, MergedObject)]
pub struct TxnQuery(TransactionQuery);

#[derive(Default, MergedObject)]
pub struct TxnMutate(TransactionMutate);
