use std::time::SystemTime;

const TAI_OFFSET: u64 = 4611686018427387914;

#[derive(Debug,PartialEq, Copy, Clone, Eq)]
struct Tai(u64);

fn tai_unix(unix_time: u64) -> Tai {
    Tai(TAI_OFFSET + unix_time)
}

fn now() -> Tai {}

#[test]
fn tai_at_epoch() {
    assert_eq!(Tai(TAI_OFFSET), tai_unix(0));
}


#[test]
fn tai_at_epoch_plus_one() {
    assert_eq!(Tai(TAI_OFFSET + 1), tai_unix(1));
}
