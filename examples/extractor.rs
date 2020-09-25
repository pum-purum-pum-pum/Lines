use glam::{vec2, vec3, Vec2};
use std::fs::File;
use std::io::BufReader;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() -> Result<()> {
    let target_point = vec2(4135614.94, 7705216.63); // Russia, Dubna :)
    let f = File::open("map.txt").unwrap();
    let f = BufReader::new(f);
    use std::io::prelude::*;
    let mut point_sum = vec2(0., 0.);
    let mut point_cnt = 0;
    let mut res = vec![];
    let mut cnt = 0;
    for line in f.lines() {
        let mut points = vec![];
        let line = line.unwrap();
        let numbers: Vec<_> = line.split(" ").collect();
        for i in 0..numbers.len() / 2 {
            let point = vec2(
                numbers[i * 2].parse().unwrap(),
                numbers[i * 2 + 1].parse().unwrap(),
            );
            cnt += 1;
            if cnt % 10 == 0 {
                points.push(point);
                point_sum += point;
            }
        }
        if points.len() > 0 {
            res.push(points);
        }
    }
    let mut file = File::create("Dubna.txt")?;
    for i in res.iter() {
        let line: Vec<_> = i.iter().map(|p| format!("{} {}", p.x(), p.y())).collect();
        let line = line.join(" ") + "\n";
        file.write_all(line.as_bytes())?;
    }
    Ok(())
}
