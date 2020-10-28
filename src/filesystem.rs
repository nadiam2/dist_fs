use async_std;
use async_std::io::ReadExt;
use async_std::stream::StreamExt;
use async_std::task::spawn;
use crate::BoxedErrorResult;
use crate::component_manager::*;
use crate::constants;
use crate::easyhash::{EasyHash, Hex};
use crate::globals;
use crate::heartbeat;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::convert::TryInto;
use std::future::Future;
use std::io::Write;

pub fn get(args: Vec<&str>) -> BoxedErrorResult<()> {
    check_joined()?;
    if args.len() != 2 {
        return Err("Usage: get distributed_filename local_path".into())
    }

    let distributed_filename = args[0].to_string();
    let local_path = args[1].to_string();
    
    async_std::task::block_on(get_distributed_file(distributed_filename, local_path))?;
    Ok(())   
}

async fn get_distributed_file(distributed_filename: String, local_path: String) -> BoxedErrorResult<()> {
    // TODO: Find owners
    let operation = SendableOperation::for_successors(Box::new(GetOperation {
        distributed_filename: distributed_filename,
        local_path: local_path
    }));

    let mut streams = operation
        .write_all_tcp_async()
        .await?;

    // TODO: Redo whatever tf going on here
    let (result, source) = streams[0]
        .try_read_operation()
        .await?;
    result.execute(source)?;
    Ok(())
}

// args[0] = path to local file
// args[1] = distributed filename
pub fn put(args: Vec<&str>, sender: &OperationSender) -> BoxedErrorResult<()> {
    check_joined()?;
    if args.len() != 2 {
        return Err("Usage: put local_path distributed_filename".into())
    }
    
    let local_path = args[0];
    let distributed_filename = args[1];
    // Figure out who I am giving this file to
    let dest_ids = gen_file_owners(&distributed_filename)?;
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

async fn read_file_to_buf(local_path: &String) -> BoxedErrorResult<Vec<u8>> {
    let mut data_buf: Vec<u8> = Vec::new();
    let mut file = async_std::fs::File::open(&local_path).await?;
    file.read_to_end(&mut data_buf).await?;
    Ok(data_buf)
}

async fn send_file_to_all(local_path: String, distributed_filename: String, dest_ids: &Vec<String>) ->
BoxedErrorResult<()> {
    let data_buf = read_file_to_buf(&local_path).await?;
    let operation = SendableOperation::for_id_list(dest_ids.clone(), Box::new(SendFileOperation {
        filename: distributed_filename,
        data: data_buf,
        is_distributed: true
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

fn distributed_file_path(filename: &String) -> String {
    format!("{}/{}", constants::DATA_DIR, filename)
}

// Operations
#[derive(Serialize, Deserialize, Debug)]
pub struct GetOperation {
    pub distributed_filename: String,
    pub local_path: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewFileOperation {
    pub filename: String,
    pub owners: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendFileOperation {
    pub filename: String,
    pub data: Vec<u8>,
    pub is_distributed: bool
}

// Trait Impls
impl OperationWriteExecute for GetOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("GET ")))
    }
    fn execute(&self, source: Source) -> BoxedErrorResult<Vec<SendableOperation>> {
        let local_path = distributed_file_path(&self.distributed_filename);
        let data_buf = async_std::task::block_on(read_file_to_buf(&local_path))?;
        let operation = SendableOperation::for_single_tcp_stream(
            TryInto::<async_std::net::TcpStream>::try_into(source)?,
            Box::new(SendFileOperation {
                filename: self.local_path.clone(),
                data: data_buf,
                is_distributed: false
            }));
        async_std::task::block_on(operation.write_all_tcp_async());
        Ok(vec![])
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}

impl OperationWriteExecute for NewFileOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("NFIL")))
    }
    fn execute(&self, source: Source) -> BoxedErrorResult<Vec<SendableOperation>> {
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
    fn execute(&self, source: Source) -> BoxedErrorResult<Vec<SendableOperation>> {
        // TODO: Check if the file exists before overwriting
        let filename = match self.is_distributed {
            true  => format!("{}/{}", constants::DATA_DIR, self.filename),
            false => self.filename.clone()
        };
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



