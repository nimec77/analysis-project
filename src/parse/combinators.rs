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
    #[derive(Debug)]
    pub struct I32;
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

/// Обернуть строку в кавычки, экранировав кавычки, которые в строке уже есть
fn quote(input: &str) -> String {
    let mut result = String::from("\"");
    result.extend(
        input
            .chars()
            .map(|c| match c {
                '\\' | '"' => ['\\', c].into_iter().take(2),
                _ => [c, ' '].into_iter().take(1),
            })
            .flatten(),
    );
    result.push('"');
    result
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
/// Конструктор [Map]
pub(crate) fn map<T: Parser, Dest: Sized, M: Fn(T::Dest) -> Dest>(parser: T, map: M) -> Map<T, M> {
    Map { parser, map }
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
/// Конструктор [Preceded]
pub(crate) fn preceded<Prefix, T>(prefix_to_ignore: Prefix, dest_parser: T) -> Preceded<Prefix, T>
where
    Prefix: Parser,
    T: Parser,
{
    Preceded {
        prefix_to_ignore,
        dest_parser,
    }
}
/// Комбинатор, который требует, чтобы все дочерние парсеры отработали,
/// (аналог `tuple` из `nom`)
#[derive(Debug, Clone)]
pub struct Tuple<T> {
    parser: T,
}
impl<A0, A1> Parser for Tuple<(A0, A1)>
where
    A0: Parser,
    A1: Parser,
{
    type Dest = (A0::Dest, A1::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        self.parser
            .1
            .parse(remaining)
            .map(|(remaining, a1)| (remaining, (a0, a1)))
    }
}
/// Конструктор [Tuple] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn tuple2<A0: Parser, A1: Parser>(a0: A0, a1: A1) -> Tuple<(A0, A1)> {
    Tuple { parser: (a0, a1) }
}
impl<A0, A1, A2> Parser for Tuple<(A0, A1, A2)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        let (remaining, a1) = self.parser.1.parse(remaining)?;
        self.parser
            .2
            .parse(remaining)
            .map(|(remaining, a2)| (remaining, (a0, a1, a2)))
    }
}
impl<A0, A1, A2, A3> Parser for Tuple<(A0, A1, A2, A3)>
where
    A0: Parser,
    A1: Parser,
    A2: Parser,
    A3: Parser,
{
    type Dest = (A0::Dest, A1::Dest, A2::Dest, A3::Dest);
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        let (remaining, a0) = self.parser.0.parse(input)?;
        let (remaining, a1) = self.parser.1.parse(remaining)?;
        let (remaining, a2) = self.parser.2.parse(remaining)?;
        self.parser
            .3
            .parse(remaining)
            .map(|(remaining, a3)| (remaining, (a0, a1, a2, a3)))
    }
}
/// Комбинатор, который вытаскивает значения из пары `"ключ":значение,`.
/// Для простоты реализации, запятая всегда нужна в конце пары ключ-значение,
/// простое '"ключ":значение' читаться не будет
#[derive(Debug, Clone)]
pub struct KeyValue<T> {
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
/// Конструктор [Permutation] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn permutation2<A0: Parser, A1: Parser>(a0: A0, a1: A1) -> Permutation<(A0, A1)> {
    Permutation { parsers: (a0, a1) }
}
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
/// Конструктор [Permutation] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn permutation3<A0: Parser, A1: Parser, A2: Parser>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> Permutation<(A0, A1, A2)> {
    Permutation {
        parsers: (a0, a1, a2),
    }
}
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
impl<A0, A1, Dest> Parser for Alt<(A0, A1)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        if let Ok(ok) = self.parser.0.parse(input) {
            return Ok(ok);
        }
        self.parser.1.parse(input)
    }
}
/// Конструктор [Alt] для двух парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt2<Dest, A0: Parser<Dest = Dest>, A1: Parser<Dest = Dest>>(
    a0: A0,
    a1: A1,
) -> Alt<(A0, A1)> {
    Alt { parser: (a0, a1) }
}
impl<A0, A1, A2, Dest> Parser for Alt<(A0, A1, A2)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        // match вместо тут не подойдёт - нужно лениво
        if let Ok(ok) = self.parser.0.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.1.parse(input) {
            return Ok(ok);
        }
        self.parser.2.parse(input)
    }
}
/// Конструктор [Alt] для трёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt3<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
) -> Alt<(A0, A1, A2)> {
    Alt {
        parser: (a0, a1, a2),
    }
}
impl<A0, A1, A2, A3, Dest> Parser for Alt<(A0, A1, A2, A3)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        if let Ok(ok) = self.parser.0.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.1.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.2.parse(input) {
            return Ok(ok);
        }
        self.parser.3.parse(input)
    }
}
/// Конструктор [Alt] для четырёх парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt4<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
) -> Alt<(A0, A1, A2, A3)> {
    Alt {
        parser: (a0, a1, a2, a3),
    }
}
impl<A0, A1, A2, A3, A4, A5, A6, A7, Dest> Parser for Alt<(A0, A1, A2, A3, A4, A5, A6, A7)>
where
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
    A4: Parser<Dest = Dest>,
    A5: Parser<Dest = Dest>,
    A6: Parser<Dest = Dest>,
    A7: Parser<Dest = Dest>,
{
    type Dest = Dest;
    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, Self::Dest), ParseError> {
        if let Ok(ok) = self.parser.0.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.1.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.2.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.3.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.4.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.5.parse(input) {
            return Ok(ok);
        }
        if let Ok(ok) = self.parser.6.parse(input) {
            return Ok(ok);
        }
        self.parser.7.parse(input)
    }
}
/// Конструктор [Alt] для восьми парсеров
/// (в Rust нет чего-то, вроде variadic templates из C++)
pub(crate) fn alt8<
    Dest,
    A0: Parser<Dest = Dest>,
    A1: Parser<Dest = Dest>,
    A2: Parser<Dest = Dest>,
    A3: Parser<Dest = Dest>,
    A4: Parser<Dest = Dest>,
    A5: Parser<Dest = Dest>,
    A6: Parser<Dest = Dest>,
    A7: Parser<Dest = Dest>,
>(
    a0: A0,
    a1: A1,
    a2: A2,
    a3: A3,
    a4: A4,
    a5: A5,
    a6: A6,
    a7: A7,
) -> Alt<(A0, A1, A2, A3, A4, A5, A6, A7)> {
    Alt {
        parser: (a0, a1, a2, a3, a4, a5, a6, a7),
    }
}

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
}
