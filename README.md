# rdns-ipv6

In dual stack networks, This project can help in rdns resolution of IPv6 addresses by reusing hostname from DHCPv4 leases for that client.

It works like this,

1. Read IPv6 neighbours on the machine.
2. Identify hostname for this IPv6 client by trying to find a IPv4 lease for this MAC address
3. Update `/etc/hosts` with a list of IPv4 and IPv6 records.

There should be `unbound` or some other DNS resolver running on the machine to read `/etc/hosts` and respond to PTR queries and queries in your local search domain.
This program does not handle any aspect related to that.


This program exits after modifying `/etc/hosts`. It can be used with systemd to run at some interval.

It might be better to run this some how on netlink events, I have not looked into it yet.

```
# /etc/systemd/system/rdns-ipv6.timer
[Unit]
Description=rdns-ipv6

[Timer]
OnUnitActiveSec=30s
OnBootSec=30s

[Install]
WantedBy=timers.target


#/etc/systemd/system/rdns-ipv6.Service
[Unit]
Description=RDNS for IPv6

[Service]
Type=oneshot
ExecStart=/config/rdns-ipv6
```
