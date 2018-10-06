error_chain! {
    links {
        StoreError(::replicante_data_store::Error, ::replicante_data_store::ErrorKind);
    }
}
