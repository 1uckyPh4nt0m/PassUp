extern crate pwsafe;

use pwsafe::{PwsafeReader, PwsafeWriter, PwsafeRecordField};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use crate::utils::{DB, DBEntry, Uuid};

pub fn test() {
    let filename = "tests/resources/pwsafe.psafe3";
    let file = match File::open(filename) {
        Ok(file) => BufReader::new(file),
        Err(e) => panic!("{}", e)
    };
    let mut psdb = PwsafeReader::new(file, b"password").unwrap();
    let mut entry = DBEntry::empty();
    let mut entry_vec = Vec::new();
    let mut record_vec = Vec::new();

    let version = psdb.read_version().unwrap();

    let mut skipped_version_field = false;
    while let Some((field_type, field_data)) = psdb.read_field().unwrap() {
        if !skipped_version_field {
            if field_type == 0xff {
                skipped_version_field = true;
            }
            continue;
        }
        //println!("Read field of type {} and length {}", field_type, field_data.len());
        let record = match pwsafe::PwsafeRecordField::new(field_type, field_data.clone()) {
            Ok(r) => r,
            Err(e) => { println!("{}", e); continue }
        };
        record_vec.push((field_type, field_data));
        match &record {
            PwsafeRecordField::Url(url) => entry.url_ = url.to_owned(),
            PwsafeRecordField::Username(username) => entry.username_ = username.to_owned(),
            PwsafeRecordField::Password(password) => entry.old_password_ = password.to_owned(),
            PwsafeRecordField::Uuid(uuid) => entry.uuid_ = Uuid::Pwsafe(uuid.to_owned()),
            pwsafe::PwsafeRecordField::EndOfRecord => entry = DBEntry::empty(),
            _ => entry = DBEntry::empty()
        };

        let entry_ = entry.clone();
        if entry_.url_.ne("") && entry_.username_.ne("") && entry_.old_password_.ne("") {
            entry_vec.push(entry_);
        }
    }
    psdb.verify().unwrap();

    let db = DB::new(entry_vec);
    println!("{:?}", db);

    //------------------------------------------------
    // do stuff with db
    //------------------------------------------------

    let filename_copy = format!("{}_copy", filename);
    match fs::rename(filename, filename_copy) {
        Ok(_) => (),
        Err(e) => panic!("{}", e)
    };
    let file = BufWriter::new(File::create(filename).unwrap());
    let mut db = PwsafeWriter::new(file, 2048, b"password").unwrap();
    let empty = [0u8, 0];
    //let version = [0x0eu8, 0x03u8];
    //db.write_field(0x00, &version).unwrap(); // Version field
    db.write_field(0x00, &[(version - (version >> 8) << 8) as u8, (version >> 8) as u8]).unwrap(); // Version field
    db.write_field(0xff, &empty).unwrap(); // End of header
    
    for (record_type, record_data) in record_vec {
        db.write_field(record_type, &record_data).unwrap();
    }

    db.finish().unwrap(); // EOF and HMAC
}