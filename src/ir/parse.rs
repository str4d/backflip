use nom::{
    branch::alt,
    bytes::{
        complete::{tag, take_while1},
        streaming::take_till,
    },
    character::{
        complete::{digit1, newline},
        is_digit, is_newline,
    },
    combinator::{map, map_res, opt},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};

use super::{Button, ButtonKind, ParsedButton, RawButton};

fn comment(input: &str) -> IResult<&str, &str> {
    map(
        delimited(tag("#"), take_till(|c| is_newline(c as u8)), newline),
        |s: &str| s.trim(),
    )(input)
}

fn line(key: &'static str) -> impl FnMut(&str) -> IResult<&str, &str> {
    move |input| {
        delimited(
            pair(tag(key), tag(": ")),
            take_till(|c| is_newline(c as u8)),
            newline,
        )(input)
    }
}

fn header(input: &str) -> IResult<&str, (&str, u32)> {
    pair(line("Filetype"), map_res(line("Version"), str::parse))(input)
}

fn parsed_button(input: &str) -> IResult<&str, ParsedButton> {
    preceded(
        pair(tag("type: parsed"), newline),
        map(
            tuple((line("protocol"), line("address"), line("command"))),
            |(protocol, address, command)| ParsedButton {
                protocol: protocol.to_owned(),
                address: address.to_owned(),
                command: command.to_owned(),
            },
        ),
    )(input)
}

fn u32_str(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

fn f32_str(input: &str) -> IResult<&str, f32> {
    map_res(take_while1(|c| is_digit(c as u8) || c == '.'), str::parse)(input)
}

fn on_off(input: &str) -> IResult<&str, (u32, u32)> {
    separated_pair(u32_str, tag(" "), u32_str)(input)
}

fn raw_button(input: &str) -> IResult<&str, RawButton> {
    preceded(
        pair(tag("type: raw"), newline),
        map(
            tuple((
                delimited(tag("frequency: "), u32_str, newline),
                delimited(tag("duty_cycle: "), f32_str, newline),
                delimited(
                    tag("data: "),
                    pair(
                        separated_list1(tag(" "), on_off),
                        opt(preceded(tag(" "), u32_str)),
                    ),
                    newline,
                ),
            )),
            |(frequency, duty_cycle, (data, final_on))| RawButton {
                frequency,
                duty_cycle,
                data,
                final_on,
            },
        ),
    )(input)
}

fn button(input: &str) -> IResult<&str, Button> {
    map(
        pair(
            line("name"),
            alt((
                map(parsed_button, ButtonKind::Parsed),
                map(raw_button, ButtonKind::Raw),
            )),
        ),
        |(name, kind)| Button {
            name: name.to_owned(),
            kind,
        },
    )(input)
}

pub(super) fn ir_file(input: &str) -> IResult<&str, Vec<Button>> {
    preceded(pair(header, comment), separated_list1(comment, button))(input)
}
