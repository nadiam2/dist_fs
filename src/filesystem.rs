use async_std;
use async_std::stream::StreamExt;
use async_std::task::spawn;
use crate::BoxedErrorResult;
use crate::component_manager::*;
use crate::constants;
use crate::globals;
use crate::heartbeat;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::future::Future;

pub fn get(args: Vec<&str>) -> BoxedErrorResult<()> {
    check_joined()?;
    if args.len() == 0 {
        return Err("No filename given for 'get'".into())
    }

    let filename = args[0].to_string();
    // TODO: Find owners
    let operation = SendableOperation::for_successors(Box::new(GetOperation {
        filename: filename
    }));
    async_std::task::block_on(operation.write_all_tcp_async())?;
    Ok(())   
}

// args[0] = path to local file
// args[1] = distributed filename
// pub fn put(args: Vec<&str>, sender: &OperationSender) {
//     let local_path = args[0];
//     let distributed_filename = args[1];
//     // Figure out who I am giving this file to
//     let dests = gen_file_owners(&distributed_filename);
//     // Send them the file
//     for dest in &dests {
//         // SEND THE THING
//     }
//     // Gossip who has the file now
//     sender.send(
//         SendableOperation::for_successors(Box::new(NewFileOperation {
//             filename: distributed_filename.to_string(),
//             owners: dests
//         }))
//     )?;
// }

pub async fn file_server<'a>(_sender: &'a OperationSender) -> BoxedErrorResult<()> {
    let server = globals::SERVER_SOCKET.read();
    let mut incoming = server.incoming();

    while let Some(stream) = incoming.next().await {
        let connection = stream?;
        log(format!("Handling connection from {:?}", connection.peer_addr()));
        spawn(handle_connection(connection));
    }
    Ok(())
}

async fn handle_connection(mut connection: async_std::net::TcpStream) -> BoxedErrorResult<()> {
    let (operation, source) = connection.try_read_operation().await?;
    println!("reached");
    // TODO: Think about what standard we want with these
    let _generated_operations = operation.execute(source)?;
    Ok(())
}



// Operations
#[derive(Serialize, Deserialize, Debug)]
pub struct GetOperation {
    pub filename: String
}

// Trait Impls
impl OperationWriteExecute for GetOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("GET ")))
    }
    fn execute(&self, source: String) -> BoxedErrorResult<Vec<SendableOperation>> {
        // Assert that source and self.id correspond to same ip
        println!("Received a GET request from {:?} for file {}", source, self.filename);
        Ok(vec![])
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}
