extern crate xml;
extern crate flate2;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate derive_error_chain;
extern crate itertools;

use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;
use std::collections::BTreeMap;
use std::mem;
use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;
use flate2::read::GzDecoder;
use std::path::Path;
mod duration;
mod time_id;
use duration::parse_duration;
use time_id::parse_time_id;


/*
enum GroupBetweenError {
    ExpectedOpen,
    UnclosedGroup,
}

pub trait IterExt : Iterator {
    fn group_between(self, start_pred, end_pred) {
        let mut open_group = false;
        self.batching(|mut it| {
            if (open_group) && it.next().is_none() {
                Some(Err(UnclosedGroup))
            }
            match it.next() {
                None => None,
                Some(elem) => {
                    if start_pred(elem) {
                        *open_group = true;
                        Some(elem, it.take_while(|x| !end_pred(x)))
                    } else {
                        Some(Err(ExpectedOpen))
                    }
                }
            }
        })
    }
}
*/

type MetaMap = BTreeMap<(String, String), String>;
type GzFileRead = GzDecoder<BufReader<File>>;

/// A word/token.
#[derive(Debug)]
pub struct Word {
    pub id: u64,
    pub word: String,
}

/// Whether a sentence or block delimiter is at the start or end of the sentence or block.
#[derive(Debug)]
pub enum DelimType {
    Start,
    End
}

/// A sentence delimiter.
#[derive(Debug)]
pub struct SentDelim {
    pub id: u64,
    pub delim_type: DelimType
}

/// A block delimiter.
#[derive(Debug)]
pub struct BlockDelim {
    pub id: u64,
    pub offset: Duration,
    pub delim_type: DelimType
}

/// An event from main part of the stream.
#[derive(Debug)]
pub enum StreamBit {
    SentDelim(SentDelim),
    BlockDelim(BlockDelim),
    Word(Word),
}

/// An event from the whole file including metadata.
#[derive(Debug)]
pub enum FlatStreamBit {
    Meta(MetaMap),
    StreamBit(StreamBit),
    EndStream,
}

/*

struct PreindexReader<'a>(&'a mut File);

impl<'a> Iterator for PreindexReader<'a> {
    type Item = Result<(String, u64, u64), PreindexReaderError>;

    fn next(&mut self) -> Option<Result<(String, u64, u64), PreindexReaderError>> {
        fn read_record(mut f: &File, token_len: u64) -> Result<(String, u64, u64), PreindexReaderError> {
            let mut buf = vec![0; token_len as usize];
            f.read_exact(buf.as_mut_slice())?;
            return Ok((String::from_utf8(buf)?,
                       f.read_u64::<BigEndian>()?,
                       f.read_u64::<BigEndian>()?));
        }

        match self.0.read_u64::<BigEndian>() {
            Ok(token_len) => Some(read_record(self.0, token_len)),
            Err(err) => {
                if err.kind() == io::ErrorKind::UnexpectedEof {
                    None
                } else {
                    Some(Err(PreindexReaderError::from(err)))
                }
            }
        }
    }
}
*/

// XML helpers

fn get_value<'a>(attrs: &'a Vec<OwnedAttribute>, name: &str) -> Option<&'a String> {
    attrs.iter().find(|e| e.name.local_name.as_str() == name)
                .map(|e| &e.value)
}

fn req_value<'a>(attrs: &'a Vec<OwnedAttribute>, name: &str) -> Result<&'a String> {
    get_value(attrs, name).ok_or_else(|| ErrorKind::ExpectedAttribute(name.to_owned()).into())
}

// Open subtitles iterator

/*
#[derive(Debug)]
enum OpusParseErrorType {
    ParseIntError(std::num::ParseIntError),
    XmlParseError(xml::reader::Error),
    ExceptedAttribute(String),
}

#[derive(Debug)]
struct OpusParseError {
    position: TextPosition,
    err: OpusParseErrorType,
}
*/

#[derive(Debug, error_chain)]
pub enum ErrorKind {
    Msg(String),

    #[error_chain(link="duration::Error")]
    DurationParseErr(duration::ErrorKind),

    #[error_chain(link="time_id::Error")]
    TimeIdParseErr(time_id::ErrorKind),

    #[error_chain(foreign)]
    ParseIntError(std::num::ParseIntError),
    #[error_chain(foreign)]
    XmlParseError(xml::reader::Error),

    #[error_chain(custom)]
    #[error_chain(description = r#"|_| "Expected attribute""#)]
    #[error_chain(display = r#"|t| write!(f, "expected attribute: '{}'", t)"#)]
    ExpectedAttribute(String),
}

pub struct OpusStream<T> where T: Read {
    pub er: EventReader<T>,
    pub word_id: Option<u64>,
    pub sent_id: u64,
    pub in_meta: bool,
    pub meta_cat: Option<String>,
    pub meta_attr: Option<String>,
    pub meta: MetaMap,
}

impl OpusStream<GzFileRead> {
    pub fn from_path<P: AsRef<Path>>(path: P)
            -> std::io::Result<OpusStream<GzFileRead>> {
        let subf = File::open(path)?;
        let subf_buf = BufReader::new(subf);
        let subf_dec = GzDecoder::new(subf_buf)?;
        Ok(OpusStream::new(subf_dec))
    }
}

fn both<A, B>(a: Option<A>, b: Option<B>) -> Option<(A, B)> {
    a.and_then(|a| b.map(|b| (a, b)))
}


impl<T: Read> OpusStream<T> {
    pub fn new(subtitle_stream: T) -> OpusStream<T> {
        let parser = EventReader::new(subtitle_stream);
        OpusStream {
            er: parser,
            sent_id: 0,
            word_id: None,
            in_meta: false,
            meta_cat: None,
            meta_attr: None,
            meta: BTreeMap::new(),
        }
    }

    pub fn next(&mut self) -> Result<FlatStreamBit> {
        loop {
            let ev = self.er.next();
            match ev? {
                XmlEvent::StartElement { name, attributes , .. } => {
                    match name.local_name.as_str() {
                        "meta" => {
                            self.in_meta = true;
                        }
                        "s" => {
                            self.sent_id = req_value(&attributes, "id")?.parse::<u64>()?;
                            return Ok(
                                FlatStreamBit::StreamBit(
                                    StreamBit::SentDelim(
                                        SentDelim {
                                            id: self.sent_id,
                                            delim_type: DelimType::Start
                                        })));
                        }
                        "time" => {
                            let full_id = req_value(&attributes, "id")?;
                            let (delim_type, num_id) = parse_time_id(full_id.as_str())?;
                            let offset = parse_duration(req_value(&attributes, "value")?.as_str())?;
                            return Ok(
                                FlatStreamBit::StreamBit(
                                    StreamBit::BlockDelim(
                                        BlockDelim {
                                            id: num_id,
                                            offset: offset,
                                            delim_type: delim_type,
                                        })));
                        }
                        "w" => {
                            let dot_word_id = req_value(&attributes, "id")?;
                            let end_word_id = dot_word_id.split('.').next_back().unwrap();
                            self.word_id = Some(end_word_id.parse::<u64>()?);
                        }
                        tag_name => {
                            if self.in_meta {
                                if self.meta_cat.is_some() {
                                    self.meta_attr = Some(tag_name.to_owned())
                                } else {
                                    self.meta_cat = Some(tag_name.to_owned())
                                }
                            }
                            // pass on unknown tag currently
                        }
                    }
                }
                XmlEvent::EndElement { name } => {
                    match name.local_name.as_str() {
                        "s" => {
                            return Ok(
                                FlatStreamBit::StreamBit(
                                    StreamBit::SentDelim(
                                        SentDelim {
                                            id: self.sent_id,
                                            delim_type: DelimType::End
                                        })));
                        }
                        "w" => {
                            self.word_id = None;
                        }
                        "meta" => {
                            let meta = mem::replace(&mut self.meta, BTreeMap::new());
                            return Ok(FlatStreamBit::Meta(meta));
                        }
                        tag_name => {
                            if self.in_meta {
                                if self.meta_attr.as_ref().map(|s| s.as_str() == tag_name).unwrap_or(false) {
                                    self.meta_attr = None
                                } else if self.meta_cat.as_ref().map(|s| s.as_str() == tag_name).unwrap_or(false) {
                                    self.meta_cat = None
                                }
                            }
                            // pass on unknown tag currently
                        }
                    }
                }
                XmlEvent::Characters(chars) => {
                    if self.in_meta {
                        if let Some((attr, cat)) = both(self.meta_cat.as_ref(), self.meta_attr.as_ref()) {
                            // XXX: Might not strictly need to copy cat here
                            self.meta.insert((attr.to_owned(), cat.to_owned()), chars);
                        }
                    } else if let Some(word_id) = self.word_id {
                        return Ok(
                            FlatStreamBit::StreamBit(
                                StreamBit::Word(
                                    Word { id: word_id, word: chars })));
                    }
                }
                XmlEvent::EndDocument => {
                    return Ok(FlatStreamBit::EndStream);
                }
                _ => {}
            }
        }
    }
}

//fn parse(subtitle_stream: &Read) -> Iterator<DocumentBit> {
//}
