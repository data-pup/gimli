use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use gimli::read;
use gimli::write::{self, Address, EndianVec};
use gimli::LittleEndian;

fn read_section(section: &str) -> Vec<u8> {
    let mut path = PathBuf::new();
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        path.push(dir);
    }
    path.push("fixtures/self");
    path.push(section);

    println!("Reading section \"{}\" at path {:?}", section, path);
    assert!(path.is_file());
    let mut file = File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    buf
}

#[test]
fn test_convert_debug_info() {
    // Convert existing sections
    let debug_abbrev = read_section("debug_abbrev");
    let debug_abbrev = read::DebugAbbrev::new(&debug_abbrev, LittleEndian);

    let debug_info = read_section("debug_info");
    let debug_info = read::DebugInfo::new(&debug_info, LittleEndian);

    let debug_line = read_section("debug_line");
    let debug_line = read::DebugLine::new(&debug_line, LittleEndian);

    let debug_str = read_section("debug_str");
    let debug_str = read::DebugStr::new(&debug_str, LittleEndian);

    let debug_ranges = read_section("debug_ranges");
    let debug_ranges = read::DebugRanges::new(&debug_ranges, LittleEndian);

    let debug_rnglists = read::DebugRngLists::new(&[], LittleEndian);

    let ranges = gimli::RangeLists::new(debug_ranges, debug_rnglists);

    let dwarf = read::Dwarf {
        debug_abbrev,
        debug_info,
        debug_line,
        debug_str,
        ranges,
        ..Default::default()
    };

    let mut dwarf = write::Dwarf::from(&dwarf, &|address| Some(Address::Absolute(address)))
        .expect("Should convert DWARF information");

    assert_eq!(dwarf.units.count(), 23);
    let entries: usize = (0..dwarf.units.count())
        .map(|i| dwarf.units.get(dwarf.units.id(i)).count())
        .sum();
    assert_eq!(entries, 29_560);
    assert_eq!(dwarf.line_strings.count(), 0);
    assert_eq!(dwarf.strings.count(), 3921);

    // Write to new sections
    let mut write_sections = write::Sections::new(EndianVec::new(LittleEndian));
    dwarf
        .write(&mut write_sections)
        .expect("Should write DWARF information");
    let debug_info_data = write_sections.debug_info.slice();
    let debug_abbrev_data = write_sections.debug_abbrev.slice();
    let debug_line_data = write_sections.debug_line.slice();
    let debug_ranges_data = write_sections.debug_ranges.slice();
    let debug_str_data = write_sections.debug_str.slice();
    assert_eq!(debug_info_data.len(), 394_930);
    assert_eq!(debug_abbrev_data.len(), 9701);
    assert_eq!(debug_line_data.len(), 105_797);
    assert_eq!(debug_ranges_data.len(), 155_712);
    assert_eq!(debug_str_data.len(), 144_731);

    // Convert new sections
    let debug_abbrev = read::DebugAbbrev::new(debug_abbrev_data, LittleEndian);
    let debug_info = read::DebugInfo::new(debug_info_data, LittleEndian);
    let debug_line = read::DebugLine::new(debug_line_data, LittleEndian);
    let debug_str = read::DebugStr::new(debug_str_data, LittleEndian);
    let debug_ranges = read::DebugRanges::new(debug_ranges_data, LittleEndian);
    let debug_rnglists = read::DebugRngLists::new(&[], LittleEndian);

    let ranges = gimli::RangeLists::new(debug_ranges, debug_rnglists);

    let dwarf = read::Dwarf {
        debug_abbrev,
        debug_info,
        debug_line,
        debug_str,
        ranges,
        ..Default::default()
    };

    let dwarf = write::Dwarf::from(&dwarf, &|address| Some(Address::Absolute(address)))
        .expect("Should convert DWARF information");

    assert_eq!(dwarf.units.count(), 23);
    let entries: usize = (0..dwarf.units.count())
        .map(|i| dwarf.units.get(dwarf.units.id(i)).count())
        .sum();
    assert_eq!(entries, 29_560);
    assert_eq!(dwarf.strings.count(), 3921);
}
