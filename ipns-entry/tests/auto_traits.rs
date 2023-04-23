mod tests {

    use ipns_entry::entry::IpnsEntry;

    fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    fn normal_types() {
        is_normal::<IpnsEntry>();
    }
}
