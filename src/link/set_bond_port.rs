// SPDX-License-Identifier: MIT

use netlink_packet_core::{NetlinkMessage, NLM_F_ACK, NLM_F_REQUEST};
use netlink_packet_route::{
    link::nlas::{Info, InfoBondPort, InfoPortData, InfoPortKind, Nla},
    LinkMessage, RtnlMessage,
};

use crate::{Error, LinkSetRequest};

pub struct BondPortSetRequest {
    pub(crate) request: LinkSetRequest,
    pub(crate) info_port_data: Vec<InfoBondPort>,
}

impl BondPortSetRequest {
    /// Execute the request.
    pub async fn execute(mut self) -> Result<(), Error> {
        self.port_link_info(
            InfoPortKind::Bond,
            Some(InfoPortData::BondPort(self.info_port_data.clone())),
        );
        self.request.execute().await
    }

    /// Adds the `queue_id` attribute to the bond port
    /// This is equivalent to `ip link set name NAME type bond_slave queue_id
    /// QUEUE_ID`.
    pub fn queue_id(mut self, queue_id: u16) -> Self {
        self.info_port_data.push(InfoBondPort::QueueId(queue_id));
        self
    }

    /// Adds the `prio` attribute to the bond port
    /// This is equivalent to `ip link set name NAME type bond_slave prio PRIO`.
    pub fn prio(mut self, prio: i32) -> Self {
        self.info_port_data.push(InfoBondPort::Prio(prio));
        self
    }

    pub(crate) fn generate_netlink_message(
        msg: LinkMessage,
    ) -> NetlinkMessage<RtnlMessage> {
        let mut ret = NetlinkMessage::from(RtnlMessage::NewLink(msg));
        ret.header.flags = NLM_F_REQUEST | NLM_F_ACK;
        ret
    }

    pub(crate) fn is_bond_port(message: &LinkMessage) -> bool {
        let port_data = message.nlas.iter().find_map(|nla| match nla {
            Nla::Info(info_data) => {
                info_data.iter().find_map(|info| match info {
                    Info::PortData(data) => Some(data),
                    _ => None,
                })
            }
            _ => None,
        });

        if let Some(InfoPortData::BondPort(_)) = port_data {
            return true;
        } else {
            return false;
        };
    }

    pub fn port_link_info(
        &mut self,
        portkind: InfoPortKind,
        portdata: Option<InfoPortData>,
    ) {
        let mut link_info_nlas = vec![Info::PortKind(portkind)];
        if let Some(portdata) = portdata {
            link_info_nlas.push(Info::PortData(portdata));
        }
        self.request
            .message_mut()
            .nlas
            .push(Nla::Info(link_info_nlas));
    }
}
