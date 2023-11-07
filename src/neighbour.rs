use futures::TryStreamExt;
use netlink_packet_route::rtnl::neighbour::nlas::Nla;
use rtnetlink::{Handle, IpVersion, NeighbourHandle};
use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug)]
pub struct Record {
    pub lla: [u8; 6],
    pub address: IpAddr,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            lla: [0; 6],
            address: IpAddr::from([0, 0, 0, 0]),
        }
    }
}

pub async fn fetch_reachable_neighbours(
    handle: Handle,
    version: IpVersion,
) -> Result<Vec<Record>, String> {
    let mut output = vec![];
    let nh = NeighbourHandle::new(handle);
    let mut resp = nh.get().set_family(version.clone()).execute();

    while let Some(resp) = resp.try_next().await.map_err(|e| e.to_string())? {
        let record: Record = resp.nlas.into_iter().fold(Default::default(), |mut r, x| {
            match x {
                Nla::LinkLocalAddress(d) if !d.is_empty() => {
                    r.lla = [d[0], d[1], d[2], d[3], d[4], d[5]]
                }
                Nla::Destination(d) => {
                    r.address = match version {
                        IpVersion::V4 => IpAddr::from([d[0], d[1], d[2], d[3]]),
                        IpVersion::V6 => IpAddr::V6(Ipv6Addr::from(to_u128(&d))),
                    };
                }

                _ => (),
            };
            r
        });

        if record.lla != [0; 6] {
            output.push(record);
        }
    }

    Ok(output)
}

fn to_u128(v: &[u8]) -> u128 {
    let mut out = 0;
    let l = v.len() - 1;

    for (i, x) in v.iter().enumerate() {
        out |= (*x as u128) << ((l - i) * 8);
    }

    out
}
