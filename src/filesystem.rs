use async_std;
use async_std::io::ReadExt;
use async_std::stream::StreamExt;
use async_std::task::spawn;
use crate::{BoxedError, BoxedErrorResult};
use crate::component_manager::*;
use crate::constants;
use crate::easyhash::{EasyHash, Hex};
use crate::globals;
use crate::heartbeat;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fmt;
use std::future::Future;
use std::io::Write;

pub fn get(args: Vec<&str>) -> BoxedErrorResult<()> {
    check_joined()?;
    if args.len() != 2 {
        return Err("Usage: get distributed_filename local_path".into())
    }

    let distributed_filename = args[0].to_string();
    let local_path = args[1].to_string();
    
    async_std::task::block_on(get_distributed_file_as_local(&distributed_filename, &local_path))?;
    Ok(())   
}

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
        SendableOperation::for_successors(Box::new(NewFileOwnersOperation {
            distributed_filename: distributed_filename.to_string(),
            new_owners: dest_ids
                .iter()
                .map(|x| x.to_string())
                .collect::<HashSet<_>>(),
            from_failure: false
        }))
    )?;
    // Send them the file
    async_std::task::block_on(send_file_to_all(local_path.to_string(),
                                               distributed_filename.to_string(),
                                               &dest_ids))?;
    Ok(())
}

pub fn ls(args: Vec<&str>) -> BoxedErrorResult<()> {
    check_joined()?;
    let invalid_args: BoxedErrorResult<()> = Err("Usage: ls [distributed_filename]".into());
    match args.len() {
        0 => {
            // All
            print_file_owners(None, true)?;            
            Ok(())
        },
        1 => {
            // Just File
            let distributed_filename = args[0];
            print_file_owners(Some(distributed_filename), false)?;
            Ok(())
        },
        _ => invalid_args
    }
}

// TODO: You wrote this very late - maybe fix
fn print_file_owners(maybe_distributed_filename: Option<&str>, full: bool) -> BoxedErrorResult<()> {
    let all_file_owners = globals::ALL_FILE_OWNERS.read();
    match (maybe_distributed_filename, full) {
        (Some(_), true) => {
           Err("Cannot set distributed_filename and full to true when printing owners".into())
        },
        (Some(distributed_filename), false) => {
            // Print the files owners
            match all_file_owners.get(distributed_filename) {
                Some(owners) => {
                    println!("{:?}", owners);
                },
                None => {
                    // A little unoptimal - change if above format changes
                    println!("{{}}");
                }
            }
            Ok(())
        },
        (None, true) => {
            // Print the whole map
            println!("{:?}", *all_file_owners);
            Ok(())
        },
        (None, false) => {
            Err("Cannot print owners of nonexistant distributed_filename with full set to false".into())
        }
    }
}

async fn get_distributed_file(distributed_filename: &String) -> BoxedErrorResult<()> {
    get_distributed_file_as_local(distributed_filename, &distributed_file_path(distributed_filename)).await
}

async fn get_distributed_file_as_local(distributed_filename: &String, local_path: &String) -> BoxedErrorResult<()> {
    // TODO: Find owners
    let operation = SendableOperation::for_owners(&distributed_filename, Box::new(GetOperation {
        distributed_filename: distributed_filename.clone(),
        local_path: local_path.clone()
    }));

    let mut streams = operation
        .write_all_tcp_async()
        .await?;

    // TODO: Redo whatever tf going on here
    match streams.len() {
        0 => Err(format!("No owners found for file {}", distributed_filename).into()),
        _ => {
            let (result, source) = streams[0]
                .try_read_operation()
                .await?;
            result.execute(source)?;
            Ok(())       
        }
    }
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

// TODO: This function makes the entire system assume there are always at least two nodes in the system
//       and the file must have an owner or else the operation will not work correctly. This is fine for now
//       but it is worth improving sooner rather than later (make distinct Error types to differentiate, etc).
fn gen_new_file_owner(filename: &str) -> BoxedErrorResult<String> {
    match globals::ALL_FILE_OWNERS.read().get(filename) {
        Some(owners) => {
            let potential_owners = gen_file_owners(filename)?;
            for potential_owner in &potential_owners {
                if !owners.contains(potential_owner) {
                    return Ok(potential_owner.clone());
                }
            }
            Err(format!("No new owners available for file {}", filename).into())
        },
        None => Err(format!("No owner found for file {}", filename).into())
    }
}

fn distributed_file_path(filename: &String) -> String {
    format!("{}/{}", constants::DATA_DIR, filename)
}

// Returns messages to be gossiped
// TODO: This function does NOT scale as # of files gets very large
pub fn handle_failed_node(failed_id: &String) -> BoxedErrorResult<Vec<SendableOperation>> {
    match heartbeat::is_master() {
        true => {
            let mut generated_operations: Vec<SendableOperation> = Vec::new();
            let myself_source = Source::myself();
            // Find files they owned
            let mut lost_files: HashSet<String> = HashSet::new();
            for (distributed_filename, owners) in globals::ALL_FILE_OWNERS.read().iter() {
                if owners.contains(failed_id) {
                    lost_files.insert(distributed_filename.clone());
                }
            }
            log("Found lost files".to_string());
            // Send that they no longer own those files
            // Separate operation so that this acts like a confirmation to fully forget about the node from
            // the master. Can be used with a delay later if you want more error resistance.
            let lost_file_operation = LostFilesOperation {
                failed_owner: failed_id.clone(),
                lost_files: lost_files.clone()
            };
            generated_operations.append(&mut lost_file_operation.execute(myself_source.clone())?);
            log("Executed lost files operation locally".to_string());
            // Gen new owners of the file and propagate
            let mut new_owners: HashMap<String, HashSet<String>> = HashMap::new();
            for lost_file in &lost_files {
                let new_owner = gen_new_file_owner(&lost_file)?;
                // TODO: Maybe optimize this into one fat packet - probably a new operation?
                let new_owner_operation = NewFileOwnersOperation {
                    distributed_filename: lost_file.clone(),
                    new_owners: vec![new_owner].iter().map(|x| x.to_string()).collect(),
                    from_failure: true
                };
                generated_operations.append(&mut new_owner_operation.execute(myself_source.clone())?);
            }
            log("Executed all new_owner operations locally".to_string());
            Ok(generated_operations)
        },
        false => {
            Ok(vec![])
        }
    }
}

// Operations
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetOperation {
    pub distributed_filename: String,
    pub local_path: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewFileOwnersOperation {
    pub distributed_filename: String,
    pub new_owners: HashSet<String>,
    pub from_failure: bool
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SendFileOperation {
    pub filename: String,
    pub data: Vec<u8>,
    pub is_distributed: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LostFilesOperation {
    pub failed_owner: String,
    pub lost_files: HashSet<String>
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

impl OperationWriteExecute for NewFileOwnersOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("NFO ")))
    }
    fn execute(&self, source: Source) -> BoxedErrorResult<Vec<SendableOperation>> {
        let mut all_file_owners = globals::ALL_FILE_OWNERS.get_mut();
        let mut file_owners = all_file_owners.entry(self.distributed_filename.clone()).or_insert(HashSet::new());
        let added_owners = &self.new_owners - file_owners;
        match added_owners.len() {
            0 => {
                Ok(vec![])
            },
            _ => {
                let mut generated_operations = vec![SendableOperation::for_successors(Box::new(self.clone()))];
                
                *file_owners = &self.new_owners | file_owners;
                // Need to drop all_file_owners since get_distributed_file needs to read the owners of the files
                drop(all_file_owners);

                if self.from_failure && added_owners.contains(&*globals::MY_ID.read()) {
                    async_std::task::block_on(get_distributed_file(&self.distributed_filename))?;
                }
                Ok(generated_operations)
            }
        }
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

impl fmt::Debug for SendFileOperation {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted_data  = if self.data.len() > 40 {
            format!("{:?}...", &self.data[..40])
        } else {
            format!("{:?}", &self.data)
        }; 
        fmt.debug_struct("SendFileOperation")
            .field("filename", &self.filename)
            .field("data", &formatted_data)
            .field("is_distributed", &self.is_distributed)
            .finish()
    }
}

impl OperationWriteExecute for LostFilesOperation {
    fn to_bytes(&self) -> BoxedErrorResult<Vec<u8>> {
        Ok(create_buf(&self, str_to_vec("LOST")))
    }
    fn execute(&self, source: Source) -> BoxedErrorResult<Vec<SendableOperation>> {
        let mut did_remove = false;
        let mut all_file_owners = globals::ALL_FILE_OWNERS.get_mut();
        for lost_file in &self.lost_files {
            if let Some(owners) = all_file_owners.get_mut(lost_file) {
                did_remove |= owners.remove(&self.failed_owner);
            }
        }
        if did_remove {
            Ok(vec![SendableOperation::for_successors(Box::new(self.clone()))])
        } else {
            Ok(vec![])
        }
    }
    fn to_string(&self) -> String { format!("{:?}", self) }
}
