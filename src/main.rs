//! Read one or more FIT files and dump their contents as JSON TPV 'focus.json'
use fitparser::de::{from_reader_with_options, DecodeOption};
use fitparser::profile::MesgNum;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::{io, thread, time};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, Serialize, Clone)]
#[allow(dead_code, non_snake_case)]
pub struct Focus {
    name: String,
    country: String,
    team: String,
    teamCode: String,
    power: u32,
    avgPower: u32,
    nrmPower: u32,
    maxPower: u32,
    cadence: u32,
    avgCadence: u32,
    maxCadence: u32,
    heartrate: u32,
    avgHeartrate: u32,
    maxHeartrate: u32,
    time: u32,
    distance: u32,
    height: u32,
    speed: u32,
    tss: u32,
    calories: u32,
    draft: u32,
    windSpeed: u32,
    windAngle: u32,
    slope: i32,
    eventLapsTotal: u32,
    eventLapsDone: i32,
    eventDistanceTotal: u32,
    eventDistanceDone: u32,
    eventDistanceToNextLocation: u32,
    eventNextLocation: u32,
    eventPosition: u32,
}

impl Focus {
    pub(crate) fn new() -> Focus {
        Focus {
            name: String::from("--"),
            country:  String::from("--"),
            team: String::from("--"),
            teamCode: String::from("--"),
            power: 0,
            avgPower: 0,
            nrmPower: 0,
            maxPower: 0,
            cadence: 0,
            avgCadence: 0,
            maxCadence: 0,
            heartrate: 0,
            avgHeartrate: 0,
            maxHeartrate: 0,
            time: 0,
            distance: 0,
            height: 0,
            speed: 0,
            tss: 0,
            calories: 0,
            draft: 0,
            windSpeed: 0,
            windAngle: 0,
            slope: 0,
            eventLapsTotal: 0,
            eventLapsDone: 0,
            eventDistanceTotal: 0,
            eventDistanceDone: 0,
            eventDistanceToNextLocation: 0,
            eventNextLocation: 0,
            eventPosition: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct ValueU32 {
    value: u32,
    units: String,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
struct ValueF32 {
    value: f32,
    units: String,
}

/// Read FIT formatted files and output each waypoint as TPV 'focus.json' file
#[derive(Debug, StructOpt)]
#[structopt(name = "tpvfitplay")]
struct Cli {
    /// FIT files to read and play back as TPV 'focus.json'
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,

    /// Output file location, if not provided the JSON file will be named 'focus.json'
    #[structopt(short, long, parse(from_os_str), default_value = "focus.json")]
    output: PathBuf,

    /// Delay between updates of 'focus.json' in msec.
    #[structopt(short, long, default_value = "250")]
    delay: u64,

    /// Drop fields and messages that aren't defined in the profile
    #[structopt(long)]
    drop_unknown: bool,

    /// Return all enum values with their numeric value instead of the string variant name
    #[structopt(long)]
    numeric_enums: bool,

    /// Keep generic subfield names in the output instead of the specific resolved name
    #[structopt(long)]
    keep_generic_names: bool,

    /// Keep composite fields that are expanded into 1 or more component fields
    #[structopt(long)]
    keep_composite_fields: bool,

    /// Skip checking the header and data section CRC values
    #[structopt(long)]
    no_crc_check: bool,
}

/// Alternate serialization format
#[derive(Clone, Debug, Serialize)]
struct FitDataMap {
    kind: fitparser::profile::MesgNum,
    fields: BTreeMap<String, fitparser::ValueWithUnits>,
}

impl FitDataMap {
    fn new(record: fitparser::FitDataRecord) -> Self {
        FitDataMap {
            kind: record.kind(),
            fields: record
                .into_vec()
                .into_iter()
                .map(|f| (f.name().to_owned(), fitparser::ValueWithUnits::from(f)))
                .collect(),
        }
    }
}

fn write_json_file_focus(
    filename: &Path,
    data: Vec<fitparser::FitDataRecord>, delay: u64) -> Result<(), Box<dyn Error>> {
    let data: Vec<FitDataMap> = data.into_iter().map(FitDataMap::new).collect();

    let mut ts: u32 = 0;

    for fdm in data {
        if fdm.kind == MesgNum::Record {
            let mut focus = Focus::new();

            focus.time = ts;
            ts += 1;

            for field in fdm.fields {
                // println!("{} = {}", field.0, field.1);
                let tmp = serde_json::to_string(&field.1)?;
                if field.0 == "power" {
                    let value_u32: ValueU32 = serde_json::from_str(&tmp)?;
                    focus.power = value_u32.value;
                } else if field.0 == "heart_rate" {
                    let value_u32: ValueU32 = serde_json::from_str(&tmp)?;
                    focus.heartrate = value_u32.value;
                } else if field.0 == "cadence" {
                    let value_u32: ValueU32 = serde_json::from_str(&tmp)?;
                    focus.cadence = value_u32.value;
                } else if field.0 == "distance" {
                    let value_f32: ValueF32 = serde_json::from_str(&tmp)?;
                    focus.distance = value_f32.value as u32;
                } else if field.0 == "enhanced_speed" {
                    let value_f32: ValueF32 = serde_json::from_str(&tmp)?;
                    focus.speed = (value_f32.value * 3.6 * 275.0) as u32;
                } else if field.0 == "grade" {
                    let value_f32: ValueF32 = serde_json::from_str(&tmp)?;
                    focus.slope = value_f32.value as i32;
                } else if field.0 == "enhanced_altitude" {
                    let value_f32: ValueF32 = serde_json::from_str(&tmp)?;
                    focus.height = 450 + value_f32.value as u32;
                }
            }
            
            let focus_list = vec![focus];
            let json = serde_json::to_string(&focus_list)?;
            // print!("{focus_list:#?}");
            print!("- processing time-stamp: {:5}", ts);

            // let mut fp = File::create("/home/stefan/devel/tpvbc2http/http/testing/focus.json")?;
            let mut fp = File::create(filename)?;
            fp.write_all(json.as_bytes())?;

            thread::sleep(time::Duration::from_millis(delay));
            println!("\x1b[5D\x1b[1A");
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let opt = Cli::from_args();

    // set any decode options
    let mut decode_opts = HashSet::new();
    if opt.drop_unknown {
        decode_opts.insert(DecodeOption::DropUnknownFields);
        decode_opts.insert(DecodeOption::DropUnknownMessages);
    }
    if opt.keep_generic_names {
        decode_opts.insert(DecodeOption::UseGenericSubFieldName);
    }
    if opt.keep_composite_fields {
        decode_opts.insert(DecodeOption::KeepCompositeFields);
    }
    if opt.numeric_enums {
        decode_opts.insert(DecodeOption::ReturnNumericEnumValues);
    }
    if opt.no_crc_check {
        decode_opts.insert(DecodeOption::SkipHeaderCrcValidation);
        decode_opts.insert(DecodeOption::SkipDataCrcValidation);
    }

    // define parsed and serialized data output location
    let output_loc = opt.output.as_path();

    // read from STDIN if no files were given
    if opt.files.is_empty() {
        println!("Reading from: stdin");
        println!("Writing   to: {:?}", output_loc);

        let mut stdin = io::stdin();
        let data = from_reader_with_options(&mut stdin, &decode_opts)?;
        write_json_file_focus(output_loc, data, opt.delay)?;
        return Ok(());
    }

    // Read each FIT file and output it
    for file in opt.files {
        // open file and parse data
        println!("Reading from: {:?}", file);
        println!("Writing   to: {:?}", output_loc);

        let mut fp = File::open(&file)?;
        let data = from_reader_with_options(&mut fp, &decode_opts)?;
        write_json_file_focus(output_loc, data, opt.delay)?;
        println!("");
    }

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}