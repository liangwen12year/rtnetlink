// SPDX-License-Identifier: MIT

use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Error, Handle};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let link = "dummy0".to_string();
    println!("dumping bond port settings for link \"{link}\"");

    if let Err(e) = dump_bond_port_settings(handle, link).await {
        eprintln!("{e}");
    }

    Ok(())
}

async fn dump_bond_port_settings(handle: Handle, link: String) -> Result<(), Error> {
    let mut links = handle.link().get().match_name(link.clone()).execute();
    if let Some(link) = links.try_next().await? {
        let mut addresses = handle
            .link()
            .get()
            .match_name("dummy0".to_string())
            .execute();
        while let Some(msg) = addresses.try_next().await? {
            println!("{msg:?}");
        }
        Ok(())
    } else {
        eprintln!("link {link} not found");
        Ok(())
    }
}
