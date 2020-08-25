use std::fs::File;
use std::io::{BufReader, Write};
use wkt::{self, Geometry};
use nanoserde::{DeBin, SerBin};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const WAY_COL_NUM: usize = 68;

fn main() -> Result<()> {
    let mut max_x = 1.;
    let mut max_y = 1.;
    let mut res = vec![];
    let file = File::open("planet_osm_line_202008251533_copy.csv")?;
    let buf_reader = BufReader::new(file);
    let mut rdr = csv::Reader::from_reader(buf_reader);
    let records = rdr.records() ;
    for result in records {
        let mut res_linestring = vec![];
        let record = result?;
        let wkt_way = record.get(WAY_COL_NUM).unwrap();
        let parsed: wkt::Wkt<f64> = wkt::Wkt::from_str(wkt_way).unwrap();
        for g in parsed.items {
            if let Geometry::LineString(linestring) = g {
                for c in linestring.0 {
                    res_linestring.push((c.x, c.y));
                    max_x = if max_x < c.x {c.x} else {max_x};
                    max_y = if max_y < c.y {c.y} else {max_y};
                }
            }
        }
        res.push(res_linestring);
    }
    for j in res.iter_mut() {
        for i in j.iter_mut() {
            *i = (1. * i.0 / max_x, 1. * i.1 / max_y)
        }
    }
    // dbg!(res);
    let bytes = SerBin::serialize_bin(&res);

    let mut buffer = File::create("map.bin")?;

    // Writes some prefix of the byte string, not necessarily all of it.
    buffer.write(&bytes)?;
    Ok(())
}