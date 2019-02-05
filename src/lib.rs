use std::collections::HashMap;
use std::ffi::CString;

use libcruby_sys::*;

use helix::*;

const BUFFER_SIZE_KEY: usize = 64usize;
const BUFFER_SIZE_VALUE: usize = 128usize;

struct HashMapWrapper {
    map: HashMap<Symbol, Option<CString>>,
}

impl HashMapWrapper {
    fn with_capacity(capacity: usize) -> Self {
        HashMapWrapper {
            map: HashMap::with_capacity(capacity)
        }
    }

    fn insert(&mut self, key: &String, value: &str) {
        unsafe {
            let value_to_insert: Option<CString>;
            if value.is_empty() {
                value_to_insert = None;
            } else {
                value_to_insert = Some(CString::new(value).expect("This must success since it is from String"))
            }
            let key_to_insert = Symbol::from_string(key.clone());

            self.map.insert(key_to_insert, value_to_insert);
        }
    }

    fn inner(self) -> HashMap<Symbol, Option<CString>> {
        self.map
    }
}

impl ToRuby for HashMapWrapper {
    fn to_ruby(self) -> ToRubyResult {
        let hash: VALUE;
        unsafe {
            hash = rb_hash_new();

            for (key, value) in self.inner().into_iter() {
                let key_ruby = key.to_ruby().expect("This must success since it is from String");
                let value_ruby = match value {
                    None => Qnil,
                    Some(v) => rb_utf8_str_new(v.as_ptr(), v.as_bytes().len() as i64),
                };
                rb_hash_aset(hash, key_ruby, value_ruby);
            }
        }

        return Ok(hash);
    }
}

fn parse_line(input: &str) -> HashMapWrapper {
    let mut key = String::with_capacity(BUFFER_SIZE_KEY);
    let mut value = String::with_capacity(BUFFER_SIZE_VALUE);
    let mut in_value = false;
    let mut escaping = false;

    // First: count the key-value pairs.
    let mut count = 0usize;
    let mut chars = input.chars();
    loop {
        match chars.next() {
            None => break,
            Some(c) => {
                match c {
                    '\t' => count += 1,
                    _ => {}
                }
            }
        }
    }

    let mut line=  HashMapWrapper::with_capacity(count);

    // start actual parsing
    let mut chars = input.chars();
    loop {
        match chars.next() {
            None => {
                if !key.is_empty() {
                    line.insert(&key, &value);
                }
                break;
            }
            Some(c) => {
                if c == '\r' || c == '\n' {
                    if !key.is_empty() {
                        line.insert(&key, &value);
                    }
                    break;
                }

                let current_char: char;
                if escaping {
                    match c {
                        'n' => current_char = '\n',
                        'r' => current_char = '\r',
                        't' => current_char = '\t',
                        '\\' => current_char = '\\',
                        _ => {
                            current_char = c;
                            if in_value {
                                value.push('\\')
                            }else {
                                key.push('\\')
                            }
                        },
                    }
                    escaping = false;
                } else {
                    match c {
                        '\\' => {
                            escaping = true;
                            continue;
                        },
                        ':' => {
                            // key-value separator(only when not separated)
                            if !in_value {
                                in_value = true;
                                continue;
                            }
                        }
                        '\t' => {
                            // field separator
                            if !key.is_empty() {
                                // TODO: Whether OR NOT the empty label should cause error?
                                line.insert(&key, &value);
                            }
                            key.clear();
                            value.clear();
                            in_value = false;
                            continue;
                        }
                        _ => {}
                    }
                    current_char = c;
                }

                if in_value {
                    value.push(current_char)
                } else {
                    key.push(current_char)
                }
            }
        }
    }

    line
}

ruby! {
    class Altsv {
        def parse_native(input: String) -> Vec<HashMapWrapper> {
            let mut hashes: Vec<HashMapWrapper> = Vec::new();
            let mut lines = input.lines();
            loop {
                match lines.next() {
                    Some(line) => hashes.push(parse_line(line)),
                    None => break,
                }
            }
            hashes
        }
        def parse_line_native(input: String) -> HashMapWrapper {
            parse_line(&input)
        }
    }
}
