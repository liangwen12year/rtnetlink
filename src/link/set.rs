// SPDX-License-Identifier: MIT

use std::os::unix::io::RawFd;

use futures::stream::StreamExt;
use netlink_packet_core::{
    NetlinkMessage, NLM_F_ACK, NLM_F_CREATE, NLM_F_EXCL, NLM_F_REPLACE,
    NLM_F_REQUEST,
};
use netlink_packet_route::{
    link::nlas::{Info, InfoBondPort, InfoSlaveData, InfoSlaveKind, Nla},
    LinkMessage, RtnlMessage, IFF_NOARP, IFF_PROMISC, IFF_UP,
};

use crate::{try_nl, Error, Handle};

pub struct BondPortSetRequest {
    request: LinkSetRequest,
    info_slave_data: Vec<InfoBondPort>,
}

impl BondPortSetRequest {
    /// Execute the request.
    pub async fn execute(self) -> Result<(), Error> {
        let s = self
            .request
            .link_info(InfoSlaveKind::Bond, Some(InfoSlaveData::BondPort(self.info_slave_data)));
        s.execute().await
    }

    /// Sets the interface up
    /// This is equivalent to `ip link set up dev NAME`.
    pub fn up(mut self) -> Self {
        self.request = self.request.up();
        self
    }

    /// Adds the `queue_id` attribute to the bond port
    /// This is equivalent to `ip link set name NAME type bond_slave queue_id QUEUE_ID`.
    pub fn queue_id(mut self, queue_id: u16) -> Self {
        eprintln!("queue_id starting");
        self.info_slave_data.push(InfoBondPort::QueueId(queue_id));
        self
    }

    /// Adds the `prio` attribute to the bond port
    /// This is equivalent to `ip link set name NAME type bond_slave prio PRIO`.
    pub fn prio(mut self, prio: i32) -> Self {
        eprintln!("prio starting");
        self.info_slave_data.push(InfoBondPort::Prio(prio));
        self
    }

    pub fn linkfailurecount(mut self, linkfailurecount: u32) -> Self {
        eprintln!("linkfailurecount starting");
        self.info_slave_data.push(InfoBondPort::LinkFailureCount(linkfailurecount));
        self
    }

    /// Lookup a link by name
    ///
    /// This function requires support from your kernel (>= 2.6.33). If yours is
    /// older, consider filtering the resulting stream of links.
    pub fn match_name(mut self, name: String) -> Self {
        self.request.message.nlas.push(Nla::IfName(name));
        self
    }
}

pub struct LinkSetRequest {
    handle: Handle,
    message: LinkMessage,
    replace: bool,
}

impl LinkSetRequest {
    pub(crate) fn new(handle: Handle, index: u32) -> Self {
        let mut message = LinkMessage::default();
        message.header.index = index;
        LinkSetRequest { handle, message:LinkMessage::default(), replace: false}
    }

    /// Execute the request
    pub async fn execute(self) -> Result<(), Error> {
        let LinkSetRequest {
            mut handle,
            message,
            replace,
        } = self;
        eprintln!("******bond port replace bool*******");
        eprintln!("{:?}", replace);
        eprintln!("******link total message*******");
        eprintln!("{:?}", message);
        let mut req = NetlinkMessage::from(RtnlMessage::NewLink(message));
        let replace: u16 = if replace { NLM_F_REPLACE } else { NLM_F_EXCL };
        req.header.flags =
            NLM_F_REQUEST | NLM_F_ACK | replace | NLM_F_EXCL | NLM_F_CREATE;

        let mut response = handle.request(req)?;
        eprintln!("******bond port response: *******");
        // eprintln!("{:?}", response);
        while let Some(message) = response.next().await {
            try_nl!(message);
        }
        Ok(())
    }

    /// Return a mutable reference to the request
    pub fn message_mut(&mut self) -> &mut LinkMessage {
        &mut self.message
    }

    /// Attach the link to a bridge (its _master_). This is equivalent to `ip
    /// link set LINK master BRIDGE`. To succeed, both the bridge and the
    /// link that is being attached must be UP.
    ///
    /// To Remove a link from a bridge, set its master to zero.
    /// This is equvalent to `ip link set LINK nomaster`
    pub fn master(mut self, master_index: u32) -> Self {
        self.message.nlas.push(Nla::Master(master_index));
        self
    }

    /// Detach the link from its _master_. This is equivalent to `ip link set
    /// LINK nomaster`. To succeed, the link that is being detached must be
    /// UP.
    pub fn nomaster(mut self) -> Self {
        self.message.nlas.push(Nla::Master(0u32));
        self
    }

    /// Set the link with the given index up (equivalent to `ip link set dev DEV
    /// up`)
    pub fn up(mut self) -> Self {
        self.message.header.flags |= IFF_UP;
        self.message.header.change_mask |= IFF_UP;
        self
    }

    /// Set the link with the given index down (equivalent to `ip link set dev
    /// DEV down`)
    pub fn down(mut self) -> Self {
        self.message.header.flags &= !IFF_UP;
        self.message.header.change_mask |= IFF_UP;
        self
    }

    /// Enable or disable promiscious mode of the link with the given index
    /// (equivalent to `ip link set dev DEV promisc on/off`)
    pub fn promiscuous(mut self, enable: bool) -> Self {
        if enable {
            self.message.header.flags |= IFF_PROMISC;
        } else {
            self.message.header.flags &= !IFF_PROMISC;
        }
        self.message.header.change_mask |= IFF_PROMISC;
        self
    }

    /// Enable or disable the ARP protocol of the link with the given index
    /// (equivalent to `ip link set dev DEV arp on/off`)
    pub fn arp(mut self, enable: bool) -> Self {
        if enable {
            self.message.header.flags &= !IFF_NOARP;
        } else {
            self.message.header.flags |= IFF_NOARP;
        }
        self.message.header.change_mask |= IFF_NOARP;
        self
    }

    /// Set the name of the link with the given index (equivalent to `ip link
    /// set DEV name NAME`)
    pub fn name(mut self, name: String) -> Self {
        self.message.nlas.push(Nla::IfName(name));
        self
    }

    /// Set the mtu of the link with the given index (equivalent to `ip link set
    /// DEV mtu MTU`)
    pub fn mtu(mut self, mtu: u32) -> Self {
        self.message.nlas.push(Nla::Mtu(mtu));
        self
    }

    /// Set the hardware address of the link with the given index (equivalent to
    /// `ip link set DEV address ADDRESS`)
    pub fn address(mut self, address: Vec<u8>) -> Self {
        self.message.nlas.push(Nla::Address(address));
        self
    }

    /// Move this network device into the network namespace of the process with
    /// the given `pid`.
    pub fn setns_by_pid(mut self, pid: u32) -> Self {
        self.message.nlas.push(Nla::NetNsPid(pid));
        self
    }

    /// Move this network device into the network namespace corresponding to the
    /// given file descriptor.
    pub fn setns_by_fd(mut self, fd: RawFd) -> Self {
        self.message.nlas.push(Nla::NetNsFd(fd));
        self
    }
    /// Create a new bond.
    /// This is equivalent to `ip link add link NAME type bond`.
    pub fn bondport(self, name: String) -> BondPortSetRequest {
        eprintln!("bondport starting");
        let s = self.name(name);
        BondPortSetRequest {
            request: s,
            info_slave_data: vec![],
        }
    }

    /// Replace existing matching link.
    pub fn replace(self) -> Self {
        Self {
            replace: true,
            ..self
        }
    }

    fn link_info(self, slavekind: InfoSlaveKind, slavedata: Option<InfoSlaveData>) -> Self {
        let mut link_info_nlas = vec![Info::SlaveKind(slavekind)];
        if let Some(slavedata) = slavedata {
            link_info_nlas.push(Info::SlaveData(slavedata));
        }
        eprintln!("{:?}", link_info_nlas);
        self.append_nla(Nla::Info(link_info_nlas))
    }
    fn append_nla(mut self, nla: Nla) -> Self {
        eprintln!("******append_nla*******");
        eprintln!("{:?}", nla);
        self.message.nlas.push(nla);
        eprintln!("******message_nlas*******");
        eprintln!("{:?}", self.message.nlas);
        self
    }

}
