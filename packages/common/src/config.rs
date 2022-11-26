

use serde::{Deserialize};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;

use crate::cipher::XorCipher;
use crate::net::proto::ProtocolMode;

