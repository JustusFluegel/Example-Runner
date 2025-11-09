use nom::{character::streaming::digit1, combinator::map_res, IResult};

pub struct Semver {}

fn str_num(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |r| u32::from_str_radix(r, 10))(input)
}

fn semver(input: &str) -> IResult<&str, Semver> {
    todo!()
}
