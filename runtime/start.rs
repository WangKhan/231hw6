use std::env;

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: i64, starting_addr: *mut u8) -> i64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    // TODO: print error message according to writeup
    if errcode == 99 {
        eprintln!("invalid argument, the type of argument is wrong");
    } else if errcode == 101 {
        eprintln!("overflow");
    } else if errcode == 100 {
        eprintln!("value is not a tuple, can't use index to look up");
    } else if errcode == 102 {
        eprintln!("index out of bound");
    }else {
        eprintln!("an error ocurred {errcode}");
    }
    std::process::exit(1);
}

#[export_name = "\x01snek_println"]
pub extern "C" fn snek_println(val : i64) -> i64 {
    if val == 7 { println!("true"); }
    else if val == 3 { println!("false"); }
    else if val % 2 == 0 { println!("{}", val >> 1); }
    else if val == 1 { println!("nil"); }
    else {
        print!("( ");
        let addr: *const i64 = (val - 1) as *const i64;
        let mut length;
        unsafe {
            length = *(addr);
        }
        length /= 2;
        for num in 1..= length {
            let value;
            let value_addr: *const i64 = (val - 1 + num * 8) as *const i64;
            unsafe {
                value = *(value_addr);
            };
            snek_print(value);
            print!(" ");
        }
        print!(")\n");
    }
    return val;
}

fn snek_print(val: i64) {
    if val == 7 { print!("true"); }
    else if val == 3 { print!("false"); }
    else if val % 2 == 0 { print!("{}", val >> 1); }
    else if val == 1 {print!("nil");}
    else {
        print!("( ");
        let addr: *const i64 = (val - 1) as *const i64;
        let mut length;
        unsafe {
            length = *(addr);
        }
        length /= 2;
        for num in 1..= length {
            let value;
            let value_addr: *const i64 = (val - 1 + num * 8) as *const i64;
            unsafe {
                value = *(value_addr);
            };
            snek_print(value);
            print!(" ");
        }
        print!(")");
    }
}

fn parse_input(input: &str) -> i64 {
    // TODO: parse the input string into internal value representation
    if input == "false" {
        3
    } else if input == "true" {
        7
    } else {
        input.parse::<i64>().expect("illegal argument") << 1
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() == 2 { &args[1] } else { "false" };
    let input = parse_input(&input);
    // Allocate a large memory space
    let total_size: usize = 1024 * 8 ; // 1024 * 8 byte
    let mut data= Vec::with_capacity(total_size);
    let starting_addr : *mut u8 = data.as_mut_ptr();
    let output: i64 = unsafe { our_code_starts_here(input, starting_addr) };
    snek_println(output);
}
