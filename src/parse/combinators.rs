/// Structured error type for parser failures.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    /// Input did not match the expected pattern.
    #[error("unexpected input: {0}")]
    UnexpectedInput(&'static str),
    /// Input ended before the parser could finish.
    #[error("incomplete input: {0}")]
    IncompleteInput(&'static str),
    /// A parsed value was out of range or otherwise invalid.
    #[error("invalid value: {0}")]
    InvalidValue(&'static str),
}

/// Трейт, чтобы **реализовывать** и **требовать** метод 'распарсь и покажи,
/// что распарсить осталось'
pub trait Parser {
    type Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError>;

    /// Fluent combinator: transform the parsed value with a mapping function.
    ///
    /// Equivalent to `Map::new(self, f)`, but chainable:
    /// `tag("foo").map(|_| 42)`
    fn map<Dest, F: Fn(Self::Dest) -> Dest>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map { parser: self, map: f }
    }

    /// Fluent combinator: require a prefix parser to succeed first, discarding its result.
    ///
    /// `self` is the main parser whose result is kept. Equivalent to
    /// `Preceded { prefix_to_ignore: prefix, dest_parser: self }`.
    ///
    /// Example: `tag("Error").preceded_by(tag("System::"))` parses `"System::Error"`,
    /// discards the `"System::"` match, and returns `()` from the `"Error"` tag.
    fn preceded_by<P: Parser>(self, prefix: P) -> Preceded<P, Self>
    where
        Self: Sized,
    {
        Preceded {
            prefix_to_ignore: prefix,
            dest_parser: self,
        }
    }

    /// Fluent combinator: strip leading whitespace before and after parsing.
    ///
    /// Equivalent to `StripWhitespace { parser: self }`.
    fn strip_ws(self) -> StripWhitespace<Self>
    where
        Self: Sized,
    {
        StripWhitespace { parser: self }
    }
}
/// Вспомогательный трейт, чтобы писать собственный десериализатор
/// (по решаемой задаче - отдалённый аналог `serde::Deserialize`)
pub trait Parsable: Sized {
    type Parser: Parser<Dest = Self>;
    fn parser() -> Self::Parser;
}

pub(crate) mod primitives {
    // parsers for std types
    use super::{ParseError, Parser};
    use std::num::NonZeroU32;

    /// Беззнаковые числа
    #[derive(Debug)]
    pub struct U32;
    impl Parser for U32 {
        type Dest = NonZeroU32;
        fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
            let (remaining, is_hex) = input
                .strip_prefix("0x")
                .map_or((input, false), |remaining| (remaining, true));
            let end_idx = remaining
                .char_indices()
                .find_map(|(idx, c)| match (is_hex, c) {
                    (true, 'a'..='f' | '0'..='9' | 'A'..='F') => None,
                    (false, '0'..='9') => None,
                    _ => Some(idx),
                })
                .unwrap_or(remaining.len());
            let value = u32::from_str_radix(&remaining[..end_idx], if is_hex { 16 } else { 10 })
                .map_err(|_| ParseError::InvalidValue("invalid u32 literal"))?;
            let non_zero = NonZeroU32::new(value).ok_or(ParseError::InvalidValue("zero is not allowed"))?;
            Ok((&remaining[end_idx..], non_zero))
        }
    }
    /// Знаковые числа
    #[cfg(test)]
    #[derive(Debug)]
    pub struct I32;
    #[cfg(test)]
    impl Parser for I32 {
        type Dest = i32;
        fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
            let end_idx = input
                .char_indices()
                .skip(1)
                .find_map(|(idx, c)| (!c.is_ascii_digit()).then_some(idx))
                .unwrap_or(input.len());
            let value = input[..end_idx].parse().map_err(|_| ParseError::InvalidValue("invalid i32 literal"))?;
            if value == 0 {
                return Err(ParseError::InvalidValue("zero is not allowed")); // в наших логах нет нулей, ноль в операции - фикция
            }
            Ok((&input[end_idx..], value))
        }
    }
    /// Шестнадцатеричные байты (пригодится при парсинге блобов)
    #[derive(Debug, Clone)]
    pub struct Byte;
    impl Parser for Byte {
        type Dest = u8;
        fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
            let (to_parse, remaining) = input.split_at_checked(2).ok_or(ParseError::IncompleteInput("expected 2 hex digits"))?;
            if !to_parse.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ParseError::UnexpectedInput("expected hex digit"));
            }
            let value = u8::from_str_radix(to_parse, 16).map_err(|_| ParseError::InvalidValue("invalid hex byte"))?;
            Ok((remaining, value))
        }
    }
}

/// Распарсить строку, которую ранее [обернули в кавычки](quote)
// `"abc\"def\\ghi"nice` -> (`abcd"def\ghi`, `nice`)
fn unquote_escaped(input: &str) -> Result<(&str, String), ParseError> {
    let mut result = String::new();
    let mut escaped_now = false;
    let mut chars = input.strip_prefix("\"").ok_or(ParseError::UnexpectedInput("expected opening quote"))?.chars();
    while let Some(c) = chars.next() {
        match (c, escaped_now) {
            ('"' | '\\', true) => {
                result.push(c);
                escaped_now = false;
            }
            ('\\', false) => escaped_now = true,
            ('"', false) => return Ok((chars.as_str(), result)),
            (c, _) => {
                result.push(c);
                escaped_now = false;
            }
        }
    }
    Err(ParseError::IncompleteInput("unclosed quote")) // строка кончилась, не закрыв кавычку
}
/// Распарсить строку, обёрную в кавычки
/// (сокращённая версия [unquote_escaped], в которой вложенные кавычки не предусмотрены)
fn unquote_simple(input: &str) -> Result<(&str, &str), ParseError> {
    let input = input.strip_prefix("\"").ok_or(ParseError::UnexpectedInput("expected opening quote"))?;
    let quote_byteidx = input.find('"').ok_or(ParseError::IncompleteInput("unclosed quote"))?;
    if 0 == quote_byteidx || Some("\\") == input.get(quote_byteidx - 1..quote_byteidx) {
        return Err(ParseError::UnexpectedInput("empty or escaped quote"));
    }
    Ok((&input[1 + quote_byteidx..], &input[..quote_byteidx]))
}
/// Парсер кавычек
#[derive(Debug, Clone)]
pub struct Unquote;
impl Parser for Unquote {
    type Dest = String;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        unquote_escaped(input)
    }
}
/// Конструктор [Unquote]
pub(crate) fn unquote() -> Unquote {
    Unquote
}
/// Парсер константных строк
/// (аналог `nom::bytes::complete::tag`)
#[derive(Debug, Clone)]
pub struct Tag {
    tag: &'static str,
}
impl Parser for Tag {
    type Dest = ();
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        Ok((input.strip_prefix(self.tag).ok_or(ParseError::UnexpectedInput("tag mismatch"))?, ()))
    }
}
/// Конструктор [Tag]
pub(crate) fn tag(tag: &'static str) -> Tag {
    Tag { tag }
}
/// Парсер [тэга](Tag), обёрнутого в кавычки
#[derive(Debug, Clone)]
pub(crate) struct QuotedTag(Tag);
impl Parser for QuotedTag {
    type Dest = ();
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let (remaining, candidate) = unquote_simple(input)?;
        if !self.0.parse(candidate)?.0.is_empty() {
            return Err(ParseError::UnexpectedInput("quoted tag has trailing content"));
        }
        Ok((remaining, ()))
    }
}
/// Конструктор [QuotedTag]
pub(crate) fn quoted_tag(tag: &'static str) -> QuotedTag {
    QuotedTag(Tag { tag })
}
/// Комбинатор, пробрасывающий строку без лидирующих пробелов
#[derive(Debug, Clone)]
pub struct StripWhitespace<T> {
    parser: T,
}
impl<T: Parser> Parser for StripWhitespace<T> {
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        self.parser
            .parse(input.trim_start())
            .map(|(remaining, parsed)| (remaining.trim_start(), parsed))
    }
}
/// Конструктор [StripWhitespace]
pub(crate) fn strip_whitespace<T: Parser>(parser: T) -> StripWhitespace<T> {
    StripWhitespace { parser }
}
/// Комбинатор, чтобы распарсить нужное, окружённое в начале и в конце чем-то
/// обязательным, не участвующем в результате.
/// Пробрасывает строку в парсер1, оставшуюся строку после первого
/// парсинга - в парсер2, оставшуюся строку после второго парсинга - в парсер3.
/// Результат парсера2 будет результатом этого комбинатора, а оставшейся
/// строкой - строка, оставшаяся после парсера3.
/// (аналог `delimited` из `nom`)
#[derive(Debug, Clone)]
pub struct Delimited<Prefix, T, Suffix> {
    prefix_to_ignore: Prefix,
    dest_parser: T,
    suffix_to_ignore: Suffix,
}
impl<Prefix, T, Suffix> Parser for Delimited<Prefix, T, Suffix>
where
    Prefix: Parser,
    T: Parser,
    Suffix: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let (remaining, _) = self.prefix_to_ignore.parse(input)?;
        let (remaining, result) = self.dest_parser.parse(remaining)?;
        self.suffix_to_ignore
            .parse(remaining)
            .map(|(remaining, _)| (remaining, result))
    }
}
/// Конструктор [Delimited]
pub(crate) fn delimited<Prefix, T, Suffix>(
    prefix_to_ignore: Prefix,
    dest_parser: T,
    suffix_to_ignore: Suffix,
) -> Delimited<Prefix, T, Suffix>
where
    Prefix: Parser,
    T: Parser,
    Suffix: Parser,
{
    Delimited {
        prefix_to_ignore,
        dest_parser,
        suffix_to_ignore,
    }
}
/// Комбинатор-отображение. Парсит дочерним парсером, преобразует результат так,
/// как вызывающему хочется
#[derive(Debug, Clone)]
pub struct Map<T, M> {
    parser: T,
    map: M,
}
impl<T: Parser, Dest: Sized, M: Fn(T::Dest) -> Dest> Parser for Map<T, M> {
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        self.parser
            .parse(input)
            .map(|(remaining, pre_result)| (remaining, (self.map)(pre_result)))
    }
}
/// Комбинатор с отбрасываемым префиксом, упрощённая версия [Delimited]
/// (аналог `preceeded` из `nom`)
#[derive(Debug, Clone)]
pub struct Preceded<Prefix, T> {
    prefix_to_ignore: Prefix,
    dest_parser: T,
}
impl<Prefix, T> Parser for Preceded<Prefix, T>
where
    Prefix: Parser,
    T: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let (remaining, _) = self.prefix_to_ignore.parse(input)?;
        self.dest_parser.parse(remaining)
    }
}
/// Комбинатор, который требует, чтобы все дочерние парсеры отработали,
/// (аналог `tuple` из `nom`)
#[derive(Debug, Clone)]
pub struct Tuple<T> {
    parser: T,
}
macro_rules! impl_tuple {
    ($fn_name:ident, [ $($A:ident $a:ident $idx:tt),+ ], $LastA:ident $last_a:ident $last_idx:tt) => {
        impl_tuple!(@impl [ $($A $a $idx),+ ], $LastA $last_a $last_idx);
        pub(crate) fn $fn_name<$($A: Parser,)+ $LastA: Parser>(
            $($a: $A,)+ $last_a: $LastA,
        ) -> Tuple<($($A,)+ $LastA)> {
            Tuple { parser: ($($a,)+ $last_a) }
        }
    };
    (@impl [ $($A:ident $a:ident $idx:tt),+ ], $LastA:ident $last_a:ident $last_idx:tt) => {
        impl<$($A: Parser,)+ $LastA: Parser> Parser for Tuple<($($A,)+ $LastA)> {
            type Dest = ($($A::Dest,)+ $LastA::Dest);
            fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
                let remaining = input;
                $(let (remaining, $a) = self.parser.$idx.parse(remaining)?;)+
                self.parser.$last_idx.parse(remaining)
                    .map(|(remaining, $last_a)| (remaining, ($($a,)+ $last_a)))
            }
        }
    };
}
impl_tuple!(tuple2, [A0 a0 0], A1 a1 1);
impl_tuple!(@impl [A0 a0 0, A1 a1 1], A2 a2 2);
impl_tuple!(@impl [A0 a0 0, A1 a1 1, A2 a2 2], A3 a3 3);
/// Комбинатор, который вытаскивает значения из пары `"ключ":значение,`.
/// Для простоты реализации, запятая всегда нужна в конце пары ключ-значение,
/// простое '"ключ":значение' читаться не будет
#[derive(Debug, Clone)]
pub struct KeyValue<T> {
    #[allow(clippy::type_complexity)]
    parser: Delimited<
        Tuple<(StripWhitespace<QuotedTag>, StripWhitespace<Tag>)>,
        StripWhitespace<T>,
        StripWhitespace<Tag>,
    >,
}
impl<T> Parser for KeyValue<T>
where
    T: Parser,
{
    type Dest = T::Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        self.parser.parse(input)
    }
}
/// Конструктор [KeyValue]
pub(crate) fn key_value<T: Parser>(key: &'static str, value_parser: T) -> KeyValue<T> {
    KeyValue {
        parser: delimited(
            tuple2(
                strip_whitespace(quoted_tag(key)),
                strip_whitespace(tag(":")),
            ),
            strip_whitespace(value_parser),
            strip_whitespace(tag(",")),
        ),
    }
}
/// Комбинатор, который возвращает результаты дочерних парсеров, если их
/// удалось применить друг после друга в любом порядке. Результат возвращается в
/// том порядке, в каком `Permutation` был сконструирован
/// (аналог `permutation` из `nom`)
#[derive(Debug, Clone)]
pub struct Permutation<T> {
    parsers: T,
}
impl<A0, A1> Parser for Permutation<(A0, A1)>
where
    A0: Parser,
    A1: Parser,
{
    type Dest = (A0::Dest, A1::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        match self.parsers.0.parse(input) {
            Ok((remaining, a0)) => self
                .parsers
                .1
                .parse(remaining)
                .map(|(remaining, a1)| (remaining, (a0, a1))),
            Err(_) => self.parsers.1.parse(input).and_then(|(remaining, a1)| {
                self.parsers
                    .0
                    .parse(remaining)
                    .map(|(remaining, a0)| (remaining, (a0, a1)))
            }),
        }
    }
}
macro_rules! permutation_fn {
    ($fn_name:ident, $($A:ident $a:ident),+) => {
        pub(crate) fn $fn_name<$($A: Parser),+>($($a: $A),+) -> Permutation<($($A),+)> {
            Permutation { parsers: ($($a),+) }
        }
    };
}
permutation_fn!(permutation2, A0 a0, A1 a1);
impl<A0, A1, A2> Parser for Permutation<(A0, A1, A2)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        match self.parsers.0.parse(input) {
            Ok((remaining, a0)) => match self.parsers.1.parse(remaining) {
                Ok((remaining, a1)) => self
                    .parsers
                    .2
                    .parse(remaining)
                    .map(|(remaining, a2)| (remaining, (a0, a1, a2))),
                Err(_) => self.parsers.2.parse(remaining).and_then(|(remaining, a2)| {
                    self.parsers
                        .1
                        .parse(remaining)
                        .map(|(remaining, a1)| (remaining, (a0, a1, a2)))
                }),
            },
            Err(_) => match self.parsers.1.parse(input) {
                Ok((remaining, a1)) => match self.parsers.0.parse(remaining) {
                    Ok((remaining, a0)) => self
                        .parsers
                        .2
                        .parse(remaining)
                        .map(|(remaining, a2)| (remaining, (a0, a1, a2))),
                    Err(_) => self.parsers.2.parse(remaining).and_then(|(remaining, a2)| {
                        self.parsers
                            .0
                            .parse(remaining)
                            .map(|(remaining, a0)| (remaining, (a0, a1, a2)))
                    }),
                },
                Err(_) => self.parsers.2.parse(input).and_then(|(remaining, a2)| {
                    match self.parsers.0.parse(remaining) {
                        Ok((remaining, a0)) => self
                            .parsers
                            .1
                            .parse(remaining)
                            .map(|(remaining, a1)| (remaining, (a0, a1, a2))),
                        Err(_) => self.parsers.1.parse(remaining).and_then(|(remaining, a1)| {
                            self.parsers
                                .0
                                .parse(remaining)
                                .map(|(remaining, a0)| (remaining, (a0, a1, a2)))
                        }),
                    }
                }),
            },
        }
    }
}
permutation_fn!(permutation3, A0 a0, A1 a1, A2 a2);
/// Комбинатор списка из любого числа элементов, которые надо читать
/// вложенным парсером. Граница списка определяется квадратными (`[`&`]`)
/// скобками.
/// Для простоты реализации, после каждого элемента списка должна быть запятая
#[derive(Debug, Clone)]
pub struct List<T> {
    parser: T,
}
impl<T: Parser> Parser for List<T> {
    type Dest = Vec<T::Dest>;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let mut remaining = input.trim_start().strip_prefix('[').ok_or(ParseError::UnexpectedInput("expected '['"))?.trim_start();
        let mut result = Vec::new();
        while !remaining.is_empty() {
            match remaining.strip_prefix(']') {
                Some(remaining) => return Ok((remaining.trim_start(), result)),
                None => {
                    let (new_remaining, item) = self.parser.parse(remaining)?;
                    let new_remaining = new_remaining
                        .trim_start()
                        .strip_prefix(',')
                        .ok_or(ParseError::UnexpectedInput("expected ',' after list element"))?
                        .trim_start();
                    result.push(item);
                    remaining = new_remaining;
                }
            }
        }
        Err(ParseError::IncompleteInput("unclosed list bracket")) // строка кончилась, не закрыв скобку
    }
}
/// Конструктор для [List]
pub(crate) fn list<T: Parser>(parser: T) -> List<T> {
    List { parser }
}
/// Комбинатор, который вернёт тот результат, который будет успешно
/// получен первым из дочерних комбинаторов
/// (аналог `alt` из `nom`)
#[derive(Debug, Clone)]
pub struct Alt<T> {
    parser: T,
}
macro_rules! impl_alt {
    ($fn_name:ident [ $($A:ident $a:ident $idx:tt),+ ] $LastA:ident $last_a:ident $last_idx:tt) => {
        impl_alt!(@impl [ $($A $a $idx),+ ] $LastA $last_a $last_idx);
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn $fn_name<Dest, $($A: Parser<Dest = Dest>,)+ $LastA: Parser<Dest = Dest>>(
            $($a: $A,)+ $last_a: $LastA,
        ) -> Alt<($($A,)+ $LastA)> {
            Alt { parser: ($($a,)+ $last_a) }
        }
    };
    (@impl [ $($A:ident $a:ident $idx:tt),+ ] $LastA:ident $last_a:ident $last_idx:tt) => {
        impl<$($A,)+ $LastA, Dest> Parser for Alt<($($A,)+ $LastA)>
        where
            $($A: Parser<Dest = Dest>,)+
            $LastA: Parser<Dest = Dest>,
        {
            type Dest = Dest;
            fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
                $(if let Ok(ok) = self.parser.$idx.parse(input) { return Ok(ok); })+
                self.parser.$last_idx.parse(input)
            }
        }
    };
}
impl_alt!(alt2 [A0 a0 0] A1 a1 1);
impl_alt!(alt3 [A0 a0 0, A1 a1 1] A2 a2 2);
impl_alt!(alt4 [A0 a0 0, A1 a1 1, A2 a2 2] A3 a3 3);
impl_alt!(@impl [A0 a0 0, A1 a1 1, A2 a2 2, A3 a3 3] A4 a4 4);
impl_alt!(@impl [A0 a0 0, A1 a1 1, A2 a2 2, A3 a3 3, A4 a4 4] A5 a5 5);
impl_alt!(@impl [A0 a0 0, A1 a1 1, A2 a2 2, A3 a3 3, A4 a4 4, A5 a5 5] A6 a6 6);
impl_alt!(alt8 [A0 a0 0, A1 a1 1, A2 a2 2, A3 a3 3, A4 a4 4, A5 a5 5, A6 a6 6] A7 a7 7);

/// Комбинатор для применения дочернего парсера N раз
/// (аналог `take` из `nom`)
pub struct Take<T> {
    count: usize,
    parser: T,
}
impl<T: Parser> Parser for Take<T> {
    type Dest = Vec<T::Dest>;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let mut remaining = input;
        let mut result = Vec::new();
        for _ in 0..self.count {
            let (new_remaining, new_result) = self.parser.parse(remaining)?;
            result.push(new_result);
            remaining = new_remaining;
        }
        Ok((remaining, result))
    }
}
/// Конструктор `Take`
pub(crate) fn take<T: Parser>(count: usize, parser: T) -> Take<T> {
    Take { count, parser }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;
    use proptest::prelude::*;

    fn nz(n: u32) -> NonZeroU32 {
        NonZeroU32::new(n).unwrap()
    }

    #[test]
    fn test_u32() {
        assert_eq!(
            primitives::U32.parse("411"),
            Ok(("", NonZeroU32::new(411).unwrap()))
        );
        assert_eq!(
            primitives::U32.parse("411ab"),
            Ok(("ab", NonZeroU32::new(411).unwrap()))
        );
        assert!(primitives::U32.parse("").is_err());
        assert!(primitives::U32.parse("-3").is_err());
        assert_eq!(
            primitives::U32.parse("0x03"),
            Ok(("", NonZeroU32::new(0x3).unwrap()))
        );
        assert_eq!(
            primitives::U32.parse("0x03abg"),
            Ok(("g", NonZeroU32::new(0x3ab).unwrap()))
        );
        assert!(primitives::U32.parse("0x").is_err());
    }

    #[test]
    fn test_i32() {
        assert_eq!(primitives::I32.parse("411"), Ok(("", 411)));
        assert_eq!(primitives::I32.parse("411ab"), Ok(("ab", 411)));
        assert!(primitives::I32.parse("").is_err());
        assert_eq!(primitives::I32.parse("-3"), Ok(("", -3)));
        assert!(primitives::I32.parse("0x03").is_err());
        assert!(primitives::I32.parse("-").is_err());
    }

    fn quote(input: &str) -> String {
        let mut result = String::from("\"");
        result.extend(
            input
                .chars()
                .flat_map(|c| match c {
                    '\\' | '"' => ['\\', c].into_iter().take(2),
                    _ => [c, ' '].into_iter().take(1),
                }),
        );
        result.push('"');
        result
    }

    #[test]
    fn test_quote() {
        assert_eq!(quote(r#"411"#), r#""411""#.to_string());
        assert_eq!(quote(r#"4\11""#), r#""4\\11\"""#.to_string());
    }

    #[test]
    fn test_unquote_simple() {
        assert_eq!(unquote_simple(r#""411""#), Ok(("", "411")));
        assert!(unquote_simple(r#" "411""#).is_err());
        assert!(unquote_simple(r#"411"#).is_err());
    }

    #[test]
    fn test_unquote() {
        assert_eq!(Unquote.parse(r#""411""#), Ok(("", "411".into())));
        assert!(Unquote.parse(r#" "411""#).is_err());
        assert!(Unquote.parse(r#"411"#).is_err());

        assert_eq!(Unquote.parse(r#""ni\\c\"e""#), Ok(("", r#"ni\c"e"#.into())));
    }

    #[test]
    fn test_tag() {
        assert_eq!(tag("key=").parse("key=value"), Ok(("value", ())));
        assert!(tag("key=").parse("key:value").is_err());
    }

    #[test]
    fn test_quoted_tag() {
        assert_eq!(
            quoted_tag("key").parse(r#""key"=value"#),
            Ok(("=value", ()))
        );
        assert!(quoted_tag("key").parse(r#""key:"value"#).is_err());
        assert!(quoted_tag("key").parse(r#"key=value"#).is_err());
    }

    #[test]
    fn test_strip_whitespace() {
        assert_eq!(
            strip_whitespace(tag("hello")).parse(" hello world"),
            Ok(("world", ()))
        );
        assert_eq!(strip_whitespace(tag("hello")).parse("hello"), Ok(("", ())));
        assert_eq!(
            strip_whitespace(primitives::U32).parse(" 42 answer"),
            Ok(("answer", nz(42)))
        );
    }

    #[test]
    fn test_delimited() {
        assert_eq!(
            delimited(tag("["), primitives::U32, tag("]")).parse("[0x32]"),
            Ok(("", nz(0x32)))
        );
        assert_eq!(
            delimited(tag("["), primitives::U32, tag("]")).parse("[0x32] nice"),
            Ok((" nice", nz(0x32)))
        );
        assert!(
            delimited(tag("["), primitives::U32, tag("]")).parse("0x32]").is_err()
        );
        assert!(
            delimited(tag("["), primitives::U32, tag("]")).parse("[0x32").is_err()
        );
    }

    #[test]
    fn test_key_value() {
        assert_eq!(
            key_value("key", primitives::U32).parse(r#""key":32,"#),
            Ok(("", nz(32)))
        );
        assert!(
            key_value("key", primitives::U32).parse(r#"key:32,"#).is_err()
        );
        assert!(
            key_value("key", primitives::U32).parse(r#""key":32"#).is_err()
        );
        assert_eq!(
            key_value("key", primitives::U32).parse(r#" "key" : 32 , nice"#),
            Ok(("nice", nz(32)))
        );
    }

    #[test]
    fn test_list() {
        assert_eq!(
            list(primitives::U32).parse("[1,2,3,4,]"),
            Ok(("", vec![nz(1), nz(2), nz(3), nz(4)]))
        );
        assert_eq!(
            list(primitives::U32).parse(" [ 1 , 2 , 3 , 4 , ] nice"),
            Ok(("nice", vec![nz(1), nz(2), nz(3), nz(4)]))
        );
        assert!(list(primitives::U32).parse("1,2,3,4,").is_err());
        assert_eq!(list(primitives::U32).parse("[]"), Ok(("", vec![])));
    }

    #[test]
    fn test_permutation3_all_orderings() {
        let parser = || {
            permutation3(
                key_value("a", primitives::U32),
                key_value("b", primitives::U32),
                key_value("c", primitives::U32),
            )
        };
        let expected = (nz(1), nz(2), nz(3));

        // a, b, c
        assert_eq!(
            parser().parse(r#""a":1,"b":2,"c":3,"#),
            Ok(("", expected))
        );
        // a, c, b
        assert_eq!(
            parser().parse(r#""a":1,"c":3,"b":2,"#),
            Ok(("", expected))
        );
        // b, a, c
        assert_eq!(
            parser().parse(r#""b":2,"a":1,"c":3,"#),
            Ok(("", expected))
        );
        // b, c, a
        assert_eq!(
            parser().parse(r#""b":2,"c":3,"a":1,"#),
            Ok(("", expected))
        );
        // c, a, b
        assert_eq!(
            parser().parse(r#""c":3,"a":1,"b":2,"#),
            Ok(("", expected))
        );
        // c, b, a
        assert_eq!(
            parser().parse(r#""c":3,"b":2,"a":1,"#),
            Ok(("", expected))
        );
    }

    #[test]
    fn test_permutation3_error_on_missing_field() {
        let parser = permutation3(
            key_value("a", primitives::U32),
            key_value("b", primitives::U32),
            key_value("c", primitives::U32),
        );
        // Only two of three fields provided
        assert!(parser.parse(r#""a":1,"b":2,"#).is_err());
    }

    #[test]
    fn test_fluent_api_chaining() {
        // Demonstrates the chaining pattern from phase 22.5:
        // tag("Error").preceded_by(tag("System::")).map(|_| ...)
        let parser = tag("Error")
            .preceded_by(tag("System::"))
            .map(|_| "matched_system_error");
        assert_eq!(
            parser.parse("System::Error rest"),
            Ok((" rest", "matched_system_error"))
        );
        assert!(parser.parse("App::Error rest").is_err());
        assert!(parser.parse("System::Trace rest").is_err());

        // Chaining .strip_ws() with .preceded_by() and .map()
        let parser = unquote()
            .strip_ws()
            .preceded_by(tag("NetworkError").strip_ws())
            .preceded_by(tag("Error"))
            .map(|msg: String| format!("net: {msg}"));
        assert_eq!(
            parser.parse(r#"Error NetworkError "url unknown""#),
            Ok(("", "net: url unknown".to_string()))
        );
    }

    proptest! {
        #[test]
        fn test_quote_unquote_roundtrip(s in ".*") {
            let quoted = quote(&s);
            let result = unquote_escaped(&quoted);
            prop_assert_eq!(result, Ok(("", s)));
        }
    }
}
