// Пусть есть логи:
// System(requestid):
// - trace
// - error
// App(requestid):
// - trace
// - error
// - journal (человекочитаемая сводка)

// Есть прототип штуки, которая умеет:
// - парсить логи
// - фильтровать
//  -- по requestid
//  -- по ошибкам
//  -- по изменению счёта (купить/продать)

// Модель данных:
// - Пользователь (userid, имя)
// - Вещи
//  -- Предмет (assetid, название)
//  -- Набор (assetid, количество)
//      comment{-- Собственность (assetid, userid владельца, количество)}
//  -- Таблица предложения (assetid на assetid, userid продавца)
//  -- Таблица спроса (assetid на assetid, userid покупателя)
// - Операция App
//  -- Journal
//   --- Создать пользователя userid с уставным капиталом от 10usd и выше
//   --- Удалить пользователя
//   --- Зарегистрировать assetid с ликвидностью от 50usd
//   --- Удалить assetid (весь asset должен принадлежать пользователю)
//   --- Внести usd для userid (usd (aka доллар сша) - это тип asset)
//   --- Вывести usd для userid
//   --- Купить asset
//   --- Продать asset
//  -- Trace
//   --- Соединить с биржей
//   --- Получить данные с биржи
//   --- Локальная проверка корректности (упреждение ошибок в ответе)
//   --- Отправить запрос в биржу
//   --- Получить ответ от биржи
//  -- Error
//   --- нет asset
//   --- системная ошибка
// - Операция System
//  -- Trace
//   --- Отправить запрос
//   --- Получить ответ
//  -- Error
//   --- нет сети
//   --- отказано в доступе
use std::num::NonZeroU32;

use clap::Parser;

/// CLI mode for filtering log entries, mirrors `analysis::ReadMode`.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Mode {
    /// Return all log entries.
    All,
    /// Return only error entries.
    Errors,
    /// Return only exchange/journal operation entries.
    Exchanges,
}

impl From<Mode> for analysis::ReadMode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::All => Self::All,
            Mode::Errors => Self::Errors,
            Mode::Exchanges => Self::Exchanges,
        }
    }
}

/// Wrapper for comma-separated request IDs, used for clap parsing.
#[derive(Clone, Debug)]
struct RequestIds(Vec<NonZeroU32>);

impl std::str::FromStr for RequestIds {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(|part| {
                let trimmed = part.trim();
                let n: u32 = trimmed
                    .parse()
                    .map_err(|e| format!("invalid request id '{trimmed}': {e}"))?;
                NonZeroU32::new(n)
                    .ok_or_else(|| format!("request id must be non-zero, got '{trimmed}'"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(RequestIds)
    }
}

/// Log analysis tool for a trading/exchange application.
#[derive(Parser)]
#[command(name = "cli", version, about)]
struct Cli {
    /// Log file to analyze.
    filename: String,

    /// Filtering mode: all, errors, or exchanges.
    #[arg(long, value_enum, default_value_t = Mode::All)]
    mode: Mode,

    /// Comma-separated request IDs to filter by (e.g. 1,2,3).
    #[arg(long)]
    request_id: Option<RequestIds>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let read_mode: analysis::ReadMode = cli.mode.into();
    let request_ids = cli.request_id.map(|ids| ids.0).unwrap_or_default();

    println!(
        "Trying opening file '{}' from directory '{}'",
        cli.filename,
        std::env::current_dir()?.to_string_lossy()
    );
    let file = std::fs::File::open(&cli.filename)
        .map_err(|e| anyhow::anyhow!("Failed to open '{}': {}", cli.filename, e))?;
    let logs = analysis::read_log(file, read_mode, request_ids)?;
    println!("got logs:");
    logs.iter().for_each(|parsed| println!("  {}", parsed));
    Ok(())
}
