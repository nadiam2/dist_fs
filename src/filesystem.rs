use async_std;
use crate::BoxedErrorResult;
use crate::component_manager::*;
use crate::constants;
use crate::globals;
use crate::operation::*;
use serde::{Serialize, Deserialize};
use std::future::Future;

pub async fn file_server<'a>(_sender: &'a OperationSender) -> BoxedErrorResult<()> {
    thing().await;
    Ok(())
}

pub async fn thing() {
    println!("hi from the file server with joined {}", *globals::IS_JOINED.read());
}
