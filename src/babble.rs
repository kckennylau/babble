// numeric types
extern crate num;
use self::num::rational::BigRational as Rational;
use self::num::bigint::BigInt;
use self::num::traits::ToPrimitive;

use std::rc::Rc;

// adapted from @Shepmaster's code here:
// http://stackoverflow.com/a/27590832/1223693
use std::io::prelude::*;
macro_rules! warn(
    ($($arg:tt)*) => { {
        write!(&mut ::std::io::stderr(), "warn: ").unwrap();
        writeln!(&mut ::std::io::stderr(), $($arg)*).unwrap();
    } }
);

// create a Rational from an int/float respectively
macro_rules! rint(
    ($x:expr) => (
        Rational::from_integer(BigInt::from($x))
    )
);
macro_rules! rfloat(
    ($x:expr) => (
        Rational::from_float($x).unwrap()
    )
);

// the publicly exposed interface
pub struct Babble {
    primary: usize, secondary: usize, result: usize, vars: [Value; 26]
}

// a Value is anything that a variable can be set to
#[derive(Clone)]
enum Value {
    Num(Rational), Arr(Vec<Value>),
    Block(Vec<Rc<Fn(&mut Babble, &mut Write, &mut Read)>>)
}
impl Value {
    fn num(f: f64) -> Value {
        Value::Num(rfloat!(f))
    }
}
impl From<Value> for bool {
    fn from(val: Value) -> bool {
        bool::from(&val)
    }
}
impl<'a> From<&'a Value> for bool {
    fn from(val: &'a Value) -> bool {
        match val {
            &Value::Num(ref n) => *n != rint!(0),
            &Value::Arr(ref a) => !a.is_empty(),
            &Value::Block(ref b) => !b.is_empty()
        }
    }
}

// methods of the main Babble struct
impl Babble {

    // default constructor, initializing all variables
    pub fn new() -> Babble {
        Babble {
            primary: 0, secondary: 1, result: 2,
            vars: [Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0), Value::num(0.0),
                   Value::num(0.0), Value::num(0.0)]
        }
    }

    // run Babble code, using STDOUT and STDIN as I/O
    pub fn run(&mut self, code: String) {
        let (mut stdout, mut stdin) = (::std::io::stdout(), ::std::io::stdin());
        self.run_with_io(code, &mut stdout, &mut stdin);
    }

    // run Babble code, but manually specify I/O interfaces
    pub fn run_with_io(&mut self, code: String,
                       stdout: &mut Write, stdin: &mut Read) {
        for token in Babble::tokenize(code) {
            token(self, stdout, stdin)
        }
    }

    // private function that turns a string of code into a Vec of the functions
    // that the string represents
    fn tokenize(code: String) -> Vec<Rc<Fn(&mut Babble, &mut Write,
                                           &mut Read)>> {
        let mut tokens = Vec::new();
        let mut code_iter = BabbleCodeIterator::new(code);

        while let Some(token) = Babble::parse(&mut code_iter) {
            tokens.push(token);
        }

        tokens
    }

    // this is the top-level parsing function, the "normal" parsing mode
    fn parse(mut code: &mut BabbleCodeIterator)
            -> Option<Rc<Fn(&mut Babble, &mut Write, &mut Read)>> {
        // simply grab three letters and go from there
        let cmd: String = code.take(3).collect();

        // have we run out of characters?
        if cmd.len() < 3 { return None; }

        // check for primary, secondary, or result variable setting commands
        if cmd.starts_with("PV") {
            let pv = Babble::letter_idx(cmd.chars().last().unwrap());
            return Some(Rc::new(move |this, _, _| {
                this.primary = pv;
            }));
        } else if cmd.starts_with("SV") {
            let sv = Babble::letter_idx(cmd.chars().last().unwrap());
            return Some(Rc::new(move |this, _, _| {
                this.secondary = sv;
            }));
        } else if cmd.starts_with("RV") {
            let rv = Babble::letter_idx(cmd.chars().last().unwrap());
            return Some(Rc::new(move |this, _, _| {
                this.result = rv;
            }));
        }

        // a HUGE switch statement...
        match &cmd[..] {
            "XPS" => Some(Rc::new(|this, _, _| {
                this.vars.swap(this.primary, this.secondary);
            })),
            "XPR" => Some(Rc::new(|this, _, _| {
                this.vars.swap(this.primary, this.result);
            })),
            "XSR" => Some(Rc::new(|this, _, _| {
                this.vars.swap(this.secondary, this.result);
            })),

            // literals .......................................................

            // array / string literals
            "ARR" => Babble::parse_literal_array(&mut code),

            // block literals
            "BLK" => Babble::parse_literal_block(&mut code),

            // arbitrarily sized number literals
            "NUM" => Babble::parse_literal_num(&mut code),

            // small number literals
            "ZRO" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(0.0)
            })),
            "ONE" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(1.0)
            })),
            "TWO" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(2.0)
            })),
            "TRE" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(3.0)
            })),
            "FOR" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(4.0)
            })),
            "FIV" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(5.0)
            })),
            "SIX" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(6.0)
            })),
            "SVN" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(7.0)
            })),
            "EGT" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(8.0)
            })),
            "NIN" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(9.0)
            })),
            "TEN" => Some(Rc::new(|this, _, _| {
                this.vars[this.primary] = Value::num(10.0)
            })),

            // math ...........................................................

            // basic arithmetic
            "ADD" => Some(Rc::new(|this, _, _| {
                this.vars[this.result] = Value::Num(
                    match this.vars[this.primary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    } + match this.vars[this.secondary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    });
            })),
            "SUB" => Some(Rc::new(|this, _, _| {
                this.vars[this.result] = Value::Num(
                    match this.vars[this.primary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    } - match this.vars[this.secondary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    });
            })),
            "MUL" => Some(Rc::new(|this, _, _| {
                this.vars[this.result] = Value::Num(
                    match this.vars[this.primary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    } * match this.vars[this.secondary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    });
            })),
            "DIV" => Some(Rc::new(|this, _, _| {
                this.vars[this.result] = Value::Num(
                    match this.vars[this.primary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    } / match this.vars[this.secondary] {
                        Value::Num(ref n) => n.clone(),
                        _ => rint!(0)
                    });
            })),

            // block operators ................................................

            // control flow
            "WHL" => Some(Rc::new(|this, stdout, stdin| {
                let condition = match this.vars[this.primary] {
                    Value::Block(ref b) => b.clone(),
                    _ => vec![]
                };
                let statement = match this.vars[this.secondary] {
                    Value::Block(ref b) => b.clone(),
                    _ => vec![]
                };
                while {
                    for token in condition.iter() { token(this, stdout, stdin) }
                    bool::from(&this.vars[this.result])
                } {
                    for token in statement.iter() { token(this, stdout, stdin) }
                }
            })),

            // I/O ............................................................

            // stdin/stdout
            "PUT" => Some(Rc::new(|this, stdout, _| {
                match this.vars[this.primary] {
                    // for numbers, simply output the number
                    Value::Num(ref n) => {
                        write!(stdout, "{}", n).unwrap();
                    },
                    // for arrays, treat them as arrays of ASCII codes
                    Value::Arr(ref a) => for v in a { match v {
                        &Value::Num(ref n) => {
                            let mut val = n.clone();
                            while val != rint!(0) {
                                let byte = (val.clone() % rint!(256))
                                    .to_integer().to_u8().unwrap();
                                stdout.write(&[byte]).unwrap();
                                val = (val / rint!(256)).floor();
                            }
                        },
                        _ => warn!("PUT called on array with ignored \
                                   non-Num element")
                    } },
                    // doesn't make sense to PUT a block
                    Value::Block(_) => warn!("PUT called on block ignored")
                }
            })),

            // if the function is unknown, ignore .............................

            _ => Some(Rc::new(|_, _, _| {}))
        }
    }

    // when we encounter "ARR", this sub-parsing function is called, which
    // consumes the code iterator until the ending sequence (ZE)
    fn parse_literal_array(mut code: &mut BabbleCodeIterator)
            -> Option<Rc<Fn(&mut Babble, &mut Write, &mut Read)>> {
        let mut arr: Vec<Value> = Vec::new();

        // I am good at naming loops
        'loopy: loop {
            let ch = if let Some(x) = code.next() { x } else { return None };
            if ch == 'Z' {
                match if let Some(x) = code.next() { x } else { return None } {
                    // ZE: [e]nd array literal
                    'E' => break 'loopy,
                    // ZL; [l]owercase letter
                    'L' => {
                        let ch3 = if let Some(x) = code.next() { x }
                                  else { return None };
                        arr.push(Value::num(ch3 as u8 as f64 + 32.0));
                    },
                    // ZT: [t]wo base 25 digits
                    'T' => {
                        let ch3 = if let Some(x) = code.next() { x }
                                  else { return None };
                        let ch4 = if let Some(x) = code.next() { x }
                                  else { return None };
                        arr.push(Value::Num(Babble::from_letter_base(
                                    vec![ch3, ch4])));
                    },
                    // ZZ: a literal Z character
                    'Z' => arr.push(Value::num('Z' as u8 as f64)),
                    _ => {}
                }
            } else {
                arr.push(Value::num(ch as u8 as f64));
            }
        }

        // return the array as a function that sets the primary variable
        Some(Rc::new(move |this, _, _| {
            this.vars[this.primary] = Value::Arr(arr.to_owned());
        }))
    }

    fn parse_literal_block(mut code: &mut BabbleCodeIterator)
            -> Option<Rc<Fn(&mut Babble, &mut Write, &mut Read)>> {
        let mut tokens = Vec::new();

        while !code.is_ending() {
            if let Some(token) = Babble::parse(&mut code) {
                tokens.push(token);
            } else { break; }
        }

        // consume the END
        for _ in code.take(3) {}

        Some(Rc::new(move |this, _, _| {
            this.vars[this.primary] = Value::Block(tokens.to_owned());
        }))
    }

    fn parse_literal_num(mut code: &mut BabbleCodeIterator)
            -> Option<Rc<Fn(&mut Babble, &mut Write, &mut Read)>> {
        let mut digits1: Option<Vec<char>> = None;
        let mut digits2: Option<Vec<char>> = None;
        let mut num_zs = 0;

        while let Some(ch) = code.next() {
            if ch == 'Z' {
                if digits1.is_none() {
                    // if digits1 is still none, we're still counting Zs
                    num_zs += 1;
                } else if num_zs >= 2 && digits2.is_none() {
                    // if we have 2 or more preceding Zs, start the denominator
                    digits2 = Some(Vec::new());
                } else {
                    // otherwise, this marks the end of the literal
                    break;
                }
            } else {
                if let Some(ref mut digits) = digits2 {
                    digits.push(ch);
                } else {
                    if let Some(ref mut digits) = digits1 {
                        digits.push(ch);
                    } else {
                        digits1 = Some(vec![ch]);
                    }
                }
            }
        }

        let numer = digits1.map_or(rint!(0), |x| Babble::from_letter_base(x))
            * if num_zs % 2 == 0 { rint!(1) } else { rint!(-1) };
        let denom = digits2.map_or(rint!(1), |x| Babble::from_letter_base(x));

        let result = numer / denom;

        Some(Rc::new(move |this, _, _| {
            this.vars[this.primary] = Value::Num(result.to_owned());
        }))
    }

    // convert an uppercase character to its index in the alphabet, 0-indexed
    // A=0, B=1, etc.
    fn letter_idx(ch: char) -> usize {
        (ch as usize) - 65
    }

    // convert a Vec of digits in "letter base" (A..Y) to a number
    fn from_letter_base(digits: Vec<char>) -> Rational {
        let mut num = rint!(0);
        let mut mult = rint!(1);

        for &digit in digits.iter().rev() {
            num = num + rint!(Babble::letter_idx(digit)) * mult.clone();
            mult = mult * rint!(25);
        }

        num
    }
}

// a custom iterator through the characters of a Babble program
struct BabbleCodeIterator { code: Vec<char> }
impl BabbleCodeIterator {
    fn new(code: String) -> BabbleCodeIterator {
        BabbleCodeIterator {
            code: code.chars().rev().filter(|x| x.is_uppercase()).collect()
        }
    }
}
impl Iterator for BabbleCodeIterator {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.code.pop()
    }
}
impl BabbleCodeIterator {
    fn is_ending(&self) -> bool {
        self.code.ends_with(&['D','N','E'])
    }
}
