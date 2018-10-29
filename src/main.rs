extern crate csv;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;
extern crate icalendar;
extern crate chrono;
extern crate failure;

use std::fs;
use chrono::{NaiveDate, NaiveTime, DateTime, Duration, Utc};
use std::collections::HashMap;
use csv::ReaderBuilder;
use icalendar::{Event, Calendar, Component};

#[derive(Deserialize)]
pub struct Config {
    pub periodfile: String,
    pub lessonfile: String,
    pub calfile: String,
    pub startdate: NaiveDate,
    pub enddate: NaiveDate,
    pub rrule_freq: String,
    pub rrule_interval: String
}

#[derive(Deserialize)]
pub struct PeriodLine {
    pub period_num: u32,
    pub start_time: String,
    pub end_time: String
}

#[derive(Deserialize)]
pub struct LessonLine {
    pub day_num: i64,
    pub period_num: u32,
    pub lesson_name: String,
    pub location: String,
    pub desc: Option<String>
}
fn main() -> Result<(), failure::Error> {
    println!("[+] eta's timetable icaler");
    println!("[+] Reading config.toml");
    let cdata = fs::read_to_string("config.toml")?;
    let cfg: Config = toml::from_str(&cdata)?;
    println!("[+] Reading periods from {}", cfg.periodfile);
    let mut periods = HashMap::new();
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_path(&cfg.periodfile)?;
    for result in rdr.deserialize() {
        let rec: PeriodLine = result?;
        let st = NaiveTime::parse_from_str(&rec.start_time, "%H:%M")?;
        let et = NaiveTime::parse_from_str(&rec.end_time, "%H:%M")?;
        periods.insert(rec.period_num, (st, et));
    }
    println!("[*] {} periods configured", periods.len());
    println!("[+] Reading lessons from {}", cfg.lessonfile);
    let date_until = DateTime::<Utc>::from_utc(cfg.enddate.and_time(NaiveTime::from_hms(0, 0, 0)), Utc);
    let mut done = 0;
    let mut cal = Calendar::new();
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_path(&cfg.lessonfile)?;
    for result in rdr.deserialize() {
        let rec: LessonLine = result?;
        let &(start_time, end_time) = periods.get(&rec.period_num).unwrap();
        let dt_start = DateTime::<Utc>::from_utc(cfg.startdate.and_time(start_time), Utc) + Duration::days(rec.day_num);
        let dt_end = DateTime::<Utc>::from_utc(cfg.startdate.and_time(end_time), Utc) + Duration::days(rec.day_num);
        let rrule = format!("FREQ={};INTERVAL={};UNTIL={}", cfg.rrule_freq, cfg.rrule_interval, date_until.format("%Y%m%dT%H%M%S"));
        let event = Event::new()
            .summary(&format!("P{}: {}", rec.period_num, rec.lesson_name))
            .description(&rec.desc.unwrap_or("".into()))
            .location(&rec.location)
            .starts(dt_start)
            .ends(dt_end)
            .add_property("RRULE", &rrule)
            .done();
        cal.push(event);
        done += 1;
    }
    println!("[*] {} lessons processed", done);
    println!("[+] Writing calendar to {}", cfg.calfile);
    let cal = cal.to_string();
    fs::write(&cfg.calfile, &cal)?;
    println!("[+] Done!");
    Ok(())
}
