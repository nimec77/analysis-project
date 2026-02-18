use super::combinators::primitives;
use super::combinators::*;
use super::domain::*;

/// Все виды логов
#[derive(Debug, Clone, PartialEq)]
pub enum LogKind {
    System(SystemLogKind),
    App(AppLogKind),
}
/// Все виды [системных](LogKind) логов
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogKind {
    Error(SystemLogErrorKind),
    Trace(SystemLogTraceKind),
}
/// Trace [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogTraceKind {
    SendRequest(String),
    GetResponse(String),
}
/// Error [системы](SystemLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum SystemLogErrorKind {
    NetworkError(String),
    AccessDenied(String),
}
/// Все виды [логов приложения](LogKind) логов
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogKind {
    Error(AppLogErrorKind),
    Trace(AppLogTraceKind),
    Journal(AppLogJournalKind),
}
/// Error [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogErrorKind {
    LackOf(String),
    SystemError(String),
}
/// Trace [приложения](AppLogKind)
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogTraceKind {
    Connect(Box<AuthData>),
    SendRequest(String),
    Check(Announcements),
    GetResponse(String),
}
/// Журнал [приложения](AppLogKind), самые высокоуровневые события
#[derive(Debug, Clone, PartialEq)]
pub enum AppLogJournalKind {
    CreateUser {
        user_id: UserId,
        authorized_capital: std::num::NonZeroU32,
    },
    DeleteUser {
        user_id: UserId,
    },
    RegisterAsset {
        asset_id: AssetId,
        user_id: UserId,
        liquidity: std::num::NonZeroU32,
    },
    UnregisterAsset {
        asset_id: AssetId,
        user_id: UserId,
    },
    DepositCash(UserCash),
    WithdrawCash(UserCash),
    BuyAsset(UserBacket),
    SellAsset(UserBacket),
}
impl Parsable for SystemLogErrorKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogErrorKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogErrorKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Error"),
            alt2(
                map(
                    preceded(
                        strip_whitespace(tag("NetworkError")),
                        strip_whitespace(unquote()),
                    ),
                    |error| SystemLogErrorKind::NetworkError(error),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("AccessDenied")),
                        strip_whitespace(unquote()),
                    ),
                    |error| SystemLogErrorKind::AccessDenied(error),
                ),
            ),
        )
    }
}
impl Parsable for SystemLogTraceKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> SystemLogTraceKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Trace"),
            alt2(
                map(
                    preceded(
                        strip_whitespace(tag("SendRequest")),
                        strip_whitespace(unquote()),
                    ),
                    |request| SystemLogTraceKind::SendRequest(request),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("GetResponse")),
                        strip_whitespace(unquote()),
                    ),
                    |response| SystemLogTraceKind::GetResponse(response),
                ),
            ),
        )
    }
}
impl Parsable for SystemLogKind {
    type Parser = StripWhitespace<
        Preceded<
            Tag,
            Alt<(
                Map<
                    <SystemLogTraceKind as Parsable>::Parser,
                    fn(SystemLogTraceKind) -> SystemLogKind,
                >,
                Map<
                    <SystemLogErrorKind as Parsable>::Parser,
                    fn(SystemLogErrorKind) -> SystemLogKind,
                >,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(preceded(
            tag("System::"),
            alt2(
                map(SystemLogTraceKind::parser(), |trace| {
                    SystemLogKind::Trace(trace)
                }),
                map(SystemLogErrorKind::parser(), |error| {
                    SystemLogKind::Error(error)
                }),
            ),
        ))
    }
}
impl Parsable for AppLogErrorKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogErrorKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogErrorKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Error"),
            alt2(
                map(
                    preceded(strip_whitespace(tag("LackOf")), strip_whitespace(unquote())),
                    |error| AppLogErrorKind::LackOf(error),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SystemError")),
                        strip_whitespace(unquote()),
                    ),
                    |error| AppLogErrorKind::SystemError(error),
                ),
            ),
        )
    }
}
impl Parsable for AppLogTraceKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<<AuthData as Parsable>::Parser>>,
                fn(AuthData) -> AppLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogTraceKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    StripWhitespace<<Announcements as Parsable>::Parser>,
                >,
                fn(Announcements) -> AppLogTraceKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, StripWhitespace<Unquote>>,
                fn(String) -> AppLogTraceKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Trace"),
            alt4(
                map(
                    preceded(
                        strip_whitespace(tag("Connect")),
                        strip_whitespace(AuthData::parser()),
                    ),
                    |authdata| AppLogTraceKind::Connect(Box::new(authdata)),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("SendRequest")),
                        strip_whitespace(unquote()),
                    ),
                    |trace| AppLogTraceKind::SendRequest(trace),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("Check")),
                        strip_whitespace(Announcements::parser()),
                    ),
                    |announcements| AppLogTraceKind::Check(announcements),
                ),
                map(
                    preceded(
                        strip_whitespace(tag("GetResponse")),
                        strip_whitespace(unquote()),
                    ),
                    |trace| AppLogTraceKind::GetResponse(trace),
                ),
            ),
        )
    }
}
impl Parsable for AppLogJournalKind {
    type Parser = Preceded<
        Tag,
        Alt<(
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(KeyValue<<UserId as Parsable>::Parser>, KeyValue<primitives::U32>)>,
                        Tag,
                    >,
                >,
                fn((UserId, std::num::NonZeroU32)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, Delimited<Tag, KeyValue<<UserId as Parsable>::Parser>, Tag>>,
                fn(UserId) -> AppLogJournalKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<
                        Tag,
                        Permutation<(
                            KeyValue<<AssetId as Parsable>::Parser>,
                            KeyValue<<UserId as Parsable>::Parser>,
                            KeyValue<primitives::U32>,
                        )>,
                        Tag,
                    >,
                >,
                fn((AssetId, UserId, std::num::NonZeroU32)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<
                    StripWhitespace<Tag>,
                    Delimited<Tag, Permutation<(KeyValue<<AssetId as Parsable>::Parser>, KeyValue<<UserId as Parsable>::Parser>)>, Tag>,
                >,
                fn((AssetId, UserId)) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserCash as Parsable>::Parser>,
                fn(UserCash) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserBacket as Parsable>::Parser>,
                fn(UserBacket) -> AppLogJournalKind,
            >,
            Map<
                Preceded<StripWhitespace<Tag>, <UserBacket as Parsable>::Parser>,
                fn(UserBacket) -> AppLogJournalKind,
            >,
        )>,
    >;
    fn parser() -> Self::Parser {
        preceded(
            tag("Journal"),
            alt8(
                map(
                    preceded(
                        strip_whitespace(tag("CreateUser")),
                        delimited(
                            tag("{"),
                            permutation2(
                                key_value("user_id", UserId::parser()),
                                key_value("authorized_capital", primitives::U32),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(user_id, authorized_capital)| AppLogJournalKind::CreateUser {
                        user_id,
                        authorized_capital,
                    },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("DeleteUser")),
                        delimited(tag("{"), key_value("user_id", UserId::parser()), tag("}")),
                    ),
                    |user_id| AppLogJournalKind::DeleteUser { user_id },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("RegisterAsset")),
                        delimited(
                            tag("{"),
                            permutation3(
                                key_value("asset_id", AssetId::parser()),
                                key_value("user_id", UserId::parser()),
                                key_value("liquidity", primitives::U32),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(asset_id, user_id, liquidity)| AppLogJournalKind::RegisterAsset {
                        asset_id,
                        user_id,
                        liquidity,
                    },
                ),
                map(
                    preceded(
                        strip_whitespace(tag("UnregisterAsset")),
                        delimited(
                            tag("{"),
                            permutation2(
                                key_value("asset_id", AssetId::parser()),
                                key_value("user_id", UserId::parser()),
                            ),
                            tag("}"),
                        ),
                    ),
                    |(asset_id, user_id)| AppLogJournalKind::UnregisterAsset { asset_id, user_id },
                ),
                map(
                    preceded(strip_whitespace(tag("DepositCash")), UserCash::parser()),
                    |user_cash| AppLogJournalKind::DepositCash(user_cash),
                ),
                map(
                    preceded(strip_whitespace(tag("WithdrawCash")), UserCash::parser()),
                    |user_cash| AppLogJournalKind::WithdrawCash(user_cash),
                ),
                map(
                    preceded(strip_whitespace(tag("BuyAsset")), UserBacket::parser()),
                    |user_backet| AppLogJournalKind::BuyAsset(user_backet),
                ),
                map(
                    preceded(strip_whitespace(tag("SellAsset")), UserBacket::parser()),
                    |user_backet| AppLogJournalKind::SellAsset(user_backet),
                ),
            ),
        )
    }
}
impl Parsable for AppLogKind {
    type Parser = StripWhitespace<
        Preceded<
            Tag,
            Alt<(
                Map<<AppLogErrorKind as Parsable>::Parser, fn(AppLogErrorKind) -> AppLogKind>,
                Map<<AppLogTraceKind as Parsable>::Parser, fn(AppLogTraceKind) -> AppLogKind>,
                Map<<AppLogJournalKind as Parsable>::Parser, fn(AppLogJournalKind) -> AppLogKind>,
            )>,
        >,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(preceded(
            tag("App::"),
            alt3(
                map(AppLogErrorKind::parser(), |error| AppLogKind::Error(error)),
                map(AppLogTraceKind::parser(), |trace| AppLogKind::Trace(trace)),
                map(AppLogJournalKind::parser(), |journal| {
                    AppLogKind::Journal(journal)
                }),
            ),
        ))
    }
}
impl Parsable for LogKind {
    type Parser = StripWhitespace<
        Alt<(
            Map<<SystemLogKind as Parsable>::Parser, fn(SystemLogKind) -> LogKind>,
            Map<<AppLogKind as Parsable>::Parser, fn(AppLogKind) -> LogKind>,
        )>,
    >;
    fn parser() -> Self::Parser {
        strip_whitespace(alt2(
            map(SystemLogKind::parser(), |system| LogKind::System(system)),
            map(AppLogKind::parser(), |app| LogKind::App(app)),
        ))
    }
}
/// Строка логов, [лог](AppLogKind) с `request_id`
#[derive(Debug, Clone, PartialEq)]
pub struct LogLine {
    pub kind: LogKind,
    pub request_id: std::num::NonZeroU32,
}
impl Parsable for LogLine {
    type Parser = Map<
        Tuple<(
            <LogKind as Parsable>::Parser,
            StripWhitespace<Preceded<Tag, primitives::U32>>,
        )>,
        fn((LogKind, std::num::NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        map(
            tuple2(
                LogKind::parser(),
                strip_whitespace(preceded(tag("requestid="), primitives::U32)),
            ),
            |(kind, request_id)| LogLine { kind, request_id },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;

    fn nz(n: u32) -> NonZeroU32 {
        NonZeroU32::new(n).unwrap()
    }

    #[test]
    fn test_log_kind() {
        assert_eq!(
            preceded(
                strip_whitespace(tag("NetworkError")),
                strip_whitespace(unquote())
            )
            .parse(r#"NetworkError "url unknown""#),
            Ok(("", "url unknown".into()))
        );

        assert_eq!(
            LogKind::parser().parse(r#"System::Error NetworkError "url unknown""#),
            Ok((
                "",
                LogKind::System(SystemLogKind::Error(SystemLogErrorKind::NetworkError(
                    "url unknown".into()
                )))
            ))
        );
        assert_eq!(LogKind::parser().parse(r#"App::Trace Connect 30c305825b900077ae7f8259c1c328aa3e124a07f3bfbbf216dfc6e308beea6e474b9a7ea6c24d003a6ae4fcf04a9e6ef7c7f17cdaa0296f66a88036badcf01f053da806fad356546349deceff24621b895440d05a715b221af8e9e068073d6dec04f148175717d3c2d1b6af84e2375718ab4a1eba7e037c1c1d43b4cf422d6f2aa9194266f0a7544eaeff8167f0e993d0ea6a8ddb98bfeb8805635d5ea9f6592fd5297e6f83b6834190f99449722cd0de87a4c122f08bbe836fd3092e5f0d37a3057e90f3dd41048da66cad3e8fd3ef72a9d86ecd9009c2db996af29dc62af5ef5eb04d0e16ce8fcecba92a4a9888f52d5d575e7dbc302ed97dbf69df15bb4f5c5601d38fbe3bd89d88768a6aed11ce2f95a6ad30bb72e787bfb734701cea1f38168be44ea19d3e98dd3c953fdb9951ac9c6e221bb0f980d8f0952ac8127da5bda7077dd25ffc8e1515c529f29516dacec6be9c084e6c91698267b2aed9038eca5ebafad479c5fb17652e25bb5b85586fae645bd7c3253d9916c0af65a20253412d5484ac15d288c6ca8823469090ded5ce0975dada63653797129f0e926af6247b457b067db683e37d848e0acf30e5602b78f1848e8da4b640ed08b75f3519a40ec96b2be964234beab37759504376c6e5ebfacdc57e4c7a22cf1e879d7bde29a2dca5fe20420215b59d102fd016606c533e8e36f7da114910664bade9b295d9043a01bc0dc4d8abbc16b1cec7789d89e699ad99dae597c7f10d6f047efc011d67444695cb8e6e8b3dba17ccc693729d01312d0f12a3fc76e12c2e4984af5cb3049b9d8a13124a1f770e96bae1fb153ba4c91bea4fae6f03010275d5a9b14012bdd678e037934dc6762005de54b32a7684e03060d5cc80378e9bef05b8f0692202944401bd06e4553e4490a0e57c5a72fc8abb1f714e22ea950fb2f1de284d6ff3da435954de355c677f60db4252a510919cbe7dadfed0441cf125fd8894753af8114f2ddacb75c3daa460920fc47d285e59fe9110e4151fcef03fa246cd2dd9a4d573e1dbbda1c6968cf4f546289b95ce1bf0a55eea6531382826d4002bc46bf441ce16056d42b5a2079e299e3191c23a7604cde03de6081e06f93cfe632c9a6088cd328662d47a4954934832df5b5f3765dbe136114c73c55cb7ce639e5d40d1d1d8f540d3c8e1bc7423f032c0da5264353468f009c973eec0448e41f9289e8d9dadc68da77d3c3ab3a6477d44024f21fba0bd4477d81c6027657527aa0413b45f417cb7b3beea835a1d5d795414d38156324cb5c1303e9924dbe40cd497c4c23c221cb912058c939bea8b79b3fea360fecaa83375a9a84e338d9e863e8021ad2df4430b8dea0c1714e1bdc478f559705549ad738453ab65c0ffcc8cf0e3bafaf4afad75ecc4dfad0de0cfe27d50d656456ea6c361b76508357714079424"#), Ok(("", LogKind::App(AppLogKind::Trace(AppLogTraceKind::Connect(Box::new(AuthData([0x30,0xc3,0x05,0x82,0x5b,0x90,0x00,0x77,0xae,0x7f,0x82,0x59,0xc1,0xc3,0x28,0xaa,0x3e,0x12,0x4a,0x07,0xf3,0xbf,0xbb,0xf2,0x16,0xdf,0xc6,0xe3,0x08,0xbe,0xea,0x6e,0x47,0x4b,0x9a,0x7e,0xa6,0xc2,0x4d,0x00,0x3a,0x6a,0xe4,0xfc,0xf0,0x4a,0x9e,0x6e,0xf7,0xc7,0xf1,0x7c,0xda,0xa0,0x29,0x6f,0x66,0xa8,0x80,0x36,0xba,0xdc,0xf0,0x1f,0x05,0x3d,0xa8,0x06,0xfa,0xd3,0x56,0x54,0x63,0x49,0xde,0xce,0xff,0x24,0x62,0x1b,0x89,0x54,0x40,0xd0,0x5a,0x71,0x5b,0x22,0x1a,0xf8,0xe9,0xe0,0x68,0x07,0x3d,0x6d,0xec,0x04,0xf1,0x48,0x17,0x57,0x17,0xd3,0xc2,0xd1,0xb6,0xaf,0x84,0xe2,0x37,0x57,0x18,0xab,0x4a,0x1e,0xba,0x7e,0x03,0x7c,0x1c,0x1d,0x43,0xb4,0xcf,0x42,0x2d,0x6f,0x2a,0xa9,0x19,0x42,0x66,0xf0,0xa7,0x54,0x4e,0xae,0xff,0x81,0x67,0xf0,0xe9,0x93,0xd0,0xea,0x6a,0x8d,0xdb,0x98,0xbf,0xeb,0x88,0x05,0x63,0x5d,0x5e,0xa9,0xf6,0x59,0x2f,0xd5,0x29,0x7e,0x6f,0x83,0xb6,0x83,0x41,0x90,0xf9,0x94,0x49,0x72,0x2c,0xd0,0xde,0x87,0xa4,0xc1,0x22,0xf0,0x8b,0xbe,0x83,0x6f,0xd3,0x09,0x2e,0x5f,0x0d,0x37,0xa3,0x05,0x7e,0x90,0xf3,0xdd,0x41,0x04,0x8d,0xa6,0x6c,0xad,0x3e,0x8f,0xd3,0xef,0x72,0xa9,0xd8,0x6e,0xcd,0x90,0x09,0xc2,0xdb,0x99,0x6a,0xf2,0x9d,0xc6,0x2a,0xf5,0xef,0x5e,0xb0,0x4d,0x0e,0x16,0xce,0x8f,0xce,0xcb,0xa9,0x2a,0x4a,0x98,0x88,0xf5,0x2d,0x5d,0x57,0x5e,0x7d,0xbc,0x30,0x2e,0xd9,0x7d,0xbf,0x69,0xdf,0x15,0xbb,0x4f,0x5c,0x56,0x01,0xd3,0x8f,0xbe,0x3b,0xd8,0x9d,0x88,0x76,0x8a,0x6a,0xed,0x11,0xce,0x2f,0x95,0xa6,0xad,0x30,0xbb,0x72,0xe7,0x87,0xbf,0xb7,0x34,0x70,0x1c,0xea,0x1f,0x38,0x16,0x8b,0xe4,0x4e,0xa1,0x9d,0x3e,0x98,0xdd,0x3c,0x95,0x3f,0xdb,0x99,0x51,0xac,0x9c,0x6e,0x22,0x1b,0xb0,0xf9,0x80,0xd8,0xf0,0x95,0x2a,0xc8,0x12,0x7d,0xa5,0xbd,0xa7,0x07,0x7d,0xd2,0x5f,0xfc,0x8e,0x15,0x15,0xc5,0x29,0xf2,0x95,0x16,0xda,0xce,0xc6,0xbe,0x9c,0x08,0x4e,0x6c,0x91,0x69,0x82,0x67,0xb2,0xae,0xd9,0x03,0x8e,0xca,0x5e,0xba,0xfa,0xd4,0x79,0xc5,0xfb,0x17,0x65,0x2e,0x25,0xbb,0x5b,0x85,0x58,0x6f,0xae,0x64,0x5b,0xd7,0xc3,0x25,0x3d,0x99,0x16,0xc0,0xaf,0x65,0xa2,0x02,0x53,0x41,0x2d,0x54,0x84,0xac,0x15,0xd2,0x88,0xc6,0xca,0x88,0x23,0x46,0x90,0x90,0xde,0xd5,0xce,0x09,0x75,0xda,0xda,0x63,0x65,0x37,0x97,0x12,0x9f,0x0e,0x92,0x6a,0xf6,0x24,0x7b,0x45,0x7b,0x06,0x7d,0xb6,0x83,0xe3,0x7d,0x84,0x8e,0x0a,0xcf,0x30,0xe5,0x60,0x2b,0x78,0xf1,0x84,0x8e,0x8d,0xa4,0xb6,0x40,0xed,0x08,0xb7,0x5f,0x35,0x19,0xa4,0x0e,0xc9,0x6b,0x2b,0xe9,0x64,0x23,0x4b,0xea,0xb3,0x77,0x59,0x50,0x43,0x76,0xc6,0xe5,0xeb,0xfa,0xcd,0xc5,0x7e,0x4c,0x7a,0x22,0xcf,0x1e,0x87,0x9d,0x7b,0xde,0x29,0xa2,0xdc,0xa5,0xfe,0x20,0x42,0x02,0x15,0xb5,0x9d,0x10,0x2f,0xd0,0x16,0x60,0x6c,0x53,0x3e,0x8e,0x36,0xf7,0xda,0x11,0x49,0x10,0x66,0x4b,0xad,0xe9,0xb2,0x95,0xd9,0x04,0x3a,0x01,0xbc,0x0d,0xc4,0xd8,0xab,0xbc,0x16,0xb1,0xce,0xc7,0x78,0x9d,0x89,0xe6,0x99,0xad,0x99,0xda,0xe5,0x97,0xc7,0xf1,0x0d,0x6f,0x04,0x7e,0xfc,0x01,0x1d,0x67,0x44,0x46,0x95,0xcb,0x8e,0x6e,0x8b,0x3d,0xba,0x17,0xcc,0xc6,0x93,0x72,0x9d,0x01,0x31,0x2d,0x0f,0x12,0xa3,0xfc,0x76,0xe1,0x2c,0x2e,0x49,0x84,0xaf,0x5c,0xb3,0x04,0x9b,0x9d,0x8a,0x13,0x12,0x4a,0x1f,0x77,0x0e,0x96,0xba,0xe1,0xfb,0x15,0x3b,0xa4,0xc9,0x1b,0xea,0x4f,0xae,0x6f,0x03,0x01,0x02,0x75,0xd5,0xa9,0xb1,0x40,0x12,0xbd,0xd6,0x78,0xe0,0x37,0x93,0x4d,0xc6,0x76,0x20,0x05,0xde,0x54,0xb3,0x2a,0x76,0x84,0xe0,0x30,0x60,0xd5,0xcc,0x80,0x37,0x8e,0x9b,0xef,0x05,0xb8,0xf0,0x69,0x22,0x02,0x94,0x44,0x01,0xbd,0x06,0xe4,0x55,0x3e,0x44,0x90,0xa0,0xe5,0x7c,0x5a,0x72,0xfc,0x8a,0xbb,0x1f,0x71,0x4e,0x22,0xea,0x95,0x0f,0xb2,0xf1,0xde,0x28,0x4d,0x6f,0xf3,0xda,0x43,0x59,0x54,0xde,0x35,0x5c,0x67,0x7f,0x60,0xdb,0x42,0x52,0xa5,0x10,0x91,0x9c,0xbe,0x7d,0xad,0xfe,0xd0,0x44,0x1c,0xf1,0x25,0xfd,0x88,0x94,0x75,0x3a,0xf8,0x11,0x4f,0x2d,0xda,0xcb,0x75,0xc3,0xda,0xa4,0x60,0x92,0x0f,0xc4,0x7d,0x28,0x5e,0x59,0xfe,0x91,0x10,0xe4,0x15,0x1f,0xce,0xf0,0x3f,0xa2,0x46,0xcd,0x2d,0xd9,0xa4,0xd5,0x73,0xe1,0xdb,0xbd,0xa1,0xc6,0x96,0x8c,0xf4,0xf5,0x46,0x28,0x9b,0x95,0xce,0x1b,0xf0,0xa5,0x5e,0xea,0x65,0x31,0x38,0x28,0x26,0xd4,0x00,0x2b,0xc4,0x6b,0xf4,0x41,0xce,0x16,0x05,0x6d,0x42,0xb5,0xa2,0x07,0x9e,0x29,0x9e,0x31,0x91,0xc2,0x3a,0x76,0x04,0xcd,0xe0,0x3d,0xe6,0x08,0x1e,0x06,0xf9,0x3c,0xfe,0x63,0x2c,0x9a,0x60,0x88,0xcd,0x32,0x86,0x62,0xd4,0x7a,0x49,0x54,0x93,0x48,0x32,0xdf,0x5b,0x5f,0x37,0x65,0xdb,0xe1,0x36,0x11,0x4c,0x73,0xc5,0x5c,0xb7,0xce,0x63,0x9e,0x5d,0x40,0xd1,0xd1,0xd8,0xf5,0x40,0xd3,0xc8,0xe1,0xbc,0x74,0x23,0xf0,0x32,0xc0,0xda,0x52,0x64,0x35,0x34,0x68,0xf0,0x09,0xc9,0x73,0xee,0xc0,0x44,0x8e,0x41,0xf9,0x28,0x9e,0x8d,0x9d,0xad,0xc6,0x8d,0xa7,0x7d,0x3c,0x3a,0xb3,0xa6,0x47,0x7d,0x44,0x02,0x4f,0x21,0xfb,0xa0,0xbd,0x44,0x77,0xd8,0x1c,0x60,0x27,0x65,0x75,0x27,0xaa,0x04,0x13,0xb4,0x5f,0x41,0x7c,0xb7,0xb3,0xbe,0xea,0x83,0x5a,0x1d,0x5d,0x79,0x54,0x14,0xd3,0x81,0x56,0x32,0x4c,0xb5,0xc1,0x30,0x3e,0x99,0x24,0xdb,0xe4,0x0c,0xd4,0x97,0xc4,0xc2,0x3c,0x22,0x1c,0xb9,0x12,0x05,0x8c,0x93,0x9b,0xea,0x8b,0x79,0xb3,0xfe,0xa3,0x60,0xfe,0xca,0xa8,0x33,0x75,0xa9,0xa8,0x4e,0x33,0x8d,0x9e,0x86,0x3e,0x80,0x21,0xad,0x2d,0xf4,0x43,0x0b,0x8d,0xea,0x0c,0x17,0x14,0xe1,0xbd,0xc4,0x78,0xf5,0x59,0x70,0x55,0x49,0xad,0x73,0x84,0x53,0xab,0x65,0xc0,0xff,0xcc,0x8c,0xf0,0xe3,0xba,0xfa,0xf4,0xaf,0xad,0x75,0xec,0xc4,0xdf,0xad,0x0d,0xe0,0xcf,0xe2,0x7d,0x50,0xd6,0x56,0x45,0x6e,0xa6,0xc3,0x61,0xb7,0x65,0x08,0x35,0x77,0x14,0x07,0x94,0x24]))))))));
        assert_eq!(
            LogKind::parser().parse(
                r#"App::Journal CreateUser {"user_id": "Steeve", "authorized_capital": 10000,}"#
            ),
            Ok((
                "",
                LogKind::App(AppLogKind::Journal(AppLogJournalKind::CreateUser {
                    user_id: UserId("Steeve".into()),
                    authorized_capital: nz(10_000)
                }))
            ))
        );
        assert_eq!(
            LogKind::parser().parse(r#"App::Journal DeleteUser {"user_id": "Steeve",}"#),
            Ok((
                "",
                LogKind::App(AppLogKind::Journal(AppLogJournalKind::DeleteUser {
                    user_id: UserId("Steeve".into())
                }))
            ))
        );
        assert_eq!(LogKind::parser().parse(r#"App::Journal RegisterAsset {"asset_id": "bayc", "liquidity": 100000000, "user_id": "Steeve",}"#), Ok(("", LogKind::App(AppLogKind::Journal(AppLogJournalKind::RegisterAsset{asset_id: AssetId("bayc".into()), user_id: UserId("Steeve".into()), liquidity: nz(100_000_000)})))));
        assert_eq!(
            LogKind::parser()
                .parse(r#"App::Journal DepositCash UserCash{"user_id": "Steeve", "count": 10,}"#),
            Ok((
                "",
                LogKind::App(AppLogKind::Journal(AppLogJournalKind::DepositCash(
                    UserCash {
                        user_id: UserId("Steeve".into()),
                        count: nz(10)
                    }
                )))
            ))
        );
        assert_eq!(LogKind::parser().parse(r#"App::Journal BuyAsset UserBacket{"user_id": "Steeve", "backet": Backet{"asset_id":"bayc","count":1,},}"#), Ok(("", LogKind::App(AppLogKind::Journal(AppLogJournalKind::BuyAsset(UserBacket{user_id: UserId("Steeve".into()), backet: Backet{asset_id: AssetId("bayc".into()),count:nz(1)}}))))));
    }

    #[test]
    fn test_withdraw_cash() {
        assert_eq!(
            LogKind::parser()
                .parse(r#"App::Journal WithdrawCash UserCash{"user_id":"Alice","count":500,}"#),
            Ok((
                "",
                LogKind::App(AppLogKind::Journal(AppLogJournalKind::WithdrawCash(
                    UserCash {
                        user_id: UserId("Alice".into()),
                        count: nz(500)
                    }
                )))
            ))
        );
    }
}
