use crate::attr::ControllerAttribute;
use crate::attr::ControllerAttributeMulticastGroup;
use crate::attr::ControllerAttributeOperation;

#[derive(Debug, PartialEq, Eq)]
pub struct Family {
    pub family_id: u16,
    pub family_name: String,
    pub version: u32,
    pub header_size: u32,
    pub max_attr: u32,
    pub operations: Vec<FamilyOperation>,
    pub multicast_groups: Vec<FamilyMulticastGroup>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FamilyOperation {
    pub id: u32,
    pub flags: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FamilyMulticastGroup {
    pub name: String,
    pub id: u32,
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum MissingFamilyAttributeError {
    #[error("Missing family attribute")]
    MissingFamilyAttribute,
    #[error(transparent)]
    MissingOperationAttribute(#[from] MissingFamilyOperationAttributeError),
    #[error(transparent)]
    MissingMulticastGroupAttribute(#[from] MissingFamilyMulticastGroupError),
}

#[derive(Debug, Clone)]
pub struct MissingFamilyOperationAttributeError;

#[derive(Debug, Clone)]
pub struct MissingFamilyMulticastGroupError;

impl std::error::Error for MissingFamilyOperationAttributeError {}

impl std::error::Error for MissingFamilyMulticastGroupError {}

impl std::fmt::Display for MissingFamilyOperationAttributeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing family operation attribute")
    }
}

impl std::fmt::Display for MissingFamilyMulticastGroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing family multicast group attribute")
    }
}

impl TryFrom<Vec<ControllerAttribute>> for Family {
    type Error = MissingFamilyAttributeError;

    fn try_from(attrs: Vec<ControllerAttribute>) -> Result<Self, Self::Error> {
        let mut family_id = None;
        let mut family_name = None;
        let mut version = None;
        let mut header_size = None;
        let mut max_attr = None;
        let mut operations = None;
        let mut multicast_groups = None;

        for attr in attrs {
            match attr {
                ControllerAttribute::Unspec => {}
                ControllerAttribute::FamilyId(val) => family_id = Some(val),
                ControllerAttribute::FamilyName(val) => family_name = Some(val),
                ControllerAttribute::Version(val) => version = Some(val),
                ControllerAttribute::HeaderSize(val) => header_size = Some(val),
                ControllerAttribute::MaxAttr(val) => max_attr = Some(val),
                ControllerAttribute::Operations(val) => operations = Some(val),
                ControllerAttribute::MulticastGroups(val) => multicast_groups = Some(val),
                ControllerAttribute::Unknown(_) => {}
            }
        }

        Ok(Self {
            family_id: family_id.ok_or(Self::Error::MissingFamilyAttribute)?,
            family_name: family_name.ok_or(Self::Error::MissingFamilyAttribute)?,
            version: version.ok_or(Self::Error::MissingFamilyAttribute)?,
            header_size: header_size.ok_or(Self::Error::MissingFamilyAttribute)?,
            max_attr: max_attr.ok_or(Self::Error::MissingFamilyAttribute)?,
            operations: operations
                .unwrap_or_default()
                .into_iter()
                .map(|nested| nested.0.try_into())
                .collect::<Result<Vec<FamilyOperation>, _>>()?,
            multicast_groups: multicast_groups
                .unwrap_or_default()
                .into_iter()
                .map(|nested| nested.0.try_into())
                .collect::<Result<Vec<FamilyMulticastGroup>, _>>()?,
        })
    }
}

impl TryFrom<Vec<ControllerAttributeOperation>> for FamilyOperation {
    type Error = MissingFamilyOperationAttributeError;

    fn try_from(attrs: Vec<ControllerAttributeOperation>) -> Result<Self, Self::Error> {
        let mut id = None;
        let mut flags = None;

        for attr in attrs {
            match attr {
                ControllerAttributeOperation::Unspec => {}
                ControllerAttributeOperation::Id(val) => id = Some(val),
                ControllerAttributeOperation::Flags(val) => flags = Some(val),
                ControllerAttributeOperation::Unknown(_) => {}
            }
        }

        Ok(Self {
            id: id.ok_or(Self::Error {})?,
            flags: flags.ok_or(Self::Error {})?,
        })
    }
}

impl TryFrom<Vec<ControllerAttributeMulticastGroup>> for FamilyMulticastGroup {
    type Error = MissingFamilyMulticastGroupError;

    fn try_from(attrs: Vec<ControllerAttributeMulticastGroup>) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut id = None;

        for attr in attrs {
            match attr {
                ControllerAttributeMulticastGroup::Unspec => {}
                ControllerAttributeMulticastGroup::Name(val) => name = Some(val),
                ControllerAttributeMulticastGroup::Id(val) => id = Some(val),
                ControllerAttributeMulticastGroup::Unknown(_) => {}
            }
        }

        Ok(Self {
            name: name.ok_or(Self::Error {})?,
            id: id.ok_or(Self::Error {})?,
        })
    }
}
