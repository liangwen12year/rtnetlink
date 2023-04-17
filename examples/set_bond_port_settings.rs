// SPDX-License-Identifier: MIT

use rtnetlink::new_connection;

#[tokio::main]
async fn main() -> Result<(), String> {
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);
    handle
        .link()
        .set(9)
        .bondport("dummy0".to_string())
        .match_name("dummy0".to_string())
        .queue_id(1)
        .prio(6)
        .up()
        .execute()
        .await
        .map_err(|e| format!("{e}"))
}
