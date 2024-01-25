// Copyright 2024 Brandon Matthews <thenewwazoo@optimaltour.us>

const IMPROV_VERSION: u8 = 0x01;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImprovPacket {
    CurrentState(CurrentState),
    ErrorState(ErrorState),
    RPCCommand(RPCCommand),
    RPCResult(RPCResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CurrentState {
    Ready,
    Provisioning,
    Provisioned,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ErrorState {
    NoError,
    InvalidRPCPacket,
    UnknownRPCCommand,
    UnableToConnect,
    UnknownError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RPCCommand {
    SendWifiSettings(WifiSettings),
    RequestCurrentState,
    RequestDeviceInformation,
    RequestScannedWifiNetworks,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WifiSettings {
    pub ssid: String,
    pub psk: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RPCResult(Vec<Vec<u8>>);

trait TypedPacket {
    const TYPE: u8;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImprovErr {
    InvalidCurrentStateByte,
    InvalidErrorStateByte,
    InvalidRPCCommand,
    NotAnImprovPacket,
    BadLength,
    UnsupportedVersion,
    GoAway,
}

impl TypedPacket for CurrentState {
    const TYPE: u8 = 0x01;
}

impl From<CurrentState> for u8 {
    fn from(c: CurrentState) -> u8 {
        match c {
            CurrentState::Ready => 0x2,
            CurrentState::Provisioning => 0x3,
            CurrentState::Provisioned => 0x4,
        }
    }
}

impl TryFrom<u8> for CurrentState {
    type Error = ImprovErr;

    fn try_from(b: u8) -> Result<CurrentState, ImprovErr> {
        match b {
            0x2 => Ok(CurrentState::Ready),
            0x3 => Ok(CurrentState::Provisioning),
            0x4 => Ok(CurrentState::Provisioned),
            _ => Err(ImprovErr::InvalidCurrentStateByte),
        }
    }
}

impl TypedPacket for ErrorState {
    const TYPE: u8 = 0x02;
}

impl From<ErrorState> for u8 {
    fn from(e: ErrorState) -> u8 {
        match e {
            ErrorState::NoError => 0x00,
            ErrorState::InvalidRPCPacket => 0x01,
            ErrorState::UnknownRPCCommand => 0x02,
            ErrorState::UnableToConnect => 0x03,
            ErrorState::UnknownError => 0xFF,
        }
    }
}

impl TryFrom<u8> for ErrorState {
    type Error = ImprovErr;

    fn try_from(b: u8) -> Result<ErrorState, ImprovErr> {
        match b {
            0x00 => Ok(ErrorState::NoError),
            0x01 => Ok(ErrorState::InvalidRPCPacket),
            0x02 => Ok(ErrorState::UnknownRPCCommand),
            0x03 => Ok(ErrorState::UnableToConnect),
            0xFF => Ok(ErrorState::UnknownError),
            _ => Err(ImprovErr::InvalidErrorStateByte),
        }
    }
}

impl TypedPacket for RPCCommand {
    const TYPE: u8 = 0x03;
}

impl RPCCommand {
    fn inner(self) -> Vec<u8> {
        match self {
            RPCCommand::SendWifiSettings(w) => {
                let mut inner: Vec<u8> = w.into();
                let mut r = vec![0x01, inner.len() as u8];
                r.append(&mut inner);
                r
            }
            RPCCommand::RequestCurrentState => vec![0x02, 0x00],
            RPCCommand::RequestDeviceInformation => vec![0x03, 0x00],
            RPCCommand::RequestScannedWifiNetworks => vec![0x04, 0x00],
        }
    }
}

impl TryFrom<Vec<u8>> for RPCCommand {
    type Error = ImprovErr;

    fn try_from(b: Vec<u8>) -> Result<RPCCommand, ImprovErr> {
        match b[0] {
            0x01 => {
                if b[1] as usize != b.len() - 2 {
                    return Err(ImprovErr::BadLength);
                }

                let ssid = unsafe { String::from_utf8_unchecked(b[3..(b[2] as usize)].to_vec()) };
                let psk = unsafe { String::from_utf8_unchecked(b[(3 + b[2] as usize)..].to_vec()) };

                Ok(RPCCommand::SendWifiSettings(WifiSettings { ssid, psk }))
            }
            0x02 => Ok(RPCCommand::RequestCurrentState),
            0x03 => Ok(RPCCommand::RequestDeviceInformation),
            0x04 => Ok(RPCCommand::RequestScannedWifiNetworks),
            _ => Err(ImprovErr::InvalidRPCCommand),
        }
    }
}

impl From<WifiSettings> for Vec<u8> {
    fn from(w: WifiSettings) -> Vec<u8> {
        vec![
            vec![w.ssid.len() as u8],
            w.ssid.into_bytes(),
            vec![w.psk.len() as u8],
            w.psk.into_bytes(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl TypedPacket for RPCResult {
    const TYPE: u8 = 0x04;
}

impl RPCResult {
    fn inner(self) -> Vec<u8> {
        self.0
            .into_iter()
            .map(|mut v| {
                v.insert(0, v.len() as u8);
                v
            })
            .flatten()
            .collect()
    }
}

impl ImprovPacket {
    fn inner(self) -> Vec<u8> {
        match self {
            ImprovPacket::CurrentState(c) => vec![c.into()],
            ImprovPacket::ErrorState(e) => vec![e.into()],
            ImprovPacket::RPCCommand(c) => c.inner(),
            ImprovPacket::RPCResult(r) => r.inner(),
        }
    }

    fn pkt_type(&self) -> u8 {
        match self {
            ImprovPacket::CurrentState(_) => CurrentState::TYPE,
            ImprovPacket::ErrorState(_) => ErrorState::TYPE,
            ImprovPacket::RPCCommand(_) => RPCCommand::TYPE,
            ImprovPacket::RPCResult(_) => RPCResult::TYPE,
        }
    }
}

impl From<ImprovPacket> for Vec<u8> {
    fn from(p: ImprovPacket) -> Vec<u8> {
        let pkt_type = p.pkt_type();
        let inner = p.inner();
        let mut data: Vec<u8> = vec![
            String::from("IMPROV").into_bytes(),
            vec![
                IMPROV_VERSION,
                pkt_type,
                inner.len() as u8, // data len
            ],
            inner,
        ]
        .into_iter()
        .flatten()
        .collect();
        data.push(checksum(&data));
        data
    }
}

fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |s, &n| s.wrapping_add(n))
}

impl TryFrom<Vec<u8>> for ImprovPacket {
    type Error = ImprovErr;

    fn try_from(mut b: Vec<u8>) -> Result<ImprovPacket, ImprovErr> {
        if &b[0..6] != "IMPROV".as_bytes() {
            return Err(ImprovErr::NotAnImprovPacket);
        }

        if b[6] != IMPROV_VERSION {
            return Err(ImprovErr::UnsupportedVersion);
        }

        if b[8] as usize != b.len() - 10 {
            return Err(ImprovErr::BadLength);
        }

        // TODO validate checksum

        match b[7] {
            CurrentState::TYPE => Ok(ImprovPacket::CurrentState(CurrentState::try_from(b[9])?)),
            ErrorState::TYPE => Ok(ImprovPacket::ErrorState(ErrorState::try_from(b[9])?)),
            RPCCommand::TYPE => Ok(ImprovPacket::RPCCommand(RPCCommand::try_from({
                let mut data = b.split_off(9);
                data.pop(); // remove the checksum
                data
            })?)),
            //RPCResult::TYPE => {},
            _ => Err(ImprovErr::GoAway),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_get_current_state() {
        let v: Vec<u8> = vec![
            0x49, 0x4D, 0x50, 0x52, 0x4F, 0x56, 0x01, 0x03, 0x02, 0x02, 0x00, 0xE5,
        ];
        assert_eq!(
            ImprovPacket::try_from(v),
            Ok(ImprovPacket::RPCCommand(RPCCommand::RequestCurrentState)),
        );
    }

    #[test]
    fn build_get_current_state() {
        let p = ImprovPacket::RPCCommand(RPCCommand::RequestCurrentState);
        assert_eq!(
            vec![0x49, 0x4D, 0x50, 0x52, 0x4F, 0x56, 0x01, 0x03, 0x02, 0x02, 0x00, 0xE5],
            <ImprovPacket as Into<Vec<u8>>>::into(p),
        );
    }

    #[test]
    fn build_get_device_info() {
        let p = ImprovPacket::RPCCommand(RPCCommand::RequestDeviceInformation);
        assert_eq!(
            vec![0x49, 0x4D, 0x50, 0x52, 0x4F, 0x56, 0x01, 0x03, 0x02, 0x03, 0x00, 0xE6],
            <ImprovPacket as Into<Vec<u8>>>::into(p),
        );
    }

    #[test]
    fn build_get_networks() {
        let p = ImprovPacket::RPCCommand(RPCCommand::RequestScannedWifiNetworks);
        assert_eq!(
            vec![0x49, 0x4D, 0x50, 0x52, 0x4F, 0x56, 0x01, 0x03, 0x02, 0x04, 0x00, 0xE7],
            <ImprovPacket as Into<Vec<u8>>>::into(p),
        );
    }

    #[test]
    fn build_send_wifi() {
        let p = ImprovPacket::RPCCommand(RPCCommand::SendWifiSettings(WifiSettings {
            ssid: String::from("anthill"),
            psk: String::from("ants in my pants"),
        }));
        assert_eq!(
            vec![
                0x49, 0x4D, 0x50, 0x52, 0x4F, 0x56, 0x01, 0x03, 0x1B, 0x01, 0x19, 0x07, 0x61, 0x6E,
                0x74, 0x68, 0x69, 0x6C, 0x6C, 0x10, 0x61, 0x6E, 0x74, 0x73, 0x20, 0x69, 0x6E, 0x20,
                0x6D, 0x79, 0x20, 0x70, 0x61, 0x6E, 0x74, 0x73, 0x12
            ],
            <ImprovPacket as Into<Vec<u8>>>::into(p),
        );
    }
}
