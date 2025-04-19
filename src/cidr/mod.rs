use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct IpCidr {
    pub addr: IpAddr,
    pub net_bits: u8,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseIpCidrError;

impl IpCidr {
    pub fn new(addr: IpAddr, net_bits: u8) -> Self {
        Self { addr, net_bits }
    }
}

impl FromStr for IpCidr {
    type Err = ParseIpCidrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("/") {
            Some((ip_string, net_string)) => {
                let ip = IpAddr::from_str(ip_string).map_err(|_| ParseIpCidrError)?;
                let net = u8::from_str(net_string).map_err(|_| ParseIpCidrError)?;
                match ip {
                    IpAddr::V4(_) => {
                        if net > 32 {
                            return Err(ParseIpCidrError);
                        }
                        Ok(IpCidr::new(ip, net))
                    }
                    IpAddr::V6(_) => {
                        if net > 128 {
                            return Err(ParseIpCidrError);
                        }
                        Ok(IpCidr::new(ip, net))
                    }
                }
            }
            None => {
                let ip = IpAddr::from_str(s).map_err(|_| ParseIpCidrError)?;
                match ip {
                    IpAddr::V4(_) => Ok(IpCidr::new(ip, 32)),
                    IpAddr::V6(_) => Ok(IpCidr::new(ip, 128)),
                }
            }
        }
    }
}

//Simple implementation of radix-tree to perform ip address lookup
//Works with both ipv4 and ipv6
#[derive(Debug, Default)]
struct CidrTreeNode {
    is_terminal: bool,
    childrens: [Option<Box<Self>>; 2],
}

#[derive(Debug, Default)]
pub struct CidrTree {
    root: Option<CidrTreeNode>,
}

fn get_octets(ip_addr: &IpAddr) -> Vec<u8> {
    match ip_addr {
        IpAddr::V4(ipv4_addr) => ipv4_addr.octets().into(),
        IpAddr::V6(ipv6_addr) => ipv6_addr.octets().into(),
    }
}

impl CidrTree{

    pub fn insert(&mut self, ip_cidr: IpCidr) {
        let mut current_node = self.root.get_or_insert(CidrTreeNode::default());
        let net_bits = ip_cidr.net_bits;

        if net_bits == 0 {
            // /0 matches all address space
            current_node.childrens[0]
                .get_or_insert(Box::default())
                .is_terminal = true;
            current_node.childrens[1]
                .get_or_insert(Box::default())
                .is_terminal = true;
            return;
        }

        let octets = get_octets(&ip_cidr.addr);

        let mut inserted_bits = 0;

        for octect in octets {
            //Iterate bits of octet
            for bit_pos in 0..8 {
                let bit = (octect >> bit_pos) & 0x01;
                current_node = current_node.childrens[bit as usize].get_or_insert(Box::default());
                inserted_bits += 1;

                if inserted_bits == net_bits {
                    current_node.is_terminal = true;
                    return;
                }
            }
        }
    }

    pub fn lookup(&self, addr: &IpAddr) -> bool {
        if self.root.is_none() {
            return false;
        }

        let mut current_node = self.root.as_ref();
        let octets = get_octets(addr);

        for octect in octets {
            //Iterate bits of octet
            for bit_pos in 0..8 {
                let bit = (octect >> bit_pos) & 0x01;
                current_node = current_node.and_then(|x| x.childrens[bit as usize].as_deref());
                match current_node {
                    Some(node) => {
                        if node.is_terminal {
                            return true;
                        }
                    }
                    None => return false,
                }
            }
        }

        false
    }
}

#[derive(Default,Debug)]
pub struct CidrIpFilter {
    ipv4: CidrTree,
    ipv6: CidrTree,
}

impl CidrIpFilter {
    pub fn insert(&mut self, ip_cidr: IpCidr) {
        match ip_cidr.addr {
            IpAddr::V4(_) => {
                self.ipv4.insert(ip_cidr);
            }
            IpAddr::V6(_) => {
                self.ipv6.insert(ip_cidr);
            }
        }
    }

    pub fn lookup(&self, addr: &IpAddr) -> bool {
        match addr {
            IpAddr::V4(_) => self.ipv4.lookup(addr),
            IpAddr::V6(_) => self.ipv6.lookup(addr),
        }
    }
}

#[cfg(test)]
mod ipv4_tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn trie_single_address_ipv4() {
        let mut trie = CidrTree::default();

        // Insert some IPv4 addresses
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 10, 10)), 32));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 12, 10)), 32));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 32));

        // Test exact matches
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 10))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 12, 10))),
            true
        );

        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))), true);

        // Test non-matches
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 11))),
            false
        );

        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2))), false);
    }

    #[test]
    fn subnet_ipv4() {
        let mut trie = CidrTree::default();

        // Insert some IPv4 subnets
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 10, 0)), 24));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8));

        // Test addresses within the /24 subnet
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 1))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 255))),
            true
        );

        // Test addresses within the /8 subnet
        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3))), true);

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255))),
            true
        );

        // Test addresses outside the subnets
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 11, 1))),
            false
        );

        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(11, 0, 0, 0))), false);
    }

    #[test]
    fn ipv4_empty_trie() {
        let trie = CidrTree::default();

        // Test lookup in empty trie
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
            false
        );
    }

    #[test]
    fn ipv4_various_prefix_lengths() {
        let mut trie = CidrTree::default();

        // Insert subnets with different prefix lengths
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 0)), 16));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(172, 16, 32, 0)), 20));

        // Test matches at different levels
        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(172, 16, 1, 1))), true);

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(172, 16, 32, 1))),
            true
        );

        // This should match the /16 but not the /20
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(172, 16, 64, 1))),
            true
        );

        // This should not match any subnet
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(172, 17, 0, 0))),
            false
        );
    }

    #[test]
    fn ipv4_overlapping_prefixes() {
        let mut trie = CidrTree::default();

        // Insert subnets with overlapping address spaces
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)), 16));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 10, 0)), 24));
        trie.insert(IpCidr::new(
            IpAddr::V4(Ipv4Addr::new(192, 168, 10, 128)),
            25,
        ));

        // Test address matching the most specific prefix (longest match)
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 130))),
            true
        );

        // Test address matching the middle prefix
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 10))),
            true
        );

        // Test address matching only the least specific prefix
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 20, 1))),
            true
        );

        // Test address not matching any prefix
        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 169, 0, 0))),
            false
        );
    }

    #[test]
    fn ipv4_edge_cases() {
        let mut trie = CidrTree::default();

        // Test with extreme cases

        // Single IP (/32)
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 32));

        // Entire IPv4 address space (/0)
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0));

        // Test the single IP
        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))), true);

        // A different IP should still match because of the /0 prefix
        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))), true);

        // Clear the trie and test with broadcast address
        let mut trie = CidrTree::default();
        trie.insert(IpCidr::new(
            IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
            32,
        ));

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(255, 255, 255, 254))),
            false
        );
    }

    #[test]
    fn ipv4_from_string_parsing() {
        // Test the FromStr implementation for IpCidr

        // Valid cases
        let cidr1 = "192.168.10.0/24".parse::<IpCidr>().unwrap();
        assert_eq!(cidr1.addr, IpAddr::V4(Ipv4Addr::new(192, 168, 10, 0)));
        assert_eq!(cidr1.net_bits, 24);

        let cidr2 = "10.0.0.1".parse::<IpCidr>().unwrap();
        assert_eq!(cidr2.addr, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(cidr2.net_bits, 32); // Default for single IP

        // Invalid cases
        assert_eq!("192.168.10.0/33".parse::<IpCidr>(), Err(ParseIpCidrError));
        assert_eq!("300.168.10.0/24".parse::<IpCidr>(), Err(ParseIpCidrError));
        assert_eq!("192.168.10.0/abc".parse::<IpCidr>(), Err(ParseIpCidrError));

        // Test with trie
        let mut trie = CidrTree::default();
        trie.insert("192.168.10.0/24".parse::<IpCidr>().unwrap());

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 10, 123))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 11, 1))),
            false
        );
    }

    #[test]
    fn ipv4_multiple_overlapping_subnets() {
        let mut trie = CidrTree::default();

        // Create a series of overlapping subnets
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)), 8));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 10, 0, 0)), 16));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 10, 10, 0)), 24));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 10, 10, 128)), 25));
        trie.insert(IpCidr::new(IpAddr::V4(Ipv4Addr::new(10, 10, 10, 254)), 32));

        // All should match
        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 1, 1, 1))), true);

        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 10, 1, 1))), true);

        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 10, 10, 1))), true);

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 10, 10, 130))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V4(Ipv4Addr::new(10, 10, 10, 254))),
            true
        );

        // Outside of all subnets
        assert_eq!(trie.lookup(&IpAddr::V4(Ipv4Addr::new(11, 0, 0, 0))), false);
    }
}

// Unit tests for IPv6 functionality
#[cfg(test)]
mod ipv6_tests {
    use std::net::Ipv6Addr;

    use super::*;

    #[test]
    fn trie_single_address_ipv6() {
        let mut trie = CidrTree::default();

        // Insert some IPv6 addresses
        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x8a2e, 0x370, 0x7334,
            )),
            128,
        ));

        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x8a2e, 0x370, 0x7335,
            )),
            128,
        ));

        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0x0, 0x0, 0x0, 0x202, 0xb3ff, 0xfe1e, 0x8329,
            )),
            128,
        ));

        // Test exact matches
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x8a2e, 0x370, 0x7334
            ))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x8a2e, 0x370, 0x7335
            ))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0x0, 0x0, 0x0, 0x202, 0xb3ff, 0xfe1e, 0x8329
            ))),
            true
        );

        // Test non-matches
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x8a2e, 0x370, 0x7336
            ))),
            false
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0x0, 0x0, 0x0, 0x202, 0xb3ff, 0xfe1e, 0x8330
            ))),
            false
        );
    }

    #[test]
    fn subnet_ipv6() {
        let mut trie = CidrTree::default();

        // Insert a /64 subnet
        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x0, 0x0, 0x0,
            )),
            64,
        ));

        // Insert a /48 subnet
        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(0xfe80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0)),
            48,
        ));

        // Test addresses within the /64 subnet
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x1, 0x2, 0x3, 0x4
            ))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0xffff, 0xffff, 0xffff, 0xffff
            ))),
            true
        );

        // Test addresses within the /48 subnet
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0
            ))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0x0, 0x0, 0xffff, 0x0, 0x0, 0x0, 0x0
            ))),
            true
        );

        // Test addresses outside the subnets
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x1, 0x0, 0x0, 0x0, 0x0
            ))),
            false
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0xfe80, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0
            ))),
            false
        );
    }

    #[test]
    fn ipv6_empty_trie() {
        let trie = CidrTree::default();

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb8, 0x85a3, 0x0, 0x0, 0x8a2e, 0x370, 0x7334
            ))),
            false
        );
    }

    #[test]
    fn ipv6_various_prefix_lengths() {
        let mut trie = CidrTree::default();

        // Insert subnets with different prefix lengths
        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(0x2001, 0x0000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0)),
            32,
        ));

        trie.insert(IpCidr::new(
            IpAddr::V6(Ipv6Addr::new(
                0x20_01, 0xdb_80, 0x00_00, 0x0, 0x0, 0x0, 0x0, 0x0,
            )),
            40,
        ));

        // Test matches at different levels
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0
            ))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb80, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0
            ))),
            true
        );

        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2001, 0xdb80, 0x0001, 0x0, 0x0, 0x0, 0x0, 0x0
            ))),
            true
        );

        // This should not match any subnet
        assert_eq!(
            trie.lookup(&IpAddr::V6(Ipv6Addr::new(
                0x2002, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0
            ))),
            false
        );
    }
}
