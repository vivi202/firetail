use crate::cidr::{CidrIpFilter, IpCidr};
use crate::port_filter::{PortFilter, Ports};
use crate::TimestampedLog;
use senpa::{Action, ProtoName};

#[derive(Debug, Default)]
pub struct Filter {
    actions: Option<Vec<Action>>,
    protocols: Option<Vec<ProtoName>>,
    interfaces: Option<Vec<String>>,
    src_ips: Option<Vec<IpCidr>>,
    dst_ips: Option<Vec<IpCidr>>,
    src_ips_tree: CidrIpFilter,
    dst_ips_tree: CidrIpFilter,
    src_ports: PortFilter,
    dst_ports: PortFilter,
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_proto(&mut self, proto: ProtoName) {
        match self.protocols {
            Some(ref mut protocols) => protocols.push(proto),
            None => self.protocols = Some(vec![proto]),
        }
    }

    pub fn add_action(&mut self, action: Action) {
        match self.actions {
            Some(ref mut actions) => actions.push(action),
            None => self.actions = Some(vec![action]),
        }
    }

    pub fn add_src_ip(&mut self, ip: IpCidr) {
        match self.src_ips {
            Some(ref mut src) => {
                src.push(ip.clone());
            }
            None => self.src_ips = Some(vec![ip.clone()]),
        }
        self.src_ips_tree.insert(ip);
    }

    pub fn add_dst_ip(&mut self, ip: IpCidr) {
        match self.dst_ips {
            Some(ref mut dst) => {
                dst.push(ip.clone());
            }
            None => self.dst_ips = Some(vec![ip.clone()]),
        }
        self.dst_ips_tree.insert(ip);
    }

    pub fn add_src_port(&mut self, port: Ports) {
        self.src_ports.insert(port);
    }

    pub fn add_dst_port(&mut self, port: Ports) {
        self.dst_ports.insert(port);
    }

    pub fn add_interface(&mut self, interface: String) {
        match self.interfaces {
            Some(ref mut interfaces) => interfaces.push(interface),
            None => self.interfaces = Some(vec![interface]),
        }
    }

    pub fn test(&self, log: &TimestampedLog) -> bool {
        let mut ok = true;

        if let Some(protocols) = &self.protocols {
            ok &= protocols.contains(&log.log.protocol.name);
        }

        if let Some(interfaces) = &self.interfaces {
            ok &= interfaces.contains(&log.log.packet_filter.interface);
        }

        if let Some(actions) = &self.actions {
            ok &= actions.contains(&log.log.packet_filter.action);
        }

        if self.src_ips.is_some() {
            ok &= self.src_ips_tree.lookup(&log.log.ip_data.src);
        }

        if self.dst_ips.is_some() {
            ok &= self.dst_ips_tree.lookup(&log.log.ip_data.dst);
        }

        if !self.src_ports.is_empty() {
            let port = match &log.log.proto_info {
                senpa::ProtoInfo::UdpInfo(udp_info) => udp_info.ports.srcport,
                senpa::ProtoInfo::TcpInfo(tcp_info) => tcp_info.ports.srcport,
                senpa::ProtoInfo::UnknownInfo(_) => return false,
            };
            ok &= self.src_ports.contains(port);
        }

        if !self.dst_ports.is_empty() {
            let port = match &log.log.proto_info {
                senpa::ProtoInfo::UdpInfo(udp_info) => udp_info.ports.dstport,
                senpa::ProtoInfo::TcpInfo(tcp_info) => tcp_info.ports.dstport,
                senpa::ProtoInfo::UnknownInfo(_) => return false,
            };

            ok &= self.dst_ports.contains(port);
        }

        ok
    }
}
