#![allow(clippy::erasing_op)]
#![allow(clippy::identity_op)]
#![allow(clippy::eq_op)]

use crate::hosts::HostsFile;
use rtnetlink::{new_connection, IpVersion};

mod hosts;
mod neighbour;

#[tokio::main]
async fn main() -> Result<(), String> {
    run().await.expect("error occured");

    Ok(())
}

async fn run() -> Result<(), String> {
    let (conn, handle, _) = new_connection().unwrap();

    tokio::spawn(conn);

    let ipv4_neigh = neighbour::fetch_reachable_neighbours(handle.clone(), IpVersion::V4).await?;
    let ipv6_neigh = neighbour::fetch_reachable_neighbours(handle, IpVersion::V6).await?;

    let mut hosts = HostsFile::new(hosts::HOST_FILE_PATH)?;
    hosts.retain_ipv4_only();
    hosts.add_ipv6_clients(ipv6_neigh, ipv4_neigh);

    hosts.flush()
}
