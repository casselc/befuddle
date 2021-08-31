use crate::stack::FungeStack;

pub struct FungeDelta(isize, isize);
pub type FungeCoordinate = FungeDelta;

pub struct FungePointer {
    location: FungeCoordinate,
    orientation: FungeDelta,
}