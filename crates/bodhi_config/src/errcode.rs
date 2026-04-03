//! 配置模块错误码定义

bodhi_error::define_error_codes! {
  configerr [-200, -101] {
    /// 配置根目录不存在
    ConfigDirNotFound = -101,
    /// 模板目录不存在
    TemplateDirNotFound = -102,
    /// Profile 目录不存在
    ProfileDirNotFound = -103,
    /// Profile 不存在
    ProfileNotFound = -104,
    /// Service 不存在
    ServiceNotFound = -105,
    /// 读取配置文件失败
    FileLoadFailed = -106,
    /// 解析配置文件失败
    ParseFailed = -107,
    /// 合并配置失败
    MergeFailed = -108,
    /// 输出配置失败
    OutputFailed = -109,
    /// 提取类型化配置失败
    ExtractFailed = -110,
    /// 配置路径不合法
    InvalidPath = -111,
    /// 不支持的输出格式
    UnsupportedFormat = -112,
    /// 配置结构不合法
    InvalidStructure = -113,
    /// 未知配置字段
    UnknownField = -114,
    /// 配置值类型不匹配
    TypeMismatch = -115,
    /// Rust 代码生成失败
    CodegenFailed = -116,
  }
}
