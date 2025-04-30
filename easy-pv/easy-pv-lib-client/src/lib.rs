pub mod client;
pub mod config;
pub mod helpers;


pub mod command;
pub mod state;
pub mod tcp;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}


