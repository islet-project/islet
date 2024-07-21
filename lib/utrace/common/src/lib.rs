use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

pub mod config;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub enum UnsafeKind {
    Function,
    Block,
    Trait,
    Impl,
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub struct UnsafeItem {
    pub kind: UnsafeKind,
    pub name: String,
}

impl UnsafeItem {
    pub fn new(kind: UnsafeKind, name: String) -> Self {
        Self { kind, name }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub krate: String,
    pub items: Vec<UnsafeItem>,
}

impl Record {
    pub fn new(krate: String) -> Self {
        Self {
            krate,
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, kind: UnsafeKind, name: String) {
        self.items
            .push(UnsafeItem::new(kind, format!("{}{}", self.krate, name)));
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        let path = format!("{}/{}.record", path, self.krate);
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer(writer, self)?)
    }

    pub fn load(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Records(Vec<Record>);

impl Records {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, record: Record) {
        self.0.push(record);
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer(writer, self)?)
    }

    pub fn load(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

impl<'a> IntoIterator for &'a Records {
    type Item = <&'a Vec<Record> as IntoIterator>::Item;
    type IntoIter = <&'a Vec<Record> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}
