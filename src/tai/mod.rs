use std::time::{Duration,SystemTime};

const TAI_OFFSET: u64 = 4611686018427387914;

#[derive(Debug,PartialEq, Copy, Clone, Eq, Ord, PartialOrd)]
pub struct Tai(u64);

impl ::std::ops::Add for Tai {
    type Output = Tai;

    fn add(self, _rhs: Tai) -> Tai {
        Tai(self.0 + _rhs.0)
    }
}


impl ::std::ops::Sub for Tai {
    type Output = Tai;

    fn sub(self, _rhs: Tai) -> Tai {
        Tai(self.0 - _rhs.0)
    }
}

fn tai_unix(unix_time: u64) -> Tai {
    Tai(TAI_OFFSET + unix_time)
}

pub fn now() -> Tai {
    let now = ::std::time::UNIX_EPOCH.elapsed().unwrap();
    
    tai_unix(now.as_secs())
}

pub fn unpack(packed_tai: &[u8]) -> Tai {
    let mut x: u64  = packed_tai[0] as u64;
    x <<= 8; x+= packed_tai[1] as u64;
    x <<= 8; x+= packed_tai[2] as u64;
    x <<= 8; x+= packed_tai[3] as u64;
    x <<= 8; x+= packed_tai[4] as u64;
    x <<= 8; x+= packed_tai[5] as u64;
    x <<= 8; x+= packed_tai[6] as u64;
    x <<= 8; x+= packed_tai[7] as u64;
    Tai(x)
}

impl Tai {
 pub   fn as_secs(&self) -> u64 {
        self.0
    }
}
#[test]
fn tai_at_epoch() {
    assert_eq!(Tai(TAI_OFFSET), tai_unix(0));
}


#[test]
fn tai_at_epoch_plus_one() {
    assert_eq!(Tai(TAI_OFFSET + 1), tai_unix(1));
}

#[test]
fn less_tais() {
    let t0 = Tai(0);
    let t1 = Tai(1);

    assert!( t0 < t1);
}

#[test]
fn sub_tais() {
    let t0 = Tai(12345);
    let t1 = Tai(12300);

    assert_eq!(Tai(45), t0-t1);
}

#[test]
fn add_tais() {
    let t0 = Tai(123);
    let t1 = Tai(123);

    assert_eq!(Tai(246), t0+t1);
}

#[test]
fn tai_as_secs() {

    let t0 = Tai(123);
    assert_eq!(123, t0.as_secs());
}

#[test]
fn tai_unpack_zero() {
    let array: [u8; 8] = [0; 8];
    assert_eq!(Tai(0), unpack(&array));
}

#[test]
fn tai_unpack_one() {
    let array: [u8; 8] = [0,0,0,0,0,0,0,1];
    assert_eq!(Tai(1), unpack(&array));
}

#[test]
fn tai_unpack() {
    let tai_array: [u8; 8] = [ 0xff, 0, 0xff, 0, 0xff, 0, 0xff, 0 ];
    println!("{:?}", tai_array);
    assert_eq!(Tai(0xff00ff00ff00ff00), unpack(&tai_array));
}
