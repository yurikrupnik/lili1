
#[derive(Debug, Clone)]
struct AppManager {
    pub name: String,
}
// 
// use tokio::net::TcpListener;
// use shared::Env;
impl AppManager {
    // use tokio::net::TcpListener;
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
    // pub fn get_name(&self) -> &str {
    //   let listener = tokio::net::TcpListener::bind(Env::get_url().unwrap())
    //     .await
    //     .unwrap();
    //   println!("listening on {}", listener.local_addr().unwrap());
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
