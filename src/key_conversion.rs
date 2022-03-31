//This is a giant hack but hopefully it works

use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
use crate::println;

#[derive(Clone)]
pub struct KeyConversionLayout{
    name: &'static str,
    code: &'static str,
    mapping: BTreeMap<char, char>
}
impl KeyConversionLayout{
    pub fn convert_char(&self, old: char) -> char{
        match self.mapping.get(&old) {
            Some(c) => *c,
            None => old
        }
    }
}
lazy_static!{
    pub static ref KEYMAP_DE: KeyConversionLayout = KeyConversionLayout{
        name: "German (Standard)",
        code: "de",
        mapping: BTreeMap::from([
            ('~', '°'),
            ('`', '^'),
            ('@', '"'), //2 key
            ('#', '§'), //3 key
            ('^', '&'), //6 key
            ('&', '/'), //7 key
            ('*', '('), //8 key
            ('(', ')'), //9 key
            (')', '='), //ß key
            ('-', 'ß'),
            (0x5f as char, '?'),
        ])
    };
}
lazy_static!{
    pub static ref KEYMAP_US: KeyConversionLayout = KeyConversionLayout{
        name: "US",
        code: "us",
        mapping: BTreeMap::new(),
    };
}
