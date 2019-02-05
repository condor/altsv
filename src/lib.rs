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
        let value_to_insert: Option<CString>;
        if value.is_empty() {
            value_to_insert = None;
        } else {
            value_to_insert = Some(CString::new(value).expect("This must success since it is from String"))
        }
        let key_to_insert = Symbol::from_string(key.clone());

        self.map.insert(key_to_insert, value_to_insert);
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

const METHOD_NAME_RESPOND_TO: &[u8] = b"respond_to?\0`";
const METHOD_NAME_TO_H: &[u8] = b"to_h\0";
const METHOD_NAME_TO_HASH: &[u8] = b"to_hash\0";
const METHOD_NAME_TO_S: &[u8] = b"to_s\0";

const MESSAGE_DUMP_ERROR: &'static str = "Argument does not respond to neither :to_h nor :to_hash";

unsafe fn rb_intern_u8(method_name: &[u8]) -> ID {
    use std::mem;
    rb_intern(mem::transmute::<&u8, c_string>(&method_name[0]))
}

unsafe fn respond_to(object: VALUE, name: ID) -> bool {
    let respond_to = rb_intern_u8(METHOD_NAME_RESPOND_TO);
    let method = rb_id2sym(name);

    rb_funcall(object, respond_to, 1, method) == Qtrue
}

extern "C" fn join_hash(key: VALUE, value: VALUE, farg: *mut void) -> st_retval {
    use std::mem;

    let s = unsafe {mem::transmute::<*mut void, *mut DumpResult>(farg) };

    match unsafe{ (*s).push_value(key) } {
        None => {}
        Some(error) => {
            unsafe{ (*s).set_error(error) };
            return st_retval::ST_STOP;
        }
    }

    unsafe { (*s).push_char(':') };

    match unsafe{ (*s).push_value(value) } {
        None => {}
        Some(error) => {
            unsafe{ (*s).set_error(error) };
            return st_retval::ST_STOP;
        }
    }
    unsafe { (*s).push_char('\t') };

    st_retval::ST_CONTINUE
}

struct DumpResult {
    error: Option<Error>,
    buffer: String,
}

impl DumpResult {
    fn new(item_count: usize) -> Self{
        DumpResult {
            buffer: String::with_capacity(item_count * (BUFFER_SIZE_KEY + BUFFER_SIZE_VALUE)),
            error: None,
        }
    }
    fn push_value(&mut self, v: VALUE) -> Option<Error>{
        match unsafe {
            if RB_TYPE_P(v, T_SYMBOL) {
                String::from_ruby(rb_id2str(rb_sym2id(v)))
            } else if RB_TYPE_P(v, T_STRING) {
                String::from_ruby(v)
            } else {
                let to_s = rb_intern_u8(&METHOD_NAME_TO_S);
                String::from_ruby(rb_funcall(v, to_s, 0))
            }
        } {
            Ok(checked) => {
                for c in String::from_checked(checked).chars() {
                    match c {
                        '\r' => self.buffer.push_str("\\r"),
                        '\n' => self.buffer.push_str("\\n"),
                        '\t' => self.buffer.push_str("\\t"),
                        ':' => self.buffer.push_str("\\:"),
                        '\\' => self.buffer.push_str("\\\\"),
                        _ => self.buffer.push(c),
                    }
                }
                None
            },
            Err(e) => Some(e),
        }
    }

    fn push_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    fn set_error(&mut self, e: Error) {
        self.error = Some(e);
    }

    fn extract(self) -> Result<String, Error> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(String::from(self.buffer.trim_end())),
        }
    }
}

struct DumpArgument {
    inner: VALUE,
    count: usize,
}

impl DumpArgument {
    pub unsafe fn from_hash(inner: VALUE) -> Self{
        let count = RHASH_SIZE(inner) as usize;

        DumpArgument{
            inner,
            count,
        }
    }
    pub fn dump(&self) -> Result<String, Error> {
        use std::mem;

        let mut dump_result = DumpResult::new(self.count);
        unsafe{
            rb_hash_foreach(
                self.inner,
                join_hash,
                mem::transmute::<&mut DumpResult, *mut void>(&mut dump_result)
            )
        };

        dump_result.extract()
    }
}

impl FromRuby for DumpArgument {
    type Checked = CheckedValue<DumpArgument>;

    fn from_ruby(value: VALUE) -> CheckResult<CheckedValue<DumpArgument>> {
        unsafe {
            if RB_TYPE_P(value, T_HASH) {
                return Ok(CheckedValue::new(value));
            }

            let hashed_value: VALUE;
            let to_h = rb_intern_u8(METHOD_NAME_TO_H);
            if respond_to(value, to_h) {
                hashed_value = rb_funcall(value, to_h, 0);
                return Ok(CheckedValue::new(hashed_value));
            }

            let hashed_value: VALUE;
            let to_h = rb_intern_u8(METHOD_NAME_TO_HASH);
            if respond_to(value, to_h) {
                hashed_value = rb_funcall(value, to_h, 0);
                return Ok(CheckedValue::new(hashed_value));
            }
        }

        raise!(MESSAGE_DUMP_ERROR);
    }

    fn from_checked(checked: CheckedValue<DumpArgument>) -> DumpArgument {
        unsafe{ DumpArgument::from_hash(checked.to_value())}
    }
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

        def dump_native(value: DumpArgument) -> Result<String, Error> {
            value.dump()
        }
    }
}
