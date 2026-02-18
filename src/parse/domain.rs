use std::fmt;

use super::combinators::primitives;
use super::combinators::*;

const AUTHDATA_SIZE: usize = 1024;

/// Данные для авторизации
#[derive(Debug, Clone, PartialEq)]
pub struct AuthData(pub(crate) [u8; AUTHDATA_SIZE]);
impl Parsable for AuthData {
    type Parser = Map<Take<primitives::Byte>, fn(Vec<u8>) -> Self>;
    fn parser() -> Self::Parser {
        take(AUTHDATA_SIZE, primitives::Byte)
            .map(|authdata| AuthData(authdata.try_into().unwrap_or([0; AUTHDATA_SIZE])))
    }
}
impl fmt::Display for AuthData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0[..16] {
            write!(f, "{byte:02x}")?;
        }
        write!(f, "...({} bytes)", AUTHDATA_SIZE)
    }
}

/// Newtype wrapper around String for type-safe user identification.
#[derive(Debug, Clone, PartialEq)]
pub struct UserId(pub String);
impl Parsable for UserId {
    type Parser = Map<Unquote, fn(String) -> Self>;
    fn parser() -> Self::Parser {
        unquote().map(UserId)
    }
}
impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Newtype wrapper around String for type-safe asset identification.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetId(pub String);
impl Parsable for AssetId {
    type Parser = Map<Unquote, fn(String) -> Self>;
    fn parser() -> Self::Parser {
        unquote().map(AssetId)
    }
}
impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Пара 'сокращённое название предмета' - 'его описание'
#[derive(Debug, Clone, PartialEq)]
pub struct AssetDsc {
    // `dsc` aka `description`
    pub id: AssetId,
    pub dsc: String,
}
impl Parsable for AssetDsc {
    type Parser = Map<
        Delimited<
            Tuple<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<<AssetId as Parsable>::Parser>, KeyValue<Unquote>)>,
            StripWhitespace<Tag>,
        >,
        fn((AssetId, String)) -> Self,
    >;
    fn parser() -> Self::Parser {
        // комбинаторы парсеров - это круто
        delimited(
            tuple2(tag("AssetDsc").strip_ws(), tag("{").strip_ws()),
            permutation2(key_value("id", AssetId::parser()), key_value("dsc", unquote())),
            tag("}").strip_ws(),
        )
        .map(|(id, dsc)| AssetDsc { id, dsc })
    }
}
impl fmt::Display for AssetDsc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.id, self.dsc)
    }
}
/// Сведение о предмете в некотором количестве
#[derive(Debug, Clone, PartialEq)]
pub struct Backet {
    pub asset_id: AssetId,
    pub count: std::num::NonZeroU32,
}
impl Parsable for Backet {
    type Parser = Map<
        Delimited<
            Tuple<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<<AssetId as Parsable>::Parser>, KeyValue<primitives::U32>)>,
            StripWhitespace<Tag>,
        >,
        fn((AssetId, std::num::NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        delimited(
            tuple2(tag("Backet").strip_ws(), tag("{").strip_ws()),
            permutation2(
                key_value("asset_id", AssetId::parser()),
                key_value("count", primitives::U32),
            ),
            tag("}").strip_ws(),
        )
        .map(|(asset_id, count)| Backet { asset_id, count })
    }
}
impl fmt::Display for Backet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.count, self.asset_id)
    }
}
/// Фиатные деньги конкретного пользователя
#[derive(Debug, Clone, PartialEq)]
pub struct UserCash {
    pub user_id: UserId,
    pub count: std::num::NonZeroU32,
}
impl Parsable for UserCash {
    type Parser = Map<
        Delimited<
            Tuple<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<<UserId as Parsable>::Parser>, KeyValue<primitives::U32>)>,
            StripWhitespace<Tag>,
        >,
        fn((UserId, std::num::NonZeroU32)) -> Self,
    >;
    fn parser() -> Self::Parser {
        delimited(
            tuple2(tag("UserCash").strip_ws(), tag("{").strip_ws()),
            permutation2(
                key_value("user_id", UserId::parser()),
                key_value("count", primitives::U32),
            ),
            tag("}").strip_ws(),
        )
        .map(|(user_id, count)| UserCash { user_id, count })
    }
}
impl fmt::Display for UserCash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user={}, amount={}", self.user_id, self.count)
    }
}
/// [Backet] конкретного пользователя
#[derive(Debug, Clone, PartialEq)]
pub struct UserBacket {
    pub user_id: UserId,
    pub backet: Backet,
}
impl Parsable for UserBacket {
    type Parser = Map<
        Delimited<
            Tuple<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(KeyValue<<UserId as Parsable>::Parser>, KeyValue<<Backet as Parsable>::Parser>)>,
            StripWhitespace<Tag>,
        >,
        fn((UserId, Backet)) -> Self,
    >;
    fn parser() -> Self::Parser {
        delimited(
            tuple2(tag("UserBacket").strip_ws(), tag("{").strip_ws()),
            permutation2(
                key_value("user_id", UserId::parser()),
                key_value("backet", Backet::parser()),
            ),
            tag("}").strip_ws(),
        )
        .map(|(user_id, backet)| UserBacket { user_id, backet })
    }
}
impl fmt::Display for UserBacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user={}, backet={}", self.user_id, self.backet)
    }
}
/// [Бакеты](Backet) конкретного пользователя
#[derive(Debug, Clone, PartialEq)]
pub struct UserBackets {
    pub user_id: UserId,
    pub backets: Vec<Backet>,
}
impl Parsable for UserBackets {
    type Parser = Map<
        Delimited<
            Tuple<(StripWhitespace<Tag>, StripWhitespace<Tag>)>,
            Permutation<(
                KeyValue<<UserId as Parsable>::Parser>,
                KeyValue<List<<Backet as Parsable>::Parser>>,
            )>,
            StripWhitespace<Tag>,
        >,
        fn((UserId, Vec<Backet>)) -> Self,
    >;
    fn parser() -> Self::Parser {
        delimited(
            tuple2(tag("UserBackets").strip_ws(), tag("{").strip_ws()),
            permutation2(
                key_value("user_id", UserId::parser()),
                key_value("backets", list(Backet::parser())),
            ),
            tag("}").strip_ws(),
        )
        .map(|(user_id, backets)| UserBackets { user_id, backets })
    }
}
impl fmt::Display for UserBackets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user={}, backets=[", self.user_id)?;
        for (i, backet) in self.backets.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{backet}")?;
        }
        write!(f, "]")
    }
}
/// Список опубликованных бакетов
#[derive(Debug, Clone, PartialEq)]
pub struct Announcements(Vec<UserBackets>);
impl Parsable for Announcements {
    type Parser = Map<List<<UserBackets as Parsable>::Parser>, fn(Vec<UserBackets>) -> Self>;
    fn parser() -> Self::Parser {
        list(UserBackets::parser()).map(Announcements)
    }
}
impl fmt::Display for Announcements {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, ub) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{ub}")?;
        }
        write!(f, "]")
    }
}

/// Generic wrapper for parsing any [Parsable] type.
pub fn just_parse<T: Parsable>(input: &str) -> Result<(&str, T), ParseError> {
    T::parser().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;

    fn nz(n: u32) -> NonZeroU32 {
        NonZeroU32::new(n).unwrap()
    }

    #[test]
    fn test_authdata() {
        let s = "30c305825b900077ae7f8259c1c328aa3e124a07f3bfbbf216dfc6e308beea6e474b9a7ea6c24d003a6ae4fcf04a9e6ef7c7f17cdaa0296f66a88036badcf01f053da806fad356546349deceff24621b895440d05a715b221af8e9e068073d6dec04f148175717d3c2d1b6af84e2375718ab4a1eba7e037c1c1d43b4cf422d6f2aa9194266f0a7544eaeff8167f0e993d0ea6a8ddb98bfeb8805635d5ea9f6592fd5297e6f83b6834190f99449722cd0de87a4c122f08bbe836fd3092e5f0d37a3057e90f3dd41048da66cad3e8fd3ef72a9d86ecd9009c2db996af29dc62af5ef5eb04d0e16ce8fcecba92a4a9888f52d5d575e7dbc302ed97dbf69df15bb4f5c5601d38fbe3bd89d88768a6aed11ce2f95a6ad30bb72e787bfb734701cea1f38168be44ea19d3e98dd3c953fdb9951ac9c6e221bb0f980d8f0952ac8127da5bda7077dd25ffc8e1515c529f29516dacec6be9c084e6c91698267b2aed9038eca5ebafad479c5fb17652e25bb5b85586fae645bd7c3253d9916c0af65a20253412d5484ac15d288c6ca8823469090ded5ce0975dada63653797129f0e926af6247b457b067db683e37d848e0acf30e5602b78f1848e8da4b640ed08b75f3519a40ec96b2be964234beab37759504376c6e5ebfacdc57e4c7a22cf1e879d7bde29a2dca5fe20420215b59d102fd016606c533e8e36f7da114910664bade9b295d9043a01bc0dc4d8abbc16b1cec7789d89e699ad99dae597c7f10d6f047efc011d67444695cb8e6e8b3dba17ccc693729d01312d0f12a3fc76e12c2e4984af5cb3049b9d8a13124a1f770e96bae1fb153ba4c91bea4fae6f03010275d5a9b14012bdd678e037934dc6762005de54b32a7684e03060d5cc80378e9bef05b8f0692202944401bd06e4553e4490a0e57c5a72fc8abb1f714e22ea950fb2f1de284d6ff3da435954de355c677f60db4252a510919cbe7dadfed0441cf125fd8894753af8114f2ddacb75c3daa460920fc47d285e59fe9110e4151fcef03fa246cd2dd9a4d573e1dbbda1c6968cf4f546289b95ce1bf0a55eea6531382826d4002bc46bf441ce16056d42b5a2079e299e3191c23a7604cde03de6081e06f93cfe632c9a6088cd328662d47a4954934832df5b5f3765dbe136114c73c55cb7ce639e5d40d1d1d8f540d3c8e1bc7423f032c0da5264353468f009c973eec0448e41f9289e8d9dadc68da77d3c3ab3a6477d44024f21fba0bd4477d81c6027657527aa0413b45f417cb7b3beea835a1d5d795414d38156324cb5c1303e9924dbe40cd497c4c23c221cb912058c939bea8b79b3fea360fecaa83375a9a84e338d9e863e8021ad2df4430b8dea0c1714e1bdc478f559705549ad738453ab65c0ffcc8cf0e3bafaf4afad75ecc4dfad0de0cfe27d50d656456ea6c361b76508357714079424";
        let res = AuthData::parser().parse(s);
        assert!(res.is_ok());
        assert_eq!(res.as_ref().unwrap().0.len(), 0);
    }

    #[test]
    fn test_asset_dsc() {
        assert_eq!(
            tuple2(
                strip_whitespace(tag("AssetDsc")),
                strip_whitespace(tag("{"))
            )
            .parse(" AssetDsc { "),
            Ok(("", ((), ())))
        );

        assert_eq!(
            AssetDsc::parser().parse(r#"AssetDsc{"id":"usd","dsc":"USA dollar",}"#),
            Ok((
                "",
                AssetDsc {
                    id: AssetId("usd".into()),
                    dsc: "USA dollar".into()
                }
            ))
        );
        assert_eq!(
            AssetDsc::parser().parse(r#" AssetDsc { "id" : "usd" , "dsc" : "USA dollar" , } "#),
            Ok((
                "",
                AssetDsc {
                    id: AssetId("usd".into()),
                    dsc: "USA dollar".into()
                }
            ))
        );
        assert_eq!(
            AssetDsc::parser()
                .parse(r#" AssetDsc { "id" : "usd" , "dsc" : "USA dollar" , } nice "#),
            Ok((
                "nice ",
                AssetDsc {
                    id: AssetId("usd".into()),
                    dsc: "USA dollar".into()
                }
            ))
        );

        assert_eq!(
            AssetDsc::parser().parse(r#"AssetDsc{"dsc":"USA dollar","id":"usd",}"#),
            Ok((
                "",
                AssetDsc {
                    id: AssetId("usd".into()),
                    dsc: "USA dollar".into()
                }
            ))
        );
    }

    #[test]
    fn test_backet() {
        assert_eq!(
            Backet::parser().parse(r#"Backet{"asset_id":"usd","count":42,}"#),
            Ok((
                "",
                Backet {
                    asset_id: AssetId("usd".into()),
                    count: nz(42)
                }
            ))
        );
        assert_eq!(
            Backet::parser().parse(r#"Backet{"count":42,"asset_id":"usd",}"#),
            Ok((
                "",
                Backet {
                    asset_id: AssetId("usd".into()),
                    count: nz(42)
                }
            ))
        );
    }

    #[test]
    fn test_just_parse_asset_dsc() {
        assert_eq!(
            just_parse::<AssetDsc>(r#"AssetDsc{"id":"usd","dsc":"USA dollar",}"#),
            Ok((
                "",
                AssetDsc {
                    id: AssetId("usd".into()),
                    dsc: "USA dollar".into()
                }
            ))
        );
    }

    #[test]
    fn test_just_parse_backet() {
        assert_eq!(
            just_parse::<Backet>(r#"Backet{"asset_id":"milk","count":3,}"#),
            Ok((
                "",
                Backet {
                    asset_id: AssetId("milk".into()),
                    count: nz(3)
                }
            ))
        );
    }

    #[test]
    fn test_just_parse_user_cash() {
        assert_eq!(
            just_parse::<UserCash>(r#"UserCash{"user_id":"Alice","count":500,}"#),
            Ok((
                "",
                UserCash {
                    user_id: UserId("Alice".into()),
                    count: nz(500)
                }
            ))
        );
    }

    #[test]
    fn test_just_parse_user_backet() {
        assert_eq!(
            just_parse::<UserBacket>(
                r#"UserBacket{"user_id":"Bob","backet":Backet{"asset_id":"milk","count":3,},}"#
            ),
            Ok((
                "",
                UserBacket {
                    user_id: UserId("Bob".into()),
                    backet: Backet {
                        asset_id: AssetId("milk".into()),
                        count: nz(3)
                    }
                }
            ))
        );
    }

    #[test]
    fn test_just_parse_user_backets() {
        assert_eq!(
            just_parse::<UserBackets>(
                r#"UserBackets{"user_id":"Bob","backets":[Backet{"asset_id":"milk","count":3,},],}"#
            ),
            Ok((
                "",
                UserBackets {
                    user_id: UserId("Bob".into()),
                    backets: vec![Backet {
                        asset_id: AssetId("milk".into()),
                        count: nz(3)
                    }]
                }
            ))
        );
    }

    #[test]
    fn test_just_parse_announcements() {
        assert_eq!(
            just_parse::<Announcements>(
                r#"[UserBackets{"user_id":"Bob","backets":[Backet{"asset_id":"milk","count":3,},],},]"#
            ),
            Ok((
                "",
                Announcements(vec![UserBackets {
                    user_id: UserId("Bob".into()),
                    backets: vec![Backet {
                        asset_id: AssetId("milk".into()),
                        count: nz(3)
                    }]
                }])
            ))
        );
    }

    #[test]
    fn test_just_parse_error_cases() {
        assert!(just_parse::<AssetDsc>("invalid input").is_err());
        assert!(just_parse::<Backet>("").is_err());
        assert!(just_parse::<Announcements>("not a list").is_err());
    }

    #[test]
    fn test_authdata_error_cases() {
        // Empty input
        assert!(AuthData::parser().parse("").is_err());
        // Too short (only a few hex digits)
        assert!(AuthData::parser().parse("30c305").is_err());
        // Non-hex characters
        assert!(AuthData::parser().parse("zzzz").is_err());
    }

    #[test]
    fn test_asset_dsc_error_cases() {
        // Wrong type name
        assert!(AssetDsc::parser().parse(r#"Bucket{"id":"usd","dsc":"dollar",}"#).is_err());
        // Missing closing brace
        assert!(AssetDsc::parser().parse(r#"AssetDsc{"id":"usd","dsc":"dollar","#).is_err());
        // Missing required field "dsc"
        assert!(AssetDsc::parser().parse(r#"AssetDsc{"id":"usd",}"#).is_err());
        // Missing required field "id"
        assert!(AssetDsc::parser().parse(r#"AssetDsc{"dsc":"dollar",}"#).is_err());
        // Empty input
        assert!(AssetDsc::parser().parse("").is_err());
    }

    #[test]
    fn test_backet_error_cases() {
        // Wrong type name
        assert!(Backet::parser().parse(r#"Basket{"asset_id":"usd","count":1,}"#).is_err());
        // Missing closing brace
        assert!(Backet::parser().parse(r#"Backet{"asset_id":"usd","count":1,"#).is_err());
        // Missing required field "count"
        assert!(Backet::parser().parse(r#"Backet{"asset_id":"usd",}"#).is_err());
        // Missing required field "asset_id"
        assert!(Backet::parser().parse(r#"Backet{"count":1,}"#).is_err());
        // Zero count (NonZeroU32 rejects zero)
        assert!(Backet::parser().parse(r#"Backet{"asset_id":"usd","count":0,}"#).is_err());
        // Empty input
        assert!(Backet::parser().parse("").is_err());
    }

    #[test]
    fn test_user_cash_error_cases() {
        // Wrong type name
        assert!(UserCash::parser().parse(r#"Cash{"user_id":"Bob","count":1,}"#).is_err());
        // Missing closing brace
        assert!(UserCash::parser().parse(r#"UserCash{"user_id":"Bob","count":1,"#).is_err());
        // Missing required field "count"
        assert!(UserCash::parser().parse(r#"UserCash{"user_id":"Bob",}"#).is_err());
        // Missing required field "user_id"
        assert!(UserCash::parser().parse(r#"UserCash{"count":1,}"#).is_err());
        // Zero count
        assert!(UserCash::parser().parse(r#"UserCash{"user_id":"Bob","count":0,}"#).is_err());
        // Empty input
        assert!(UserCash::parser().parse("").is_err());
    }

    #[test]
    fn test_user_backet_error_cases() {
        // Wrong type name
        assert!(UserBacket::parser().parse(r#"Backet{"user_id":"Bob","backet":Backet{"asset_id":"x","count":1,},}"#).is_err());
        // Missing closing brace
        assert!(UserBacket::parser().parse(r#"UserBacket{"user_id":"Bob","backet":Backet{"asset_id":"x","count":1,},"#).is_err());
        // Missing required field "backet"
        assert!(UserBacket::parser().parse(r#"UserBacket{"user_id":"Bob",}"#).is_err());
        // Missing required field "user_id"
        assert!(UserBacket::parser().parse(r#"UserBacket{"backet":Backet{"asset_id":"x","count":1,},}"#).is_err());
        // Empty input
        assert!(UserBacket::parser().parse("").is_err());
    }

    #[test]
    fn test_user_backets_error_cases() {
        // Wrong type name
        assert!(UserBackets::parser().parse(r#"Backets{"user_id":"Bob","backets":[],}"#).is_err());
        // Missing closing brace
        assert!(UserBackets::parser().parse(r#"UserBackets{"user_id":"Bob","backets":[],"#).is_err());
        // Missing required field "backets"
        assert!(UserBackets::parser().parse(r#"UserBackets{"user_id":"Bob",}"#).is_err());
        // Missing required field "user_id"
        assert!(UserBackets::parser().parse(r#"UserBackets{"backets":[],}"#).is_err());
        // Empty input
        assert!(UserBackets::parser().parse("").is_err());
    }

    #[test]
    fn test_announcements_error_cases() {
        // Not a list (no brackets)
        assert!(Announcements::parser().parse("not a list").is_err());
        // Unclosed bracket
        assert!(Announcements::parser().parse("[").is_err());
        // Invalid element inside list
        assert!(Announcements::parser().parse("[invalid,]").is_err());
        // Empty input
        assert!(Announcements::parser().parse("").is_err());
    }
}
