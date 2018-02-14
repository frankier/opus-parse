use std;
use std::time::Duration;
use itertools::Itertools;

//mod strict {
    #[derive(Debug, error_chain)]
    pub enum ErrorKind {
        Msg(String),

        #[error_chain(custom)]
        #[error_chain(description = r#"|_| "Can't parse duration, wrong number of colons (expected 2)""#)]
        WrongNumberOfColons(()),

        #[error_chain(custom)]
        #[error_chain(description = r#"|_| "Can't parse duration, wrong number of commas (expected 1)""#)]
        WrongNumberOfCommas(()),

        #[error_chain(foreign)]
        ParseIntError(std::num::ParseIntError),
    }

    pub fn parse_duration(in_dur: &str) -> Result<Duration> {
        // Parse duration like 00:01:31,950
        let bits = in_dur.split(':').collect_vec();
        if bits.len() != 3 {
            bail!(ErrorKind::WrongNumberOfColons(()));
        }
        let sec_bits = bits[2].split(',').collect_vec();
        if sec_bits.len() != 2 {
            bail!(ErrorKind::WrongNumberOfCommas(()));
        }
        let secs = bits[0].parse::<u64>()? * 60 * 60 +
                   bits[1].parse::<u64>()? * 60 + 
                   sec_bits[0].parse::<u64>()?;
        let millis = sec_bits[1].parse::<u64>()?;
        Ok(Duration::new(secs, (millis * 1_000_000) as u32))
    }
//}

/*

mod lax {
    #[derive(Debug, error_chain)]
    pub enum ErrorKind {
        Msg(String),

        #[error_chain(custom)]
        #[error_chain(description = r#"|_| "Can't parse duration, wrong number of fields (expected 3 or 4)""#)]
        WrongNumberOfFields(u64),

        #[error_chain(custom)]
        #[error_chain(description = r#"|_| "Can't parse duration, wrong number of commas (expected 1)""#)]
        WrongNumberOfCommas(()),

        #[error_chain(foreign)]
        ParseIntError(std::num::ParseIntError),
    }

    pub fn parse_int(num: &str) -> IntParseResult {
        num.parse::<u64>()?
    }

    pub fn parse_duration(in_dur: &str) -> Result<Duration> {
        // Parse broken durations like 0 0:06:29 and 01:38:07.100
        let bits = in_dur.split(&[':', ',', '.']).collect_vec();
        if bits.len() != 3 && bits.len() != 4 {
            bail!(ErrorKind::WrongNumberOfFields(bits.len()));
        }
        let secs = parse_int(bits[0])? * 60 * 60 +
                   bits[1].parse::<u64>()? * 60 + 
                   sec_bits[0].parse::<u64>()?;
        let millis = sec_bits[1].parse::<u64>()?;
        Ok(Duration::new(secs, (millis * 1_000_000) as u32))
    }
}
*/
