use super::*;
use quote::quote;

mod mutate;
mod output;
mod query;
mod store;
mod update;

pub(crate) fn derive(input: DeriveData) -> TokenStream {
    let store = store::derive(&input);

    let output = output::derive(&input);

    let query = query::derive(&input);

    let mutate = mutate::derive(&input);

    let update = update::derive(&input);

    TokenStream::from(quote! {
        #store

        #output

        #query

        #mutate

        #update
    })
}
