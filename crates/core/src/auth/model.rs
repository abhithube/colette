#[derive(Debug)]
pub struct Register<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Debug)]
pub struct Login<'a> {
    pub email: &'a str,
    pub password: &'a str,
}
