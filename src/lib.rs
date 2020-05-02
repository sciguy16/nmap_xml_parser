//!Parse Nmap XML output into Rust.
//!
//!The root of this crate is
//![`NmapResults::parse()`](struct.NmapResults.html#method.parse). Its use
//!should be similar to the following:
//!
//!```
//!# use std::path::PathBuf;
//!# use std::fs;
//!use nmap_xml_parser::NmapResults;
//!# let mut nmap_xml_file = PathBuf::new();
//!# nmap_xml_file.push(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
//!# nmap_xml_file.push("tests/test.xml");
//!let content = fs::read_to_string(nmap_xml_file).unwrap();
//!let results = NmapResults::parse(&content).unwrap();
//!```
//!
//!This crate is still a work-in-progress and does not represent the full
//!Nmap output structure. However, it _should_ successfully parse any Nmap XML
//!output. Please file a bug report if it fails.
//!
//!The API is __not stable__ and is subject to breaking changes until the
//!crate reaches 1.0. Use with care.
use roxmltree::{Document, Node};
use strum;

pub mod host;
pub mod port;

use crate::host::Host;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error parsing file as XML document")]
    XmlError(#[from] roxmltree::Error),
    #[error("file is not an Nmap XML output")]
    InvalidNmapOutput,
}

///Root structure of a Nmap scan result.
pub struct NmapResults {
    ///List of hosts in the Nmap scan.
    pub hosts: Vec<Host>,

    ///Start time of the Nmap scan as seconds since Unix epoch.
    pub scan_start_time: i64,

    ///End time of the Nmap scan as seconds since Unix epoch.
    pub scan_end_time: i64,
}

impl NmapResults {
    pub fn parse(xml: &str) -> Result<Self, Error> {
        let doc = Document::parse(&xml)?;
        let root_element = doc.root_element();
        if root_element.tag_name().name() != "nmaprun" {
            return Err(Error::InvalidNmapOutput);
        }

        let scan_start_time = root_element
            .attribute("start")
            .ok_or(Error::InvalidNmapOutput)
            .and_then(|s| s.parse::<i64>().or(Err(Error::InvalidNmapOutput)))?;

        let mut hosts: Vec<Host> = Vec::new();
        let mut scan_end_time = None;

        for child in root_element.children() {
            match child.tag_name().name() {
                "host" => {
                    hosts.push(Host::parse(child)?);
                }
                "runstats" => scan_end_time = Some(parse_runstats(child)?),
                _ => {}
            }
        }

        let scan_end_time = scan_end_time.ok_or(Error::InvalidNmapOutput)?;

        Ok(NmapResults {
            hosts,
            scan_start_time,
            scan_end_time,
        })
    }
}

fn parse_runstats(node: Node) -> Result<i64, Error> {
    for child in node.children() {
        if child.tag_name().name() == "finished" {
            let finished = child
                .attribute("time")
                .ok_or(Error::InvalidNmapOutput)
                .and_then(|s| s.parse::<i64>().or(Err(Error::InvalidNmapOutput)))?;
            return Ok(finished);
        }
    }

    Err(Error::InvalidNmapOutput)
}