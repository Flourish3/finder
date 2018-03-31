use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::sync::mpsc::RecvError;

pub use self::types::Backend;
pub use self::types::BackendData;
pub use self::types::BackendResponse;
pub use self::types::BackendCommand;

mod types;

impl Backend {
    pub fn new(tx : Sender<BackendResponse>) -> Backend {
        let data =  BackendData {
            search_query : "".to_string(),
        };
        Backend {
            tx : tx,
            internal_tx : None,
            data : Arc::new(Mutex::new(data)),
        }
    }

    pub fn run(mut self) -> Sender<BackendCommand> {
        let (apptx, rx) : (Sender<BackendCommand>, Receiver<BackendCommand>) = channel();

        self.internal_tx = Some(apptx.clone());
        thread::spawn(move || loop{
            let cmd = rx.recv();
            if !self.recieve_commad(cmd) {
                break;
            }
        });

        apptx
    }

    pub fn recieve_commad(&mut self, cmd : Result<BackendCommand, RecvError>) -> bool {

        match cmd {
            Ok(BackendCommand::Search(query)) => {
                self.process_query(query);
            }
            Ok(BackendCommand::Reset) => {
                println!( "Backend recieved reset");
            }
            Err(_) => {
                return false;
            }
        }
        true
    }

    pub fn process_query( &mut self, _query : String ) {
        let tx = self.tx.clone();
        tx.send(BackendResponse::SearchResult(vec!["1".to_string(),"2".to_string()])).unwrap();
    }
}
