struct Mite<'a> {
    key: &'a str,
    namespace: &'a str
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
