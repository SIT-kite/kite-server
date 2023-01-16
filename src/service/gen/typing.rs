/// 校区定义
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Campus {
    /// 徐汇校区
    Xuhui = 0,
    /// 奉贤校区
    Fengxian = 1,
}
impl Campus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Campus::Xuhui => "Xuhui",
            Campus::Fengxian => "Fengxian",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Xuhui" => Some(Self::Xuhui),
            "Fengxian" => Some(Self::Fengxian),
            _ => None,
        }
    }
}
