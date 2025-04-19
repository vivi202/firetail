use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Eq, PartialEq)]
pub struct ParsePortError;

#[derive(Debug, Eq, PartialEq)]
pub enum Ports {
    Port(u16),
    PortRange(u16, u16),
}

impl FromStr for Ports {
    type Err = ParsePortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("-") {
            Some((start_str, end_str)) => {
                let start = u16::from_str(start_str).map_err(|_| ParsePortError)?;
                let end = u16::from_str(end_str).map_err(|_| ParsePortError)?;
                Ok(Ports::PortRange(start, end))
            }
            None => {
                let port = u16::from_str(s).map_err(|_| ParsePortError)?;
                Ok(Ports::Port(port))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_range() {
        let port_range_str_valid = "80-443";
        assert_eq!(
            Ports::from_str(&port_range_str_valid),
            Ok(Ports::PortRange(80, 443))
        );
        let invalid0 = "a-443";
        let invalid1 = "443-b";
        let invalid2 = "a-";
        let invalid3 = "-b";
        let invalid4 = "a-b";
        let invalid5 = "-";
        assert_eq!(Ports::from_str(&invalid0), Err(ParsePortError));
        assert_eq!(Ports::from_str(&invalid1), Err(ParsePortError));
        assert_eq!(Ports::from_str(&invalid2), Err(ParsePortError));
        assert_eq!(Ports::from_str(&invalid3), Err(ParsePortError));
        assert_eq!(Ports::from_str(&invalid4), Err(ParsePortError));
        assert_eq!(Ports::from_str(&invalid5), Err(ParsePortError));
    }

    #[test]
    fn test_parse_port() {
        let port_str = "80";
        assert_eq!(Ports::from_str(&port_str), Ok(Ports::Port(80)))
    }
}

#[derive(Debug,Default, Eq, PartialEq)]
pub struct PortFilter {
    port_map: BTreeMap<u16, u16>,
}

impl PortFilter {

    pub fn insert_range(&mut self, start: u16, end: u16) {
        self.port_map.insert(start, end);
    }

    pub fn insert(&mut self,ports: Ports) {
        match ports {
            Ports::Port(port) => self.insert_single(port),
            Ports::PortRange(start, end) => self.insert_range(start,end),
        }
    }

    pub fn insert_single(&mut self, port: u16) {
        self.port_map.insert(port, port);
    }

    pub fn contains(&self, port: u16) -> bool {
        if let Some((_, &end)) = self.port_map.range(..=port).next_back() {
            port <= end
        } else {
           false
        }
    }
    pub fn is_empty(&self) -> bool {
        self.port_map.is_empty()
    }   
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_single() {
        let mut filter = PortFilter::default();
        filter.insert_single(443);
        filter.insert_single(80);
        assert_eq!(filter.contains(443), true);
        assert_eq!(filter.contains(80), true);
        assert_eq!(filter.contains(22), false);
    }
    #[test]
    fn test_port_range() {
        let mut filter = PortFilter::default();
        filter.insert_range(80,443);
        for x in 80..=443 {
        assert_eq!(filter.contains(x),true);
        }
        assert_eq!(filter.contains(67), false);
    }
}
