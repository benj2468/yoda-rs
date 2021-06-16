use async_graphql::InputObject;

#[derive(Default, InputObject, Copy, Clone)]
pub struct PaginationOption {
    pub limit: usize,
    pub skip: usize,
}
