#![deny(unused_extern_crates)]

#[macro_use] 
extern crate lazy_static;
extern crate regex;

#[macro_use]
pub mod util;

pub mod backend;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
