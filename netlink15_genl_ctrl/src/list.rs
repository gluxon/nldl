use crate::attr::ControllerAttribute;
use crate::err::GenlCtrlCommandError;
use netlink15_genl::socket::GenlSocket;
use netlink15_genl::GenericNetlinkHeader;
use netlink15_genl::GenericNetlinkRequest;

pub type ListResult = Result<Vec<Vec<ControllerAttribute>>, GenlCtrlCommandError>;

pub fn list(sock: &GenlSocket) -> ListResult {
    let genl_request = GenericNetlinkRequest {
        header: GenericNetlinkHeader {
            cmd: libc::CTRL_CMD_GETFAMILY as u8,
            version: 0,
        },
        payload: Option::<ControllerAttribute>::None,
    };
    let flags = (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16;

    sock.send(genl_request, flags)?;
    let resp = sock.recv_until_done_buffered::<Vec<ControllerAttribute>>()?;

    // TODO: There's a bit of unnecessary copying here.
    Ok(resp.into_iter().map(|message| message.payload).collect())
}

#[cfg(test)]
mod tests {
    use crate::attr::ControllerAttribute;
    use crate::family::Family;
    use crate::family::FamilyMulticastGroup;
    use crate::family::FamilyOperation;
    use netlink15_core::message::utils::create_message_iterator;
    use netlink15_core::message::NetlinkMessageType;
    use netlink15_genl::GenericNetlinkResponse;

    fn expect_protocol_message<T>(message_type: NetlinkMessageType<T>) -> T {
        match message_type {
            NetlinkMessageType::ProtocolMessage(protocol_message) => protocol_message,
            _ => panic!("There should not be other message types in this test. Only valid Generic Netlink protocol messages were expected."),
        }
    }

    #[test]
    fn test_response_deserialization() -> anyhow::Result<()> {
        // This buffer was received from running "genl ctrl list" on Ubuntu 20.10
        let recv_bytes = vec![
            0x88, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6,
            0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x0B, 0x00, 0x02, 0x00, 0x6E, 0x6C, 0x63, 0x74,
            0x72, 0x6C, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x05, 0x00, 0x08, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x06, 0x00, 0x14, 0x00,
            0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0A, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x07, 0x00,
            0x18, 0x00, 0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00, 0x00, 0x0B, 0x00,
            0x01, 0x00, 0x6E, 0x6F, 0x74, 0x69, 0x66, 0x79, 0x00, 0x00, 0x60, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02,
            0x00, 0x00, 0x0E, 0x00, 0x02, 0x00, 0x56, 0x46, 0x53, 0x5F, 0x44, 0x51, 0x55, 0x4F,
            0x54, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x11, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x05, 0x00, 0x07, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x07, 0x00, 0x18, 0x00,
            0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x11, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x01, 0x00,
            0x65, 0x76, 0x65, 0x6E, 0x74, 0x73, 0x00, 0x00, 0xE4, 0x03, 0x00, 0x00, 0x10, 0x00,
            0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00,
            0x0C, 0x00, 0x02, 0x00, 0x64, 0x65, 0x76, 0x6C, 0x69, 0x6E, 0x6B, 0x00, 0x06, 0x00,
            0x01, 0x00, 0x13, 0x00, 0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x90, 0x00,
            0x00, 0x00, 0x88, 0x03, 0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x02, 0x00, 0x08, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x06, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x05, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x06, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x07, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x10, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x09, 0x00, 0x08, 0x00, 0x01, 0x00, 0x13, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0A, 0x00, 0x08, 0x00, 0x01, 0x00, 0x14, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0B, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x17, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0E, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x0C, 0x00, 0x08, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0D, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x1B, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x0E, 0x00, 0x08, 0x00, 0x01, 0x00, 0x1C, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0F, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x1D, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x10, 0x00, 0x08, 0x00, 0x01, 0x00, 0x1E, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x11, 0x00, 0x08, 0x00, 0x01, 0x00, 0x1F, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00, 0x12, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x20, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x13, 0x00, 0x08, 0x00, 0x01, 0x00, 0x21, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00, 0x14, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x22, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x15, 0x00, 0x08, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x16, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x24, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x17, 0x00, 0x08, 0x00, 0x01, 0x00, 0x25, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x18, 0x00, 0x08, 0x00, 0x01, 0x00, 0x26, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x19, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x27, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x1A, 0x00, 0x08, 0x00, 0x01, 0x00, 0x2F, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x1B, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x30, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x1C, 0x00, 0x08, 0x00, 0x01, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x14, 0x00, 0x1D, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x2C, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x1E, 0x00, 0x08, 0x00, 0x01, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x1F, 0x00, 0x08, 0x00, 0x01, 0x00, 0x2E, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x14, 0x00, 0x20, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x33, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0E, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x21, 0x00, 0x08, 0x00, 0x01, 0x00, 0x34, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x22, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x35, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x23, 0x00, 0x08, 0x00, 0x01, 0x00, 0x36, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x24, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x37, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x25, 0x00, 0x08, 0x00, 0x01, 0x00, 0x38, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0D, 0x00, 0x00, 0x00, 0x14, 0x00, 0x26, 0x00, 0x08, 0x00, 0x01, 0x00, 0x39, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x27, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x28, 0x00, 0x08, 0x00, 0x01, 0x00, 0x3D, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x29, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x3E, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x2A, 0x00, 0x08, 0x00, 0x01, 0x00, 0x41, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x2B, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x42, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x2C, 0x00, 0x08, 0x00, 0x01, 0x00, 0x45, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x2D, 0x00, 0x08, 0x00, 0x01, 0x00, 0x46, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x07, 0x00,
            0x18, 0x00, 0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0B, 0x00,
            0x01, 0x00, 0x63, 0x6F, 0x6E, 0x66, 0x69, 0x67, 0x00, 0x00, 0x7C, 0x02, 0x00, 0x00,
            0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02,
            0x00, 0x00, 0x0C, 0x00, 0x02, 0x00, 0x65, 0x74, 0x68, 0x74, 0x6F, 0x6F, 0x6C, 0x00,
            0x06, 0x00, 0x01, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x20, 0x02, 0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x02, 0x00, 0x08, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x04, 0x00, 0x08, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x05, 0x00, 0x08, 0x00, 0x01, 0x00, 0x05, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00, 0x06, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x06, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x06, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x07, 0x00, 0x08, 0x00, 0x01, 0x00, 0x07, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x09, 0x00, 0x08, 0x00, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x16, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0A, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x0A, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x0B, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0C, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0C, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0D, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x06, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x0E, 0x00, 0x08, 0x00, 0x01, 0x00, 0x0E, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00, 0x0F, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x10, 0x00, 0x08, 0x00, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00, 0x11, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x11, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x12, 0x00, 0x08, 0x00, 0x01, 0x00, 0x12, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x12, 0x00, 0x00, 0x00, 0x14, 0x00, 0x13, 0x00, 0x08, 0x00, 0x01, 0x00, 0x13, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x14, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x15, 0x00, 0x08, 0x00, 0x01, 0x00, 0x15, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x16, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x16, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x17, 0x00, 0x08, 0x00, 0x01, 0x00, 0x17, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x18, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x18, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x19, 0x00, 0x08, 0x00, 0x01, 0x00, 0x19, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x14, 0x00, 0x1A, 0x00, 0x08, 0x00, 0x01, 0x00, 0x1A, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00, 0x00, 0x00, 0x14, 0x00, 0x1B, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x1B, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x12, 0x00,
            0x00, 0x00, 0x1C, 0x00, 0x07, 0x00, 0x18, 0x00, 0x01, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x03, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x01, 0x00, 0x6D, 0x6F, 0x6E, 0x69, 0x74, 0x6F,
            0x72, 0x00, 0xE8, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x0E, 0x00, 0x02, 0x00, 0x4E, 0x4C,
            0x42, 0x4C, 0x5F, 0x4D, 0x47, 0x4D, 0x54, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00,
            0x15, 0x00, 0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x0C, 0x00, 0x00, 0x00,
            0xA4, 0x00, 0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x05, 0x00, 0x08, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x06, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x07, 0x00, 0x08, 0x00, 0x01, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0C, 0x00, 0x00, 0x00, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x01, 0x00, 0x08, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x9C, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02,
            0x00, 0x00, 0x11, 0x00, 0x02, 0x00, 0x4E, 0x4C, 0x42, 0x4C, 0x5F, 0x43, 0x49, 0x50,
            0x53, 0x4F, 0x76, 0x34, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x16, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x54, 0x00,
            0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x9C, 0x00,
            0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00,
            0x01, 0x02, 0x00, 0x00, 0x11, 0x00, 0x02, 0x00, 0x4E, 0x4C, 0x42, 0x4C, 0x5F, 0x43,
            0x41, 0x4C, 0x49, 0x50, 0x53, 0x4F, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00,
            0x17, 0x00, 0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x54, 0x00, 0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0C, 0x00, 0x00, 0x00,
            0xE8, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6,
            0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x0F, 0x00, 0x02, 0x00, 0x4E, 0x4C, 0x42, 0x4C,
            0x5F, 0x55, 0x4E, 0x4C, 0x42, 0x4C, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x18, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x07, 0x00, 0x00, 0x00, 0xA4, 0x00,
            0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x05, 0x00, 0x08, 0x00, 0x01, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x06, 0x00, 0x08, 0x00, 0x01, 0x00, 0x08, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x14, 0x00, 0x07, 0x00,
            0x08, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00,
            0x00, 0x00, 0x14, 0x00, 0x08, 0x00, 0x08, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x10, 0x00,
            0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00,
            0x0F, 0x00, 0x02, 0x00, 0x61, 0x63, 0x70, 0x69, 0x5F, 0x65, 0x76, 0x65, 0x6E, 0x74,
            0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x19, 0x00, 0x00, 0x00, 0x08, 0x00, 0x03, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x05, 0x00, 0x01, 0x00, 0x00, 0x00, 0x24, 0x00, 0x07, 0x00, 0x20, 0x00, 0x01, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x12, 0x00, 0x01, 0x00, 0x61, 0x63,
            0x70, 0x69, 0x5F, 0x6D, 0x63, 0x5F, 0x67, 0x72, 0x6F, 0x75, 0x70, 0x00, 0x00, 0x00,
            0x70, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6,
            0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x74, 0x63, 0x70, 0x5F,
            0x6D, 0x65, 0x74, 0x72, 0x69, 0x63, 0x73, 0x00, 0x06, 0x00, 0x01, 0x00, 0x1A, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x2C, 0x00,
            0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0xE4, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6,
            0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x0D, 0x00, 0x02, 0x00, 0x6D, 0x70, 0x74, 0x63,
            0x70, 0x5F, 0x70, 0x6D, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x1B, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x03, 0x00, 0x00, 0x00, 0x7C, 0x00,
            0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x05, 0x00, 0x08, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x06, 0x00, 0x08, 0x00, 0x01, 0x00, 0x06, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x24, 0x00, 0x07, 0x00,
            0x20, 0x00, 0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x05, 0x00, 0x00, 0x00, 0x12, 0x00,
            0x01, 0x00, 0x6D, 0x70, 0x74, 0x63, 0x70, 0x5F, 0x70, 0x6D, 0x5F, 0x63, 0x6D, 0x64,
            0x73, 0x00, 0x00, 0x00, 0xC4, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x0B, 0x00, 0x02, 0x00,
            0x4E, 0x45, 0x54, 0x5F, 0x44, 0x4D, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x1C, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x16, 0x00, 0x00, 0x00, 0x68, 0x00,
            0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x05, 0x00, 0x08, 0x00, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00,
            0x0A, 0x00, 0x00, 0x00, 0x1C, 0x00, 0x07, 0x00, 0x18, 0x00, 0x01, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x01, 0x00, 0x65, 0x76, 0x65, 0x6E,
            0x74, 0x73, 0x00, 0x00, 0x94, 0x00, 0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x09, 0x00, 0x02, 0x00,
            0x53, 0x45, 0x47, 0x36, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x1D, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x07, 0x00, 0x00, 0x00, 0x54, 0x00,
            0x06, 0x00, 0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0D, 0x00, 0x00, 0x00,
            0x14, 0x00, 0x03, 0x00, 0x08, 0x00, 0x01, 0x00, 0x03, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x04, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x70, 0x00,
            0x00, 0x00, 0x10, 0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3A, 0xC6, 0x00, 0x00,
            0x01, 0x02, 0x00, 0x00, 0x0E, 0x00, 0x02, 0x00, 0x54, 0x41, 0x53, 0x4B, 0x53, 0x54,
            0x41, 0x54, 0x53, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x1E, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x04, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x06, 0x00,
            0x14, 0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00,
            0x02, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x14, 0x00, 0x02, 0x00, 0x08, 0x00, 0x01, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x08, 0x00, 0x02, 0x00, 0x0A, 0x00, 0x00, 0x00,
        ];

        let expected = vec![
            Family {
                family_id: 16,
                family_name: "nlctrl".to_string(),
                version: 2,
                header_size: 0,
                max_attr: 8,
                operations: vec![
                    FamilyOperation { id: 3, flags: 14 },
                    FamilyOperation { id: 10, flags: 12 },
                ],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "notify".to_string(),
                    id: 16,
                }],
            },
            Family {
                family_id: 17,
                family_name: "VFS_DQUOT".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 7,
                operations: vec![],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "events".to_string(),
                    id: 17,
                }],
            },
            Family {
                family_id: 19,
                family_name: "devlink".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 144,
                operations: vec![
                    FamilyOperation { id: 1, flags: 14 },
                    FamilyOperation { id: 5, flags: 14 },
                    FamilyOperation { id: 6, flags: 11 },
                    FamilyOperation { id: 9, flags: 11 },
                    FamilyOperation { id: 10, flags: 11 },
                    FamilyOperation { id: 11, flags: 14 },
                    FamilyOperation { id: 15, flags: 14 },
                    FamilyOperation { id: 16, flags: 11 },
                    FamilyOperation { id: 19, flags: 14 },
                    FamilyOperation { id: 20, flags: 11 },
                    FamilyOperation { id: 23, flags: 14 },
                    FamilyOperation { id: 24, flags: 11 },
                    FamilyOperation { id: 27, flags: 11 },
                    FamilyOperation { id: 28, flags: 11 },
                    FamilyOperation { id: 29, flags: 11 },
                    FamilyOperation { id: 30, flags: 11 },
                    FamilyOperation { id: 31, flags: 10 },
                    FamilyOperation { id: 32, flags: 10 },
                    FamilyOperation { id: 33, flags: 10 },
                    FamilyOperation { id: 34, flags: 11 },
                    FamilyOperation { id: 35, flags: 11 },
                    FamilyOperation { id: 36, flags: 10 },
                    FamilyOperation { id: 37, flags: 11 },
                    FamilyOperation { id: 38, flags: 14 },
                    FamilyOperation { id: 39, flags: 11 },
                    FamilyOperation { id: 47, flags: 14 },
                    FamilyOperation { id: 48, flags: 11 },
                    FamilyOperation { id: 42, flags: 15 },
                    FamilyOperation { id: 44, flags: 11 },
                    FamilyOperation { id: 45, flags: 11 },
                    FamilyOperation { id: 46, flags: 13 },
                    FamilyOperation { id: 51, flags: 14 },
                    FamilyOperation { id: 52, flags: 14 },
                    FamilyOperation { id: 53, flags: 11 },
                    FamilyOperation { id: 54, flags: 11 },
                    FamilyOperation { id: 55, flags: 11 },
                    FamilyOperation { id: 56, flags: 13 },
                    FamilyOperation { id: 57, flags: 11 },
                    FamilyOperation { id: 58, flags: 11 },
                    FamilyOperation { id: 61, flags: 14 },
                    FamilyOperation { id: 62, flags: 11 },
                    FamilyOperation { id: 65, flags: 14 },
                    FamilyOperation { id: 66, flags: 11 },
                    FamilyOperation { id: 69, flags: 14 },
                    FamilyOperation { id: 70, flags: 11 },
                ],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "config".to_string(),
                    id: 2,
                }],
            },
            Family {
                family_id: 20,
                family_name: "ethtool".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 0,
                operations: vec![
                    FamilyOperation { id: 1, flags: 6 },
                    FamilyOperation { id: 2, flags: 6 },
                    FamilyOperation { id: 3, flags: 18 },
                    FamilyOperation { id: 4, flags: 6 },
                    FamilyOperation { id: 5, flags: 18 },
                    FamilyOperation { id: 6, flags: 6 },
                    FamilyOperation { id: 7, flags: 6 },
                    FamilyOperation { id: 8, flags: 18 },
                    FamilyOperation { id: 9, flags: 22 },
                    FamilyOperation { id: 10, flags: 18 },
                    FamilyOperation { id: 11, flags: 6 },
                    FamilyOperation { id: 12, flags: 18 },
                    FamilyOperation { id: 13, flags: 6 },
                    FamilyOperation { id: 14, flags: 18 },
                    FamilyOperation { id: 15, flags: 6 },
                    FamilyOperation { id: 16, flags: 18 },
                    FamilyOperation { id: 17, flags: 6 },
                    FamilyOperation { id: 18, flags: 18 },
                    FamilyOperation { id: 19, flags: 6 },
                    FamilyOperation { id: 20, flags: 18 },
                    FamilyOperation { id: 21, flags: 6 },
                    FamilyOperation { id: 22, flags: 18 },
                    FamilyOperation { id: 23, flags: 6 },
                    FamilyOperation { id: 24, flags: 18 },
                    FamilyOperation { id: 25, flags: 6 },
                    FamilyOperation { id: 26, flags: 18 },
                    FamilyOperation { id: 27, flags: 18 },
                ],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "monitor".to_string(),
                    id: 3,
                }],
            },
            Family {
                family_id: 21,
                family_name: "NLBL_MGMT".to_string(),
                version: 3,
                header_size: 0,
                max_attr: 12,
                operations: vec![
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 2, flags: 11 },
                    FamilyOperation { id: 3, flags: 12 },
                    FamilyOperation { id: 4, flags: 11 },
                    FamilyOperation { id: 5, flags: 11 },
                    FamilyOperation { id: 6, flags: 10 },
                    FamilyOperation { id: 7, flags: 12 },
                    FamilyOperation { id: 8, flags: 10 },
                ],
                multicast_groups: vec![],
            },
            Family {
                family_id: 22,
                family_name: "NLBL_CIPSOv4".to_string(),
                version: 3,
                header_size: 0,
                max_attr: 12,
                operations: vec![
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 2, flags: 11 },
                    FamilyOperation { id: 3, flags: 10 },
                    FamilyOperation { id: 4, flags: 12 },
                ],
                multicast_groups: vec![],
            },
            Family {
                family_id: 23,
                family_name: "NLBL_CALIPSO".to_string(),
                version: 3,
                header_size: 0,
                max_attr: 2,
                operations: vec![
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 2, flags: 11 },
                    FamilyOperation { id: 3, flags: 10 },
                    FamilyOperation { id: 4, flags: 12 },
                ],
                multicast_groups: vec![],
            },
            Family {
                family_id: 24,
                family_name: "NLBL_UNLBL".to_string(),
                version: 3,
                header_size: 0,
                max_attr: 7,
                operations: vec![
                    FamilyOperation { id: 3, flags: 11 },
                    FamilyOperation { id: 4, flags: 11 },
                    FamilyOperation { id: 5, flags: 12 },
                    FamilyOperation { id: 6, flags: 11 },
                    FamilyOperation { id: 7, flags: 11 },
                    FamilyOperation { id: 8, flags: 12 },
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 2, flags: 10 },
                ],
                multicast_groups: vec![],
            },
            Family {
                family_id: 25,
                family_name: "acpi_event".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 1,
                operations: vec![],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "acpi_mc_group".to_string(),
                    id: 4,
                }],
            },
            Family {
                family_id: 26,
                family_name: "tcp_metrics".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 13,
                operations: vec![
                    FamilyOperation { id: 1, flags: 14 },
                    FamilyOperation { id: 2, flags: 11 },
                ],
                multicast_groups: vec![],
            },
            Family {
                family_id: 27,
                family_name: "mptcp_pm".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 3,
                operations: vec![
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 2, flags: 11 },
                    FamilyOperation { id: 4, flags: 11 },
                    FamilyOperation { id: 3, flags: 14 },
                    FamilyOperation { id: 5, flags: 11 },
                    FamilyOperation { id: 6, flags: 10 },
                ],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "mptcp_pm_cmds".to_string(),
                    id: 5,
                }],
            },
            Family {
                family_id: 28,
                family_name: "NET_DM".to_string(),
                version: 2,
                header_size: 0,
                max_attr: 22,
                operations: vec![
                    FamilyOperation { id: 2, flags: 11 },
                    FamilyOperation { id: 3, flags: 10 },
                    FamilyOperation { id: 4, flags: 10 },
                    FamilyOperation { id: 6, flags: 10 },
                    FamilyOperation { id: 8, flags: 10 },
                ],
                multicast_groups: vec![FamilyMulticastGroup {
                    name: "events".to_string(),
                    id: 1,
                }],
            },
            Family {
                family_id: 29,
                family_name: "SEG6".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 7,
                operations: vec![
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 2, flags: 13 },
                    FamilyOperation { id: 3, flags: 11 },
                    FamilyOperation { id: 4, flags: 11 },
                ],
                multicast_groups: vec![],
            },
            Family {
                family_id: 30,
                family_name: "TASKSTATS".to_string(),
                version: 1,
                header_size: 0,
                max_attr: 4,
                operations: vec![
                    FamilyOperation { id: 1, flags: 11 },
                    FamilyOperation { id: 4, flags: 10 },
                ],
                multicast_groups: vec![],
            },
        ];

        let actual =
            create_message_iterator::<GenericNetlinkResponse<Vec<ControllerAttribute>>>(recv_bytes)
                .map(|read_message_result| {
                    read_message_result
                        .map(|message| message.payload)
                        .map(expect_protocol_message)
                        .map(|genl_message| genl_message.payload)
                })
                .flat_map(|read_message_result| read_message_result.map(|attrs| attrs.try_into()))
                .collect::<Result<Vec<Family>, _>>()?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
