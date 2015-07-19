#![feature(convert)]
#![feature(slice_patterns)]
use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

type Expr = i64;

struct VM<'a> {
  stack: Vec<Expr>,
  instructions: Vec<&'a str>,
  locals: HashMap<&'a str, Expr>,
}

impl<'a> VM<'a> {
  pub fn new(instructions: Vec<&'a str>) -> VM {
    VM {
      stack: Vec::new(),
      instructions: instructions,
      locals: HashMap::new(),
    }
  }

  pub fn run(&mut self) {
    // just some aliases for brevity
    let len = self.instructions.len();

    // VM variables (might be moved back to the struct itself)
    let mut ip = 0; // instruction pointer
    let mut carry = false;

    while ip < len {
      let instr = self.instructions[ip];
      ip += 1;
      match instr.split(" ").collect::<Vec<_>>().as_slice() {
        ["push", n] => self.stack.push(n.parse::<i64>().unwrap()),
        ["add"] => if let (Some(arg1), Some(arg2)) = (self.stack.pop(), self.stack.pop()) {
          self.stack.push(arg1 + arg2);
        } else {
          println!("VM error: not enough arguments to `add`");
        },

        ["say"] => if let Some(arg) = self.stack.pop() {
          println!("hey {}", arg);
        } else {
          println!("VM error: not enough arguments to `say`");
        },

        //["cmp", "<"] =>

        // how = "carry" | "always"
        ["jump", how, n] => {
          let new_ip = n.parse::<usize>().unwrap();

          if new_ip < len {
            // only check carry if we're not conditionally jumping
            if how != "carry" || carry {
              ip = new_ip;
            }
            // only reset carry if it was used
            if how == "carry" {
              carry = false;
            }
          } else {
            panic!("VM error: trying to jump past end of bytecode ({} > {})", new_ip, len);
          }
        },

        ["carry", "set", val] => carry = val == "true",
        ["carry", "invert"] => carry = !carry,

        // NOTE: it's stack[*-1] OP stack[*-2]
        // which means if have stack=[1, 2]
        // you'll have 2 OP 1
        ["cmp", op] => if let (Some(arg1), Some(arg2)) = (self.stack.pop(), self.stack.pop()) {
          carry = match op {
            "<" => arg1 < arg2,
            ">" => arg1 > arg2,
            "<=" => arg1 <= arg2,
            ">=" => arg1 >= arg2,
            "=" => arg1 == arg2,
            _ => panic!("VM error: unrecognized `cmp` operator: {}", op),
          };
        } else {
          panic!("VM error: not enough arguments to `cmp`"); 
        },

        ["local", "load", name] => match self.locals.get(name) {
          Some(expr) => self.stack.push(expr.clone()),
          None       => panic!("VM error: unknown local variable {}", name),
        },

        ["local", "store", name] => if let Some(arg) = self.stack.pop() {
          let _ = self.locals.insert(name, arg);
        } else {
          panic!("VM error: not enough arguments for `local store`"); 
        },

        [instr, ..] => panic!("VM error: no such instruction {}", instr),
        [..] => panic!("VM error: Unrecognized instruction!"), // todo err!
      }
    }
  }
}

fn main() {
	if let Some(path) = env::args().nth(1) {
		println!("Loading file {}", path);

    match File::open(&path) {
      Ok(mut file) => {
        let mut content = String::new();
        if let Ok(_) = file.read_to_string(&mut content) {
          // remove trailing newlines, split by line
          let instructions = content.trim_matches('\n').split('\n').collect();
          let mut vm = VM::new(instructions);
          vm.run();
        }
      },
      Err(err) => {
        println!("Can't read file {} (error: {})", path, err);
      }
    }
	} else {
    println!("Please give a file argument");
  }

  //     let path = Path::new("chry.fa");
  // let mut file = BufferedReader::new(File::open(&path));
  // for line in file.lines() {
  //  print!("{}", line.unwrap());
  //      }
}
