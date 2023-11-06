use crate::neighbour::Record as NeighbourRecord;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::{IpAddr, Ipv6Addr},
};

pub struct HostsFile {
    host_file_path: String,
    records: Vec<Record>,
}

#[derive(Debug)]
struct Record {
    address: IpAddr,
    hostname: String,
}

pub const HOST_FILE_PATH: &str = "/etc/hosts";

impl HostsFile {
    pub fn new(path: &str) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut buf = vec![];

        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

        Ok(Self {
            records: HostsFile::parse(&buf)?,
            host_file_path: path.to_string(),
        })
    }

    fn parse(data: &[u8]) -> Result<Vec<Record>, String> {
        let mut output = vec![];
        for line in data.split(|&x| x == b'\n') {
            let s = String::from_utf8_lossy(line);

            let s = s.trim();
            if s.starts_with('#') {
                continue;
            }

            if let Some((address, hostname)) = s.split_once(' ') {
                output.push(Record {
                    address: address.parse::<IpAddr>().map_err(|e| e.to_string())?,
                    hostname: hostname.to_string(),
                });
            }
        }

        Ok(output)
    }

    pub fn retain_ipv4_only(&mut self) {
        self.records.retain(|x| {
            x.address.is_ipv4()
                || ["fe00::0", "ff00::0", "ff02::1", "ff02::2"]
                    .map(|x| x.parse::<Ipv6Addr>().unwrap())
                    .map(IpAddr::V6)
                    .contains(&x.address)
        });
    }

    pub fn add_ipv6_clients(
        &mut self,
        ipv6_neigh: Vec<NeighbourRecord>,
        ipv4_neigh: Vec<NeighbourRecord>,
    ) {
        for neighbour in ipv6_neigh {
            // Lookup address from ipv4 neigh
            let ipv4_neighbour = if let Some(v) = ipv4_neigh.iter().find(|x| x.lla == neighbour.lla)
            {
                v
            } else {
                continue;
            };

            let hostname = if let Some(v) = self
                .records
                .iter()
                .find(|x| x.address == ipv4_neighbour.address)
            {
                v.hostname.clone()
            } else {
                continue;
            };

            self.records.push(Record {
                address: neighbour.address,
                hostname,
            });
        }
    }

    pub fn flush(self) -> Result<(), String> {
        let mut output = vec![];

        for record in self.records {
            output.push(format!("{} {}", record.address, record.hostname));
        }

        let mut f = OpenOptions::new()
            .read(false)
            .write(true)
            .create(true)
            .append(false)
            .truncate(true)
            .open(self.host_file_path)
            .map_err(|e| e.to_string())?;

        f.write_all(output.join("\n").as_bytes())
            .map_err(|e| e.to_string())?;

        println!("{}", output.join("\n"));

        f.flush().map_err(|e| e.to_string())
    }
}
