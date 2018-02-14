use std;
use super::DelimType;

#[derive(Debug, error_chain)]
pub enum ErrorKind {
    Msg(String),

    #[error_chain(custom)]
    #[error_chain(description = r#"|_| "Can't parse time id, expected at least 3 characters""#)]
    NotEnoughCharacters(()),

    #[error_chain(custom)]
    #[error_chain(description = r#"|_| "Can't parse time id, expected to begin with T""#)]
    ExpectedBeginWithT(()),

    #[error_chain(custom)]
    #[error_chain(description = r#"|_| "Can't parse time id, expected to end with S or E""#)]
    ExpectedEndWithSOrE(()),

    #[error_chain(foreign)]
    ParseIntError(std::num::ParseIntError),
}

pub fn parse_time_id(time_id: &str) -> Result<(DelimType, u64)> {
    // Parse duration like T12S
    if time_id.len() < 3 {
        bail!(ErrorKind::NotEnoughCharacters(()));
    }
    let (time_marker, rest) = time_id.split_at(1);
    if time_marker != "T" {
        bail!(ErrorKind::ExpectedBeginWithT(()));
    }
    let (str_id, start_end_marker) = rest.split_at(rest.len() - 1);
    let delim_type = match start_end_marker {
        "S" => DelimType::Start,
        "E" => DelimType::End,
        _ => bail!(ErrorKind::ExpectedBeginWithT(()))
    };
    let num_id = str_id.parse::<u64>()?;
    Ok((delim_type, num_id))
}

