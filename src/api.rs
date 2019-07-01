use std::io::Read;

use rocket::{Data, State};
use byteorder::{ReadBytesExt, LittleEndian};

use crate::{Error, Crate, AppState};

#[put("/crates/new", data = "<data>")]
pub fn api_publish(state: State<AppState>, data: Data) -> Result<(), Error> {
    let mut stream = data.open();
    let metadata_size = stream.read_u32::<LittleEndian>()?;
    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    stream.read_exact(metadata_bytes.as_mut_slice())?;
    let metadata: Crate = json::from_slice(metadata_bytes.as_slice())?;
    let crate_size = stream.read_u32::<LittleEndian>()?;
    let mut crate_bytes = vec![0u8; crate_size as usize];
    stream.read_exact(crate_bytes.as_mut_slice())?;

    Ok(())
}