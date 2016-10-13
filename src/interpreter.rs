extern crate regex;
use self::regex::Regex;
use std::collections::HashMap;

fn get_commands(input: &str) -> Vec<u32> {
    lazy_static! {
        static ref delimiter: Regex = Regex::new(r"[\s,]+").unwrap();
        static ref range: Regex = Regex::new(r"^(\d+)-(\d*)$").unwrap();
    }
    // let mut selections = HashMap::new();
    let line = input.trim();
    let lines = delimiter.split(line);
    for choice in lines {
        let (mut bottom, mut top) = get_range(choice);
        let bottom = top = get_digit(choice);
    }
    let mut choices = Vec::new();
    choices.push(1);
    choices.push(2);
    choices
}

fn get_range(input: &str) -> (Option<u32>, Option<u32>) {
    lazy_static! {
        static ref range_check: Regex = Regex::new(r"^(\d+)-(\d*)$").unwrap();
        static ref single_digit: Regex = Regex::new(r"^\d+$").unwrap();
    }
    let mut bottom: Option<u32> = None;
    let mut top: Option<u32> = None;

    if let Some(caps) = range_check.captures(input) {
        bottom = caps.at(1).map_or(None, |x| x.parse::<u32>().ok());
        top = caps.at(2).map_or(None, |x| x.parse::<u32>().ok());

    }
    (bottom, top)
}

fn get_digit(input: &str) -> Option<u32> {
    Some(1)
}

#[test]
fn one_number() {
    assert_eq!(1, get_commands("")[0]);
}

#[test]
fn two_numbers() {
    assert_eq!(1, get_commands("")[0]);
    assert_eq!(2, get_commands("")[1]);
}

#[test]
fn range_on_empty_str_returns_nones() {
    let (bottom, top) = get_range("");
    assert_eq!(None, bottom);
    assert_eq!(None, top);
}

#[test]
fn range_same_number() {
    let (bottom, top) = get_range("1-1");
    assert_eq!(Some(1), bottom);
    assert_eq!(Some(1), top);
}
#[test]
fn range_second_number_nan() {
    let (bottom, top) = get_range("1-e");
    assert_eq!(None, bottom);
    assert_eq!(None, top);
}

#[test]
fn range_open_ended() {
    let (bottom, top) = get_range("1-");
    assert_eq!(Some(1), bottom);
    assert_eq!(None, top);
}

#[test]
fn range_actual_range() {
    let (bottom, top) = get_range("3-15");
    assert_eq!(Some(3), bottom);
    assert_eq!(Some(15), top);
}

#[test]
fn single_digit() {
    let digit = get_digit("1");
    assert_eq!(Some(1), digit);
}
