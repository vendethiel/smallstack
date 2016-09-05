#![feature(slice_patterns)]
use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

#[derive(Clone)]
enum Expr {
  Int(i64),
  Str(String)
}

// XXX this should probably be split up. The VM has the instructions, a "Thread" has locals, stack, and arguments.
struct VM<'a> {
  instructions: Vec<&'a str>,
  labels: HashMap<&'a str, usize>,
}

fn parse_labels<'a>(instructions: &Vec<&'a str>) -> HashMap<&'a str, usize> {
  let mut map = HashMap::new();
  for (i, instr) in instructions.iter().enumerate() {
    if instr.starts_with("$label") {
      match instr.split(' ').nth(1) {
        Some(label) => map.insert(label, i),
        None => panic!("VM error: missing label"),
      };
    }
  };
  map
}

impl<'a> VM<'a> {
  pub fn new(instructions: Vec<&'a str>) -> VM {
    VM {
      labels: parse_labels(&instructions),
      instructions: instructions,
    }
  }

  pub fn run(&mut self) {
    let ip = self.labels.get("main").unwrap_or(&0).clone();
    self.run_call(ip, &mut Vec::new());
  }

  // TODO locals
  pub fn run_call(&mut self, start_ip: usize, stack: &mut Vec<Expr>) -> Option<Expr> {
    let mut ip = start_ip;
    let len = self.instructions.len();
    //let mut locals = HashMap::new();
    let locals: &mut HashMap<&'a str, Expr> = &mut HashMap::new();

    let mut carry = false; // comparison carry

    while ip < len {
      let instr = self.instructions[ip];
      ip += 1;
      if instr.starts_with("#") || instr.trim() == "" {
        continue;
      }
      match instr.split(" ").collect::<Vec<_>>().as_slice() {
        &["$label", _] => (),
        &["push", "int", n] => stack.push(Expr::Int(n.parse::<i64>().unwrap())),
        &["push", "str", ref n..] => stack.push(Expr::Str(String::from(n.join(" ")))),

        &["dup"] => {
          let expr = stack.pop().expect("Nothing on the stack to pop");
          stack.push(expr.clone());
          stack.push(expr);
        },

        // how = "always" | "carry"
        &["jump", how, n] => {
          let new_ip = if n.starts_with("$") {
            self.labels.get(n.trim_matches('$')).expect("VM error: no such label").clone()
          } else {
            n.parse::<usize>().unwrap()
          };

          if new_ip < len {
            // only check carry if we are conditionally jumping
            if how == "always" || (how == "carry" && carry) {
              ip = new_ip;
            }
          } else {
            panic!("VM error: trying to jump past end of bytecode ({} > {})", new_ip, len);
          }
        },

        &["carry", "set", val] => carry = val == "true",
        &["carry", "invert"] => carry = !carry,

        &["call", "primitive", "say"] => if let Some(Expr::Str(arg)) = stack.pop() {
          println!("{}", arg);
        } else {
          panic!("VM error: incorrect arguments to `say`");
        },

        &["call", "primitive", "concat"] => if let
            (Some(Expr::Str(arg1)), Some(Expr::Str(arg2)))
            =
            (stack.pop(), stack.pop()) {
          stack.push(Expr::Str(arg1 + &arg2));
        } else {
          panic!("VM error: incorrect arguments to `concat`");
        },

        &["call", "primitive", "typeof"] => match stack.pop() {
            Some(Expr::Str(_)) => stack.push(Expr::Str(String::from("str"))),
            Some(Expr::Int(_)) => stack.push(Expr::Str(String::from("int"))),
            None => panic!("VM error: no argument supplied to `typeof`"),
        },

        &["call", "primitive", "int2str"] => if let Some(Expr::Int(arg)) = stack.pop() {
            stack.push(Expr::Str(arg.to_string()));
        }
        else {
            panic!("VM error: bad arguments to primitive call `int2str`");
        },

        // TODO resolve name in scope, check type, apply
        &["call", arity_str, name] => {
          // TODO assert arityStr <= stack.len()
          let arity = arity_str.parse::<usize>().unwrap();
          let stack_len = stack.len();
          let mut arguments = stack.split_off(stack_len - arity);
          let new_ip = self.labels.get(name).expect("VM error: no such label").clone();
          let ret = self.run_call(new_ip, &mut arguments);
          match ret {
            Some(val) => stack.push(val),
            None => (),
          };
        },

        &["ret"] => {
          return None;
        },

        &["ret", "val"] => {
          if stack.len() != 1 {
            panic!("VM error: trying to return a value but the stack size is not 1 ({})", stack.len());
          }
          return stack.pop();
        },

        // NOTE: it's stack[*-1] OP stack[*-2]
        // which means if have stack=[1, 2]
        // you'll have 2 OP 1
        /// XXX this should JUST be a call!
        &["math", op] => if let
            (Some(Expr::Int(arg1)), Some(Expr::Int(arg2)))
            =
            (stack.pop(), stack.pop()) {
          let value = match op {
            "+" => arg1 + arg2,
            "-" => arg1 - arg2,
            //"/" => TODO. this should have very specific rules...
            "*" => arg1 * arg2,
            _ => panic!("VM error: unrecognized `op` operator: {}", op),
          };
          stack.push(Expr::Int(value));
        } else {
          panic!("VM error: not enough arguments to `math`");
        },

        // same note about "2 OP 1"
        // XXX same note about how this should be a `call`
        &["cmp", op] => carry = match (stack.pop(), stack.pop()) {
            (Some(Expr::Int(arg1)), Some(Expr::Int(arg2))) => match op {
                "<" => arg1 < arg2,
                ">" => arg1 > arg2,
                "<=" => arg1 <= arg2,
                ">=" => arg1 >= arg2,
                "=" => arg1 == arg2,
                _ => panic!("VM error: unrecognized `cmp` operator for int: {}", op),
            },
            (Some(Expr::Str(arg1)), Some(Expr::Str(arg2))) => match op {
                "=" => arg1 == arg2,
                _ => panic!("VM error: unrecognized `cmp` operator for str: {}", op),
            },
            _ => panic!("VM error: not enough arguments to `cmp`"),
        },

        // TODO this *shouldn't* be an identifier
        //      the backend should transform SSAF to tons of load/store
        //      (probably)
        //      (see backend#1)
        &["local", "load", name] => match locals.get(name) {
          Some(expr) => stack.push(expr.clone()),
          None       => panic!("VM error: unknown local variable {}", name),
        },

        &["local", "store", name] => if let Some(arg) = stack.pop() {
          // XXX check
          let _ = locals.insert(name, arg);
        } else {
          panic!("VM error: not enough arguments for `local store`");
        },

        &[instr, ..] => panic!("VM error: Unrecognized instruction {}!", instr),
        &[..] => panic!("VM error: Unrecognized instruction!"),
      };
    }

    None
  }
}

fn main() {
    let mut content = String::new();
    let result;

	if let Some(path) = env::args().nth(1) {
		println!("Loading file {}", path);

        match File::open(&path) {
          Ok(mut file) => result = file.read_to_string(&mut content),
          Err(err) => {
            panic!("Can't read file {} (error: {})", path, err);
          }
        }
    } else {
        result = std::io::stdin().read_to_string(&mut content);
   }

   if let Ok(_) = result {
     // remove trailing newlines, split by line
     let instructions = content.trim_matches('\n').split('\n').collect();
     let mut vm = VM::new(instructions);
     vm.run();
   }

  //     let path = Path::new("chry.fa");
  // let mut file = BufferedReader::new(File::open(&path));
  // for line in file.lines() {
  //  print!("{}", line.unwrap());
  //      }
}
