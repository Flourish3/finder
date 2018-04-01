use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use regex::Regex;

pub enum BackendResponse {
    SearchResult(Vec<String>)
}

pub enum BackendCommand {
    Search(String),
    Reset,
}

pub struct BackendData {
    pub search_query : String,
    pub result : Vec<String>,
    pub re : Regex,
}

pub struct Backend {
    pub tx : Sender<BackendResponse>,
    pub data : Arc<Mutex<BackendData>>,
    pub internal_tx : Option<Sender<BackendCommand>>,
}

impl Clone for Backend {
    fn clone(&self) -> Backend {
        Backend {
            tx : self.tx.clone(),
            data : self.data.clone(),
            internal_tx : self.internal_tx.clone(),
        }
    }
}
