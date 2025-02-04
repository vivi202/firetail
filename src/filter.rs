use std::net::IpAddr;

use senpa::{Action, ProtoName};

use crate::TimestampedLog;
#[derive(Debug)]
pub struct Filter {
    actions: Option<Vec<Action>>,
    protocols: Option<Vec<ProtoName>>,
    interfaces: Option<Vec<String>>,
    src: Option<Vec<IpAddr>>,
    dst: Option<Vec<IpAddr>>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            actions: None,
            protocols: None,
            interfaces: None,
            src: None,
            dst: None,
        }
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

    pub fn add_src(&mut self, ip: IpAddr) {
        match self.src {
            Some(ref mut src) => src.push(ip),
            None => self.src = Some(vec![ip]),
        }
    }

    pub fn add_dst(&mut self, ip: IpAddr) {
        match self.dst {
            Some(ref mut dst) => dst.push(ip),
            None => self.dst = Some(vec![ip]),
        }
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

        if let Some(src) = &self.src {
            ok &= src.contains(&log.log.ip_data.src);
        }

        if let Some(dst) = &self.dst {
            ok &= dst.contains(&log.log.ip_data.dst);
        }

        ok
    }
}
