use std::cell::RefCell;

#[derive(Debug)]
struct Struct<'a> {
    strings: RefCell<Box<&'a mut Vec<String>>>
}

impl<'a> Struct<'a> {
    fn new() -> Struct<'a> {
        let mut strings = Vec::new();
        Struct {
            strings: RefCell::new(Box::new(&mut strings))
        }
    }
}

fn make_t<'a: 'b, 'b>(s: &'b Struct<'a>) -> Struct<'b> {
    let mut borrowed = s.strings.borrow_mut();
    Struct {
        strings: RefCell::new(Box::new(&mut *borrowed))
    }
}

fn borrow(s: &Struct) {
    let t = make_t(s);
    println!("t.strings.length() {}", t.strings.borrow().len());
    t.strings.borrow_mut().push("test".to_string());
    println!("AFTER: t.strings.length() {}", t.strings.borrow().len());
}

fn main() {
    let s = Struct::new();
    println!("s.strings.length() {}", s.strings.borrow().len());
    borrow(&s);
    println!("AFTER: s.strings.length() {}", s.strings.borrow().len());
}
