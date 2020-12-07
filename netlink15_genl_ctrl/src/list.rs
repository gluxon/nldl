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
    let resp = sock.recv_multipart::<Vec<ControllerAttribute>>()?;

    Ok(resp
        .map(|read_message_result| read_message_result.map(|message| message.payload.payload))
        .collect::<Result<_, _>>()?)
}
