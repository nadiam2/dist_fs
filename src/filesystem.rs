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
    operation.execute(source, sender)?;
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
        Ok(create_buf(&self, vec!['G' as u8]))
    }
    fn execute(&self, source: String, _sender: &OperationSender) -> BoxedErrorResult<()> {
        // Assert that source and self.id correspond to same ip
        println!("Received a GET request from {:?}", source);
        Ok(())
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}
