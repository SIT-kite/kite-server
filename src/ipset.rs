//! This module has an IpSet structure, which could import CIDR whitelist, and
//! detect whether one IP can access rapidly.
// TODO: Add cache to speed up more.
// TODO: Try to use HashMap to separate '/8' sections.

use regex::Regex;

/// A single CIDR whitelist record.<br>
/// A common CIDR is like `"127.0.0.0/24"`,
/// so addr_prefix will be `to_u32("127.0.0.1")` as n is `24`.
#[derive(Clone)]
struct IpCIDR {
    addr_prefix: u32,
    n: u8,
}

/// A set of IP whitelist
#[derive(Clone)]
pub struct IpSet {
    ip_list: Vec<IpCIDR>,
}

impl IpCIDR {
    /// Create a new CIDR range.
    pub fn new(prefix: u32, n: u8) -> Self {
        // FIX: Standard CIDR need to express the range of network address, but I encountered that
        // `127.0.0.1` doesn't meets `127.0.0.1/24`. Although the last `1` is not needed, we should
        // fix this issue.
        let bin_prefix = (0xFFFFFFFF00000000_u64 >> n) as u32;
        let prefix = prefix & bin_prefix;

        Self {
            addr_prefix: prefix,
            n,
        }
    }

    /// To check if addr is in range.
    pub fn contain(&self, addr: u32) -> bool {
        addr & (0xFFFFFFFF00000000_u64 >> self.n) as u32 == self.addr_prefix
    }
}

impl IpSet {
    /// Create an empty ip set.
    pub fn new() -> Self {
        Self { ip_list: Vec::new() }
    }

    /// Load ip set from string and users is required to ensure that the format they provide is correct.
    pub fn load(&mut self, text: &str) {
        let re = Regex::new(r"(\d{1,3}).(\d{1,3}).(\d{1,3}).(\d{1,3})/(\d+)").unwrap();
        for cap in re.captures_iter(text) {
            self.ip_list.push(IpCIDR::new(
                (cap[1].parse::<u32>().unwrap() << 24)
                    + (cap[2].parse::<u32>().unwrap() << 16)
                    + (cap[3].parse::<u32>().unwrap() << 8)
                    + cap[4].parse::<u32>().unwrap(),
                cap[5].parse::<u32>().unwrap() as u8,
            ));
        }
    }

    /// Check is contain or not.
    pub fn contain(&self, addr: u32) -> bool {
        for cidr_range in self.ip_list.iter() {
            if cidr_range.contain(addr) {
                return true;
            }
        }
        false
    }
}

#[inline]
pub fn convert_ipv4_addr_to_u32(ipv4_addr: &[u8; 4]) -> u32 {
    ((ipv4_addr[0] as u32) << 24)
        + ((ipv4_addr[1] as u32) << 16)
        + ((ipv4_addr[2] as u32) << 8)
        + ipv4_addr[3] as u32
}
