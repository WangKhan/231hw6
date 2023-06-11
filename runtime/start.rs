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

#[export_name = "\x01snek_equal"]
fn snek_equal(val1 : i64, val2: i64) -> i64 {
    let mut seen1 = Vec::<i64>::new();
    let mut seen2 = Vec::<i64>::new();
    let str1 = snek_str(val1, &mut seen1);
    let str2 = snek_str(val2, &mut seen1);
    if (str1 == str2) {
        return 7;
    } else {
        return 3;
    }
}



#[export_name = "\x01snek_print"]
fn snek_print(val : i64) -> i64 {
    let mut seen = Vec::<i64>::new();
    println!("{}", snek_str(val, &mut seen));
    return val;
}


fn snek_str(val: i64, seen : &mut Vec<i64>) -> String{
    if val == 7 { "true".to_string() }
    else if val == 3 { "false".to_string()  }
    else if val % 2 == 0 { format!("{}", val >> 1) }
    else if val == 1 { "nil".to_string() }
    else {
        if seen.contains(&val)  { return "(tuple <cyclic>)".to_string()}
        seen.push(val);
        let addr: *const i64 = (val - 1) as *const i64;
        let mut length;
        unsafe {
            length = *(addr);
        }
        length /= 2;
        let mut res = String::new();
        res.push('(');
        let mut res = '('.to_string();
        for num in 1..= length {
            let value;
            let value_addr: *const i64 = (val - 1 + num * 8) as *const i64;
            unsafe {
                value = *(value_addr);
            };
            res.push_str(&snek_str(value, seen));
            res.push(',');
        }
        res.pop();
        res.push(')');
        seen.pop();
        return res;
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
    let mut seen = Vec::<i64>::new();
    snek_print(output);
}
