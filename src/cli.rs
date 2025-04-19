use std::str::FromStr;

use clap::Parser;
use senpa::{Action, ProtoName};

use crate::{cidr::IpCidr, filter::Filter, port_filter::Ports};
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub logfile: Option<String>,
    #[clap(short, value_delimiter = ',')]
    pub interfaces: Vec<String>,
    #[clap(short, value_delimiter = ',')]
    pub protocols: Vec<String>,
    #[clap(short, value_delimiter = ',')]
    pub actions: Vec<String>,
    #[clap(long = "src-ip", value_delimiter = ',')]
    pub src_ip: Vec<String>,
    #[clap(long = "dst-ip", value_delimiter = ',')]
    pub dst_ip: Vec<String>,
    #[clap(long = "src-port", value_delimiter = ',')]
    pub src_port: Vec<String>,
    #[clap(long = "dst-port", value_delimiter = ',')]
    pub dst_port: Vec<String>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Eq, PartialEq)]
pub enum FilterError {
    InvalidProto(String),
    InvalidAction(String),
    InvalidSrcIp(String),
    InvalidDstIp(String),
    InvalidSrcPort(String),
    InvalidDstPort(String),
}

impl Cli {
    pub fn build_filter(&self) -> Result<Filter, FilterError> {
        let mut filter = Filter::new();

        self.interfaces
            .iter()
            .for_each(|interface| filter.add_interface(interface.to_lowercase().clone()));

        for proto_str in &self.protocols {
            let proto = ProtoName::from_str(proto_str)
                .map_err(|_| FilterError::InvalidProto(proto_str.into()))?;
            filter.add_proto(proto);
        }

        for action_str in &self.actions {
            let action = Action::from_str(action_str)
                .map_err(|_| FilterError::InvalidAction(action_str.into()))?;
            filter.add_action(action);
        }

        for ip_str in &self.src_ip {
            let ip =
                IpCidr::from_str(ip_str).map_err(|_| FilterError::InvalidSrcIp(ip_str.into()))?;
            filter.add_src_ip(ip);
        }

        for ip_str in &self.dst_ip {
            let ip =
                IpCidr::from_str(ip_str).map_err(|_| FilterError::InvalidDstIp(ip_str.into()))?;
            filter.add_dst_ip(ip);
        }

        for port_str in &self.src_port {
            let port = Ports::from_str(port_str)
                .map_err(|_| FilterError::InvalidSrcPort(port_str.into()))?;
            filter.add_src_port(port);
        }

        for port_str in &self.dst_port {
            let port = Ports::from_str(port_str)
                .map_err(|_| FilterError::InvalidDstPort(port_str.into()))?;
            filter.add_dst_port(port);
        }

        Ok(filter)
    }
}
