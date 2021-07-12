
use std::ops::Add;
use std::ops::Sub;

#[derive(Clone,Copy,Eq,PartialEq,Hash,Debug)]
pub struct V{pub x: i8, pub y:i8}

impl Default for V {
    fn default() -> Self { V{x:0,y:0} }
}

impl Add<V> for V {
    type Output = V;
    fn add(self, rhs: V) -> Self {
        V{
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl Sub<V> for V {
    type Output = V;
    fn sub(self, rhs: V) -> Self {
        V{
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

