use std::ops::Add;
use std::ops::Sub;

use crate::vector::V;

use std::fmt::Display;
use std::fmt;

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug)]
pub struct Dir(pub u8);
impl Dir {
    pub fn v(self) -> V {
        match self.0 {
            0 => V{ x: 1, y: 0},
            1 => V{ x: 1, y: 1},
            2 => V{ x: 0, y: 1},
            3 => V{ x:-1, y: 1},
            4 => V{ x:-1, y: 0},
            5 => V{ x:-1, y:-1},
            6 => V{ x: 0, y:-1},
            7 => V{ x: 1, y:-1},
            d => panic!("invalid direction: {}", d),
        }
    }
    pub const RIGHT:Dir     = Dir(0);
    pub const UPRIGHT:Dir   = Dir(1);
    pub const UP:Dir        = Dir(2);
    pub const UPLEFT:Dir    = Dir(3);
    pub const LEFT:Dir      = Dir(4);
    pub const DOWNLEFT:Dir  = Dir(5);
    pub const DOWN:Dir      = Dir(6);
    pub const DOWNRIGHT:Dir = Dir(7);

    pub fn u8(self) -> u8 { self.0 }

    pub fn reverse(self) -> Dir {
        self + Dir(4)
    }


    pub fn interpolate(self, rhs: Dir) -> Option<Vec<Dir>> {
        let n = (rhs - self).0;
        if n == 4 { // do not interpolate opposite directions
            None
        } else if n < 4 {
            let mut results = Vec::new();
            for diff in 0..=n {
                results.push(self + Dir(diff))
            }
            Some(results)
        } else { // n > 4
            let mut results = Vec::new();
            for diff in (n..=8).rev() {
                results.push(self + Dir(diff % 8))
            }
            Some(results)
        }
    }

}

impl Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.0 {
            0 => "→",
            1 => "↗",
            2 => "↑",
            3 => "↖",
            4 => "←",
            5 => "↙",
            6 => "↓",
            7 => "↘",
            _ => panic!("impossible direction"),
        };
        write!(f, "{}", s)
    }
}

impl Add<Dir> for Dir {
    type Output = Dir;
    fn add(self, rhs: Dir) -> Dir {
        Dir(
            (self.0 + rhs.0) % 8
        )
    }
}

impl Sub<Dir> for Dir {
    type Output = Dir;
    fn sub(self, rhs: Dir) -> Dir {
        Dir(
            (self.0 + 8 - rhs.0) % 8
        )
    }
}
