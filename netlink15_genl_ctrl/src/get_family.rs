use super::attr::ControllerAttribute;
use netlink15_genl::{socket::GenlSocket, GenericNetlinkHeader, GenericNetlinkRequest};

pub fn get_family(sock: &GenlSocket, family_name: String) -> nix::Result<Vec<ControllerAttribute>> {
    let genl_request = GenericNetlinkRequest {
        header: GenericNetlinkHeader {
            cmd: libc::CTRL_CMD_GETFAMILY as u8,
            version: 0,
        },
        payload: ControllerAttribute::FamilyName(family_name),
    };

    sock.send(genl_request)?;
    let resp = sock.recv::<Vec<ControllerAttribute>>()?;

    Ok(resp.payload.payload)
}

#[cfg(test)]
mod tests {
    use crate::attr::ControllerAttribute;
    use crate::attr::ControllerAttributeMulticastGroup;
    use netlink15_core::{
        attr::Nested,
        message::{NetlinkPayloadRequest, NetlinkPayloadResponse},
    };
    use netlink15_genl::GenericNetlinkHeader;
    use netlink15_genl::GenericNetlinkRequest;
    use netlink15_genl::GenericNetlinkResponse;

    /// genl ctrl get name acpi_event
    #[test]
    fn request_serialization() {
        let actual = {
            let genl_request = GenericNetlinkRequest {
                header: GenericNetlinkHeader {
                    cmd: libc::CTRL_CMD_GETFAMILY as u8,
                    version: 0,
                },
                payload: ControllerAttribute::FamilyName("acpi_event".to_string()),
            };

            let mut buf = vec![];
            genl_request.serialize(&mut buf);
            buf
        };

        let expected = [
            0x03, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x02, 0x00, 0x61, 0x63, 0x70, 0x69, 0x5f, 0x65,
            0x76, 0x65, 0x6e, 0x74, 0x00,
        ];

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialization() -> anyhow::Result<()> {
        let recv_bytes = [
            0x01, 0x02, 0x00, 0x00, 0x0f, 0x00, 0x02, 0x00, 0x61, 0x63, 0x70, 0x69, 0x5f, 0x65,
            0x76, 0x65, 0x6e, 0x74, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x08, 0x00, 0x05, 0x00, 0x01, 0x00, 0x00, 0x00, 0x24, 0x00, 0x07, 0x00,
            0x20, 0x00, 0x01, 0x00, 0x08, 0x00, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x12, 0x00,
            0x01, 0x00, 0x61, 0x63, 0x70, 0x69, 0x5f, 0x6d, 0x63, 0x5f, 0x67, 0x72, 0x6f, 0x75,
            0x70, 0x00, 0x00, 0x00,
        ];

        let actual: GenericNetlinkResponse<Vec<ControllerAttribute>> =
            NetlinkPayloadResponse::deserialize(&recv_bytes)?;

        let expected = GenericNetlinkResponse {
            header: GenericNetlinkHeader { cmd: 1, version: 2 },
            payload: vec![
                ControllerAttribute::FamilyName("acpi_event".to_string()),
                ControllerAttribute::FamilyId(24),
                ControllerAttribute::Version(1),
                ControllerAttribute::HeaderSize(0),
                ControllerAttribute::MaxAttr(1),
                ControllerAttribute::MulticastGroups(vec![Nested(vec![
                    ControllerAttributeMulticastGroup::Id(3),
                    ControllerAttributeMulticastGroup::Name("acpi_mc_group".to_string()),
                ])]),
            ],
        };

        assert_eq!(actual, expected);

        Ok(())
    }
}
