use async_std;
use async_std::io::ReadExt;
use async_std::stream::StreamExt;
use async_std::task::spawn;
use crate::BoxedErrorResult;
use crate::component_manager::*;
use crate::constants;
use crate::easyhash::EasyHash;
use crate::globals;
use crate::heartbeat;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::future::Future;
use std::io::Write;

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
pub fn put(args: Vec<&str>, sender: &OperationSender) -> BoxedErrorResult<()> {
    let local_path = args[0];
    let distributed_filename = args[1];
    // Figure out who I am giving this file to
    let dest_ids = gen_file_owners(&distributed_filename)?;
    // let dest_ips = heartbeat::tcp_ips_from_ids(&dest_ids);
    // Gossip who has the file now
    sender.send(
        SendableOperation::for_successors(Box::new(NewFileOperation {
            filename: distributed_filename.to_string(),
            owners: dest_ids.clone()
        }))
    )?;
    // Send them the file
    async_std::task::block_on(send_file_to_all(local_path.to_string(),
                                               distributed_filename.to_string(),
                                               &dest_ids))?;
    Ok(())
}

async fn send_file_to_all(local_path: String, distributed_filename: String, dest_ids: &Vec<String>) ->
BoxedErrorResult<()> {
    let mut data_buf: Vec<u8> = Vec::new();
    let mut file = async_std::fs::File::open(&local_path).await?;
    file.read_to_end(&mut data_buf).await?;
    let operation = SendableOperation::for_id_list(dest_ids.clone(), Box::new(SendFileOperation {
        filename: distributed_filename,
        data: data_buf
    }));
    operation.write_all_tcp_async().await?;
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
    // TODO: Think about what standard we want with these
    let _generated_operations = operation.execute(source)?;
    Ok(())
}

// Helpers
fn gen_file_owners(filename: &str) -> BoxedErrorResult<Vec<String>> {
    let file_idx = filename.easyhash();
    heartbeat::gen_neighbor_list_from(file_idx as i32, 1, constants::NUM_OWNERS, true)
}

// Operations
#[derive(Serialize, Deserialize, Debug)]
pub struct GetOperation {
    pub filename: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewFileOperation {
    pub filename: String,
    pub owners: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendFileOperation {
    pub filename: String,
    pub data: Vec<u8>
}

// Trait Impls
impl OperationWriteExecute for GetOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("GET ")))
    }
    fn execute(&self, source: String) -> BoxedErrorResult<Vec<SendableOperation>> {
        println!("Received a GET request from {:?} for file {}", source, self.filename);
        Ok(vec![])
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}

impl OperationWriteExecute for NewFileOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("NFIL")))
    }
    fn execute(&self, source: String) -> BoxedErrorResult<Vec<SendableOperation>> {
        // TODO: Add this file to your map with the new people that have it
        println!("Received mention of a new file from {:?} for file {} and owners {:?}", source, self.filename, self.owners);
        Ok(vec![])
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}

impl OperationWriteExecute for SendFileOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("FILE")))
    }
    fn execute(&self, source: String) -> BoxedErrorResult<Vec<SendableOperation>> {
        // TODO: Check if the file exists before overwriting
        let filename = format!("{}/{}", constants::DATA_DIR, self.filename);
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filename)?;
        file.write_all(&self.data);
        Ok(vec![])
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}



