use std::{net::IpAddr, str::FromStr};

use clap::Parser;
use senpa::{Action, ProtoName};

use crate::filter::Filter;
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
    #[clap(short, value_delimiter = ',')]
    pub src: Vec<String>,
    #[clap(short, value_delimiter = ',')]
    pub dst: Vec<String>,
}

impl Cli {
    pub fn build_filter(&self) -> Filter {
        let mut filter = Filter::new();

        self.interfaces
            .iter()
            .for_each(|interface| filter.add_interface(interface.to_lowercase().clone()));

        self.protocols.iter().for_each(|proto_str| {
            if let Ok(proto) = ProtoName::from_str(proto_str) {
                filter.add_proto(proto);
            }
        });

        self.actions.iter().for_each(|action_str| {
            if let Ok(action) = Action::from_str(action_str) {
                filter.add_action(action);
            }
        });

        self.src.iter().for_each(|ip_str| {
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                filter.add_src(ip);
            }
        });

        self.dst.iter().for_each(|ip_str| {
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                filter.add_dst(ip);
            }
        });

        filter
    }
}
